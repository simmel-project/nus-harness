#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include "esplanade_demod.h"

#define PHASE_BITS 16
#define PHASE_BASE (1 << PHASE_BITS)

#define COS_TABLE_BITS 13
#define COS_TABLE_SIZE (1 << COS_TABLE_BITS)
#ifndef COS_BASE
#define COS_BASE (1 << 14)
#endif

uint16_t cos_tab[COS_TABLE_SIZE];

static inline int dsp_cos(int phase) {
    return cos_tab[(phase >> (PHASE_BITS - COS_TABLE_BITS)) &
                   (COS_TABLE_SIZE - 1)];
}

void FSK_mod_init(FSK_mod_state *s) {
    unsigned int b;

    for (b = 0; b < COS_TABLE_SIZE; b++) {
        cos_tab[b] = (int)(cos(2 * M_PI * b / COS_TABLE_SIZE) * COS_BASE);
    }

    s->omega[0] = (PHASE_BASE * s->f_lo) / s->sample_rate;
    s->omega[1] = (PHASE_BASE * s->f_hi) / s->sample_rate;
    s->baud_incr = (s->baud_rate * 0x10000) / s->sample_rate;
    s->phase = 0;
    s->baud_frac = 0;
    s->current_bit = 0;
}

void FSK_mod(FSK_mod_state *s, uint16_t *samples, unsigned int nb) {
    int phase, baud_frac, b;
    unsigned int i;

    phase = s->phase;
    baud_frac = s->baud_frac;
    b = s->current_bit;

    for (i = 0; i < nb; i++) {
        baud_frac += s->baud_incr;
        if (baud_frac >= 0x10000) {
            baud_frac -= 0x10000;
            b = s->get_bit(s->opaque);
        }
        samples[i] = dsp_cos(phase);
        phase += s->omega[b];
    }
    s->phase = phase;
    s->baud_frac = baud_frac;
    s->current_bit = b;
}
