/* RUST-EXT-015g: destroying a content-loaded Model.
 *
 * The header is explicit that a loaded model owns the handles it publishes and
 * that they "are released when the model is destroyed", so `cna_model_destroy`
 * on a loaded model is the documented teardown. This measures it in C, with no
 * Rust in the process, so the result is evidence about CNA. */
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

static void on_segv(int sig, siginfo_t* info, void* context) {
    char line[128];
    const int n = snprintf(line, sizeof line,
                           "\n--- SIGSEGV at address %p ---\n", info->si_addr);
    write(2, line, (size_t)(n > 0 ? n : 0));
    void* frames[8];
    backtrace_symbols_fd(frames, backtrace(frames, 8), 2);
    _exit(139);
}

static const char* rs(CNA_Result r) {
    switch (r) {
        case CNA_RESULT_SUCCESS: return "SUCCESS";
        case CNA_RESULT_INVALID_ARGUMENT: return "INVALID_ARGUMENT";
        case CNA_RESULT_INVALID_HANDLE: return "INVALID_HANDLE";
        case CNA_RESULT_NOT_SUPPORTED: return "NOT_SUPPORTED";
        case CNA_RESULT_IO: return "IO";
        default: return "OTHER";
    }
}
#define SHOW(expr) do { CNA_Result _r = (expr); printf("%-58s -> %s\n", #expr, rs(_r)); fflush(stdout); } while (0)

int main(int argc, char** argv) {
    struct sigaction sa;
    memset(&sa, 0, sizeof sa);
    sa.sa_sigaction = on_segv;
    sa.sa_flags = SA_SIGINFO;
    sigaction(SIGSEGV, &sa, NULL);
    if (argc < 3) { printf("usage: %s <content-root> <asset-name>\n", argv[0]); return 2; }

    CNA_PresentationParameters pp;
    memset(&pp, 0, sizeof pp);
    pp.struct_size = (uint32_t)sizeof pp;
    pp.struct_version = 1u;
    pp.back_buffer_width = 64;
    pp.back_buffer_height = 64;
    pp.headless_ext = 1;

    CNA_Handle device = CNA_INVALID_HANDLE;
    CNA_Result dev = cna_graphics_device_create(0u, 0u, &pp, &device);
    printf("cna_graphics_device_create -> %s\n", rs(dev));
    if (dev != CNA_RESULT_SUCCESS) { printf("SKIP: no windowless device here\n"); return 0; }

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

    uint64_t bones = 0u;
    CNA_ModelBoneCollectionHandle collection = CNA_INVALID_HANDLE;
    SHOW(cna_model_get_bones(model, &collection));
    SHOW(cna_model_bone_collection_get_count(collection, &bones));
    printf("bones = %llu\n", (unsigned long long)bones);
    SHOW(cna_model_bone_collection_destroy(collection));

    /* Walk the parts first. If a part's canonical pointer were null, the
     * accessors would say so before the destructor does. */
    CNA_ModelMeshCollectionHandle meshes = CNA_INVALID_HANDLE;
    uint64_t meshCount = 0u;
    SHOW(cna_model_get_meshes(model, &meshes));
    SHOW(cna_model_mesh_collection_get_count(meshes, &meshCount));
    printf("meshes = %llu\n", (unsigned long long)meshCount);
    for (uint64_t m = 0u; m < meshCount; ++m) {
        CNA_ModelMeshHandle mesh = CNA_INVALID_HANDLE;
        SHOW(cna_model_mesh_collection_get_at(meshes, m, &mesh));
        CNA_ModelMeshPartCollectionHandle parts = CNA_INVALID_HANDLE;
        uint64_t partCount = 0u;
        SHOW(cna_model_mesh_get_mesh_parts(mesh, &parts));
        SHOW(cna_model_mesh_part_collection_get_count(parts, &partCount));
        printf("mesh %llu has %llu part(s)\n",
               (unsigned long long)m, (unsigned long long)partCount);
        for (uint64_t k = 0u; k < partCount; ++k) {
            CNA_ModelMeshPartHandle part = CNA_INVALID_HANDLE;
            SHOW(cna_model_mesh_part_collection_get_at(parts, k, &part));
            int32_t numVertices = -1;
            SHOW(cna_model_mesh_part_get_num_vertices(part, &numVertices));
            printf("  part %llu: num_vertices=%d\n", (unsigned long long)k, numVertices);
            SHOW(cna_model_mesh_part_destroy(part));
        }
        SHOW(cna_model_mesh_part_collection_destroy(parts));
        SHOW(cna_model_mesh_destroy(mesh));
    }
    SHOW(cna_model_mesh_collection_destroy(meshes));

    printf("\n-- the measurement: destroying a content-loaded model --\n");
    fflush(stdout);
    SHOW(cna_model_destroy(model));

    printf("\nprobe reached the end\n");
    return 0;
}
