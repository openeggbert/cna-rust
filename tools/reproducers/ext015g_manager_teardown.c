/* RUST-EXT-015g: does destroying the *manager* hit the same teardown?
 *
 * The model is never destroyed here. If the content cache releases its own
 * loaded models through the same PartResource path, this faults too, and the
 * finding is wider than `cna_model_destroy`. */
#include <CNA/C/models.h>
#include <CNA/C/core.h>
#include <CNA/C/content.h>
#include <CNA/C/display.h>
#include <CNA/C/graphics_device.h>
#include <stdio.h>
#include <string.h>
#include <execinfo.h>
#include <signal.h>
#include <unistd.h>
#include <stdlib.h>

static void on_segv(int sig, siginfo_t* info, void* context) {
    char line[128];
    const int n = snprintf(line, sizeof line, "\n--- SIGSEGV at %p ---\n", info->si_addr);
    write(2, line, (size_t)(n > 0 ? n : 0));
    void* frames[12];
    backtrace_symbols_fd(frames, backtrace(frames, 12), 2);
    _exit(139);
}

static const char* rs(CNA_Result r) {
    return r == CNA_RESULT_SUCCESS ? "SUCCESS" : "NOT-SUCCESS";
}
#define SHOW(expr) do { CNA_Result _r = (expr); printf("%-58s -> %s\n", #expr, rs(_r)); fflush(stdout); } while (0)

int main(int argc, char** argv) {
    if (argc < 3) { return 2; }
    struct sigaction sa;
    memset(&sa, 0, sizeof sa);
    sa.sa_sigaction = on_segv;
    sa.sa_flags = SA_SIGINFO;
    sigaction(SIGSEGV, &sa, NULL);

    CNA_PresentationParameters pp;
    memset(&pp, 0, sizeof pp);
    pp.struct_size = (uint32_t)sizeof pp;
    pp.struct_version = 1u;
    pp.back_buffer_width = 64; pp.back_buffer_height = 64; pp.headless_ext = 1;

    CNA_Handle device = CNA_INVALID_HANDLE;
    if (cna_graphics_device_create(0u, 0u, &pp, &device) != CNA_RESULT_SUCCESS) {
        printf("SKIP\n"); return 0;
    }
    CNA_ContentManagerCreateInfo info;
    memset(&info, 0, sizeof info);
    info.struct_size = (uint32_t)sizeof info;
    info.struct_version = 1u;
    info.root_directory.data = argv[1];
    info.root_directory.byte_length = strlen(argv[1]);

    CNA_Handle manager = CNA_INVALID_HANDLE;
    SHOW(cna_content_manager_create(device, &info, &manager));
    CNA_StringView name = { argv[2], strlen(argv[2]) };
    CNA_ModelHandle model = CNA_INVALID_HANDLE;
    SHOW(cna_content_manager_load_model(manager, name, &model));

    printf("\n-- destroying the manager, leaking the model handle --\n");
    fflush(stdout);
    SHOW(cna_content_manager_destroy(manager));
    printf("manager destroyed\n");
    SHOW(cna_graphics_device_destroy(device));
    printf("\nprobe reached the end\n");
    return 0;
}
