#ifndef __ORCHARD_DEMOD__
#define __ORCHARD_DEMOD__

typedef int (*get_bit_func)(void *opaque);

#ifndef FSK_FILTER_MAX_SIZE
#define FSK_FILTER_MAX_SIZE 256
#endif

#ifndef FSK_FILTER_BUF_MAX
#define FSK_FILTER_BUF_MAX 256
#endif

typedef struct {
    /* parameters */
    int f_lo, f_hi;
    int sample_rate;
    int baud_rate;

    /* local variables */
    int phase, baud_frac, baud_incr;
    int omega[2];
    int current_bit;
    void *opaque;
    get_bit_func get_bit;
} FSK_mod_state;

typedef int16_t demod_sample_t;

typedef struct {
    demod_sample_t filter_buf[FSK_FILTER_BUF_MAX];
    uint32_t buf_offset;

    /// The position inside of the current bit. Range (0,1). A new bit is
    /// complete when this exceeds 1.0.
    float baud_pll;

    /// The amount to increment the baud for each sample. This is dependent on
    /// the sample rate and baud rate.
    float baud_incr;

    /// How much to nudge the PLL by when we encounter a bit transition
    float baud_pll_adj;

    /// The last bit, 0 or 1
    int last_sample;

    int16_t shift;
    uint32_t run_length;
    int transition_count;
    int last_bit;
} FSK_demod_state;

typedef struct {
    /* parameters */
    uint32_t f_lo, f_hi;
    uint32_t sample_rate;
    float baud_rate;
    uint32_t filter_buf_size;

    /* local variables */
    int32_t filter_size;
    float filter_lo_i[FSK_FILTER_MAX_SIZE];
    float filter_lo_q[FSK_FILTER_MAX_SIZE];

    float filter_hi_i[FSK_FILTER_MAX_SIZE];
    float filter_hi_q[FSK_FILTER_MAX_SIZE];
} FSK_demod_const;

/// Generate filter table constants.  This only needs to be done
/// once per set of parameters.
void fsk_demod_generate_table(FSK_demod_const *fsk_table, float baud_rate,
                              uint32_t sample_rate, uint32_t f_lo,
                              uint32_t f_hi, uint32_t gen_filter_tab);
void fsk_demod_init(const FSK_demod_const *fsk_table,
                    FSK_demod_state *fsk_state);
int fsk_demod(const FSK_demod_const *fsk_table, FSK_demod_state *fsk_state,
              int *bit, demod_sample_t *samples, uint32_t nb);

void FSK_mod_init(FSK_mod_state *s);
void FSK_mod(FSK_mod_state *s, uint16_t *samples, unsigned int nb);

#endif
