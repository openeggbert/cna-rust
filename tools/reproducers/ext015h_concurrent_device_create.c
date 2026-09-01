/*
 * RUST-UPSTREAM-023 -- concurrent cna_graphics_device_create corrupts the heap.
 *
 * Several threads ask for a windowless GraphicsDevice at the same time. On a
 * renderer that cannot supply one (OPENGLES3 without a platform window) every
 * call is expected to fail with the same refusal, and nothing is expected to
 * be left allocated. Instead glibc aborts the process with "double free or
 * corruption" -- the create *failure* path is not thread-safe.
 *
 * Build (against an artifact's headers and library):
 *   cc -O0 -g -pthread ext015h_concurrent_device_create.c \
 *      -I<cnanext>/modules/c-api/include \
 *      -L<artifact>/modules/c-api -lcna_c_api -Wl,-rpath,<artifact>/modules/c-api \
 *      -o <build-probe>/ext015h_concurrent_device_create
 *
 * Expected: every thread reports the same non-success result, exit 0.
 * Actual on OPENGLES3: SIGABRT from glibc during the run or at teardown.
 */
#include <CNA/C/cna.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>

#define THREADS 6

static int failures[THREADS];
static int destroy_it = 1;
static int serialize_create = 0;
static int serialize_destroy = 0;
static pthread_mutex_t gate = PTHREAD_MUTEX_INITIALIZER;

static void* attempt(void* argument)
{
    long slot = (long)argument;
    CNA_PresentationParameters parameters;
    CNA_Handle device = CNA_INVALID_HANDLE;

    if (cna_presentation_parameters_init(&parameters) != CNA_RESULT_SUCCESS) {
        failures[slot] = -1;
        return NULL;
    }
    parameters.back_buffer_width = 64;
    parameters.back_buffer_height = 64;

    if (serialize_create) pthread_mutex_lock(&gate);
    failures[slot] = (int)cna_graphics_device_create(
        0u, CNA_GRAPHICS_PROFILE_HI_DEF, &parameters, &device);
    if (serialize_create) pthread_mutex_unlock(&gate);
    if (failures[slot] == CNA_RESULT_SUCCESS && destroy_it) {
        if (serialize_destroy) pthread_mutex_lock(&gate);
        cna_graphics_device_destroy(device);
        if (serialize_destroy) pthread_mutex_unlock(&gate);
    }
    return NULL;
}

int main(void)
{
    pthread_t threads[THREADS];
    long index;
    long count = THREADS;

    if (getenv("REPRO_NO_DESTROY")) destroy_it = 0;
    if (getenv("REPRO_SERIALIZE_CREATE")) serialize_create = 1;
    if (getenv("REPRO_SERIALIZE_DESTROY")) serialize_destroy = 1;
    if (getenv("REPRO_THREADS")) count = atol(getenv("REPRO_THREADS"));
    if (count < 1 || count > THREADS) count = THREADS;


    for (index = 0; index < count; ++index) {
        if (pthread_create(&threads[index], NULL, attempt, (void*)index) != 0) {
            fprintf(stderr, "pthread_create failed\n");
            return 2;
        }
    }
    for (index = 0; index < count; ++index) {
        pthread_join(threads[index], NULL);
    }

    for (index = 0; index < count; ++index) {
        printf("thread %ld -> %d\n", index, failures[index]);
    }
    printf("clean exit\n");
    return 0;
}
