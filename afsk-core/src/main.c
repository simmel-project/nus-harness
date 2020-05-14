#include <stdio.h>
#include <stdint.h>
#include <sys/stat.h>
#include <string.h>
#include <stdlib.h>
#include <fcntl.h>

#ifndef NO_MAIN
#include <unistd.h>
#endif

#include "esplanade_demod.h"
#include "esplanade_mac.h"
#include "murmur3.h"

void putBitMac(int bit);

struct demod_config {
    uint32_t sample_rate;
    uint32_t f_lo;
    uint32_t f_hi;
    uint32_t filter_width;
    uint32_t baud_rate;
};

uint32_t debug_print_sync = 0;

// static int dst_fd;

// static int skip_bits = 0;
// static void put_bit(int bit) {
//     static uint8_t counter = 0;
//     static uint8_t accumulator = 0;

//     if (skip_bits) {
//         skip_bits--;
//         return;
//     }

//     if (bit) accumulator |= 1 << (7 - counter);
//     // accumulator |= 1<<(counter);
//     counter++;
//     if (counter >= 8) {
//         write(dst_fd, &accumulator, 1);
//         // printf("byte: %3d - %c\n", accumulator, accumulator);
//         counter = 0;
//         accumulator = 0;
//     }
// }

// static int get_bit(void *data) {
//     int fd = (int)data;
//     static char offset = 8;
//     static char current_byte;
//     if (offset > 7) {
//         current_byte = 0;
//         read(fd, &current_byte, 1);
//         offset = 0;
//     }
//     return !!(current_byte & (1 << (7 - offset++)));
// }

static int validate_packet(demod_pkt_t *pkt, int should_print) {
    unsigned int i;
    demod_pkt_ctrl_t *cpkt = &pkt->ctrl_pkt;
    demod_pkt_data_t *dpkt = &pkt->data_pkt;
    uint32_t hash;

    if (should_print) {
        printf("Got packet:\n");
        printf("    Header:\n");
        printf("        Version: %d\n", pkt->header.version);
        printf("           Type: %d  ", pkt->header.type);
    }
    switch (pkt->header.type) {

    case PKTTYPE_CTRL:
        if (should_print) {
            printf("Ctrl Packet\n");
            printf("       Reserved: %d\n", cpkt->reserved);
            printf("         Length: %d\n", cpkt->length);
            printf("      Full Hash: %08x\n", cpkt->fullhash);
            printf("           GUID: ");
            for (i = 0; i < 16; i++) printf("%02x ", cpkt->guid[i]);
            printf("\n");
            printf("    Packet Hash: %08x ", cpkt->hash);
        }
        MurmurHash3_x86_32((uint8_t *)cpkt, sizeof(*cpkt) - sizeof(cpkt->hash),
                           MURMUR_SEED_BLOCK, &hash);
        if (hash != cpkt->hash) {
            if (should_print) printf("!= %08x\n", hash);
            return 0;
        }
        if (should_print) printf("Ok\n");
        break;

    case PKTTYPE_DATA:
        // unstripe the transition xor's used to keep baud sync. We
        // don't xor the header or the ending hash, but xor
        // everything else..
        for (i = sizeof(pkt->header); i < sizeof(*dpkt) - sizeof(dpkt->hash);
             i++) {
            if (pkt->header.version == PKT_VER_1) {
                // baud striping on alpha and before
                if ((i % 16) == 7)
                    ((uint8_t *)pkt)[i] ^= 0x55;
                else if ((i % 16) == 15)
                    ((uint8_t *)pkt)[i] ^= 0xAA;
            } else if (pkt->header.version == PKT_VER_2) {
                // more dense baud striping to be used on beta and
                // beyond
                if ((i % 3) == 0)
                    ((uint8_t *)pkt)[i] ^= 0x35;
                else if ((i % 3) == 1)
                    ((uint8_t *)pkt)[i] ^= 0xac;
                else if ((i % 3) == 2)
                    ((uint8_t *)pkt)[i] ^= 0x95;
            }
        }

        if (should_print) {
            printf("Data Packet\n");
            printf("   Block Number: %d\n", dpkt->block);
            printf("    Packet Hash: %08x ", dpkt->hash);
        }
        /* Make sure the packet's hash is correct. */
        MurmurHash3_x86_32((uint8_t *)dpkt, sizeof(*dpkt) - sizeof(dpkt->hash),
                           MURMUR_SEED_BLOCK, &hash);
        if (hash != dpkt->hash) {
            if (should_print) printf("!= %08x\n", hash);
            return 0;
        }
        if (should_print) printf("Ok\n");
        break;

    default:
        if (should_print) printf(" (unknown type)\n");
        return 0;
        break;
    }
    return 1;
}

#ifdef NO_MAIN
uint32_t attempt_demodulation(struct demod_config *cfg, int16_t *samples,
                         uint32_t nsamples) {
    FSK_demod_const demod_table;
    FSK_demod_state demod_state;
    struct mac_state mac_state;
    int packet_count = 0;
    int corrupt_count = 0;
    demod_pkt_t packet;

    memset(&mac_state, 0, sizeof(mac_state));
    fsk_demod_generate_table(&demod_table, cfg->baud_rate, cfg->sample_rate,
                             cfg->f_lo, cfg->f_hi, cfg->filter_width);
    fsk_demod_init(&demod_table, &demod_state);

    uint32_t remaining_words = nsamples;
    int16_t *decoding_buffer_offset = samples;

    while (remaining_words > 0) {
        int bit = 0;
        int result = fsk_demod(&demod_table, &demod_state, &bit,
                               decoding_buffer_offset, remaining_words);
        if (result != -1) {
            if (mac_put_bit(&mac_state, bit, &packet, sizeof(packet))) {
                if (validate_packet(&packet, 0)) {
                    packet_count++;
                } else {
                    corrupt_count++;
                }
            }
            decoding_buffer_offset += (remaining_words - result);
            remaining_words = result;
        }
    }
    return packet_count;
}
#else
int main(int argc, char **argv) {
    FSK_demod_const demod_table;
    FSK_demod_state demod_state;
    struct mac_state mac_state = {};
    // FSK_mod_state mod_state;
    int src_fd;
    // uint16_t mod_data[262144] = {};
    int packet_count = 0;
    int corrupt_count = 0;

    if (argc != 7) {
        fprintf(stderr,
                "Error: Usage: %s [input.raw] [sample_rate] [baud_rate] [f_lo] "
                "[f_hi] [filter_width] (argc: %d)\n",
                argv[0], argc);
        return 1;
    }

    src_fd = open(argv[1], O_RDONLY);
    if (-1 == src_fd) {
        perror("Couldn't open input file");
        return 1;
    }

    uint32_t sample_rate = strtoul(argv[2], NULL, 0);
    uint32_t baud_rate = strtoul(argv[3], NULL, 0);
    uint32_t f_lo = strtoul(argv[4], NULL, 0);
    uint32_t f_hi = strtoul(argv[5], NULL, 0);
    uint32_t filter_width = strtoul(argv[6], NULL, 0);

    fsk_demod_generate_table(&demod_table, baud_rate, sample_rate, f_lo, f_hi,
                             filter_width);
    fsk_demod_init(&demod_table, &demod_state);

    demod_sample_t decoding_buffer[131072];
    int bytes_read;
    uint32_t total_bytes = 0;
    demod_pkt_t packet;
    bytes_read = read(src_fd, decoding_buffer, 44); // Read and discard WAV header

    while (1) {
        bytes_read = read(src_fd, decoding_buffer, sizeof(decoding_buffer));
        if (bytes_read == 0) {
            printf("EOF\n");
            break;
        }
        if (bytes_read == -1) {
            perror("Couldn't read from file");
            return 1;
        }
        total_bytes += bytes_read;

        int remaining_words = bytes_read / sizeof(*decoding_buffer);
        demod_sample_t *decoding_buffer_offset = decoding_buffer;
        while (remaining_words > 0) {
            int bit = 0;
            int result = fsk_demod(&demod_table, &demod_state, &bit,
                                   decoding_buffer_offset, remaining_words);
            if (result != -1) {
                if (mac_put_bit(&mac_state, bit, &packet, sizeof(packet))) {
                    if (validate_packet(&packet, 1)) {
                        packet_count++;
                    } else {
                        corrupt_count++;
                    }
                }
                decoding_buffer_offset += (remaining_words - result);
                remaining_words = result;
            }
        }
    }
    printf("Processed %d bytes\n", total_bytes);
    printf("Encountered %d packets (%d corrupt packets)\n", packet_count,
           corrupt_count);
    return packet_count;
}
#endif
