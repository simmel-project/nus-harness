/* code to be run on the local computer to generate C files
   that contain constant tables for inclusion into the demodulator core */

#include <stdio.h>
#include <math.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>

#include "esplanade_demod.h"

#ifndef M_PI
#define M_PI 3.14159265358979323846f
#endif

void fsk_demod_generate_table(FSK_demod_const *fsk_table, float baud_rate,
                              uint32_t sample_rate, uint32_t f_lo,
                              uint32_t f_hi, uint32_t fsk_filter_size) {
    float phase;
    int i;

    assert(fsk_filter_size < FSK_FILTER_MAX_SIZE);
    fsk_table->f_lo = f_lo;
    fsk_table->f_hi = f_hi;
    fsk_table->baud_rate = baud_rate;
    fsk_table->sample_rate = sample_rate;
    fsk_table->filter_buf_size = fsk_filter_size * 2;
    fsk_table->filter_size = fsk_filter_size;

    /* compute the filters */
    // fsk_table->filter_lo_i = malloc(sizeof(float) * fsk_filter_size);
    // fsk_table->filter_lo_q = malloc(sizeof(float) * fsk_filter_size);
    // fsk_table->filter_hi_i = malloc(sizeof(float) * fsk_filter_size);
    // fsk_table->filter_hi_q = malloc(sizeof(float) * fsk_filter_size);
    for (i = 0; i < fsk_table->filter_size; i++) {
        phase = 2 * M_PI * fsk_table->f_lo * i / (float)fsk_table->sample_rate;
        fsk_table->filter_lo_i[i] = cosf(phase);
        fsk_table->filter_lo_q[i] = sinf(phase);

        phase = 2 * M_PI * fsk_table->f_hi * i / (float)fsk_table->sample_rate;
        fsk_table->filter_hi_i[i] = cosf(phase);
        fsk_table->filter_hi_q[i] = sinf(phase);
    }
}
