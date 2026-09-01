/*
 * Probe: does CNA's sample arithmetic agree with the Rust projection's?
 *
 * `SoundEffect.GetSampleDuration` and `GetSampleSizeInBytes` are XNA statics,
 * and crates/cna/src/audio.rs reproduces them deliberately -- with a comment
 * about XNA's mixed binary32/double precision being observable at 44.1 kHz.
 * CNA has its own routes for the same two questions. If the answers agree
 * everywhere, restating them in Rust costs nothing and the C routes are a
 * deliberate non-binding; if they diverge, one of the two is wrong about XNA
 * and that is a finding. This prints CNA's answers so the Rust side can be
 * compared against them.
 */
#include <CNA/C/cna.h>
#include <stdio.h>

int main(void)
{
    static const int rates[] = {8000, 11025, 22050, 44100, 48000};
    static const int sizes[] = {0, 1, 2, 4, 100, 4096, 88200, 88198, 176400};
    static const CNA_AudioChannels channels[] = {
        CNA_AUDIO_CHANNELS_MONO, CNA_AUDIO_CHANNELS_STEREO};

    printf("size,rate,channels,duration_ticks\n");
    for (unsigned c = 0; c < 2u; ++c) {
        for (unsigned r = 0; r < 5u; ++r) {
            for (unsigned s = 0; s < 9u; ++s) {
                int64_t ticks = 0;
                CNA_Result result = cna_sound_effect_get_sample_duration_ticks(
                    sizes[s], rates[r], channels[c], &ticks);
                printf("%d,%d,%u,%s%lld\n", sizes[s], rates[r], (unsigned)channels[c],
                       result == CNA_RESULT_SUCCESS ? "" : "ERR:",
                       (long long)(result == CNA_RESULT_SUCCESS ? ticks : (int64_t)result));
            }
        }
    }

    printf("ticks,rate,channels,size_bytes\n");
    static const long long durations[] = {0, 10000, 10000000, 1000000, 5000000};
    for (unsigned c = 0; c < 2u; ++c) {
        for (unsigned r = 0; r < 5u; ++r) {
            for (unsigned d = 0; d < 5u; ++d) {
                int32_t size = 0;
                CNA_Result result = cna_sound_effect_get_sample_size_in_bytes(
                    (int64_t)durations[d], rates[r], channels[c], &size);
                printf("%lld,%d,%u,%s%d\n", durations[d], rates[r], (unsigned)channels[c],
                       result == CNA_RESULT_SUCCESS ? "" : "ERR:",
                       result == CNA_RESULT_SUCCESS ? size : (int)result);
            }
        }
    }
    return 0;
}
