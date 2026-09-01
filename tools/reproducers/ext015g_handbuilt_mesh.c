/* RUST-EXT-015g: the control for the loaded-model destroy fault.
 *
 * Same teardown, but on a mesh built by hand. If the hypothesis is right --
 * that `~MeshResource` moves an empty `detachedValue` into `value` for a
 * content-loaded part, and `~PartResource` then dereferences it unguarded --
 * this must NOT crash, because a hand-built part has a real `detachedValue`. */
#include <CNA/C/models.h>
#include <CNA/C/core.h>
#include <CNA/C/display.h>
#include <CNA/C/graphics_device.h>
#include <stdio.h>
#include <string.h>

static const char* rs(CNA_Result r) {
    switch (r) {
        case CNA_RESULT_SUCCESS: return "SUCCESS";
        case CNA_RESULT_INVALID_ARGUMENT: return "INVALID_ARGUMENT";
        case CNA_RESULT_INVALID_HANDLE: return "INVALID_HANDLE";
        case CNA_RESULT_NOT_SUPPORTED: return "NOT_SUPPORTED";
        default: return "OTHER";
    }
}
#define SHOW(expr) do { CNA_Result _r = (expr); printf("%-58s -> %s\n", #expr, rs(_r)); fflush(stdout); } while (0)

int main(void) {
    CNA_PresentationParameters pp;
    memset(&pp, 0, sizeof pp);
    pp.struct_size = (uint32_t)sizeof pp;
    pp.struct_version = 1u;
    pp.back_buffer_width = 64;
    pp.back_buffer_height = 64;
    pp.headless_ext = 1;

    CNA_Handle device = CNA_INVALID_HANDLE;
    if (cna_graphics_device_create(0u, 0u, &pp, &device) != CNA_RESULT_SUCCESS) {
        printf("SKIP: no windowless device here\n");
        return 0;
    }

    CNA_ModelMeshPartHandle part = CNA_INVALID_HANDLE;
    SHOW(cna_model_mesh_part_create_default(&part));

    CNA_StringView meshName = { "HandBuilt", 9u };
    CNA_ModelMeshHandle mesh = CNA_INVALID_HANDLE;
    SHOW(cna_model_mesh_create_named(device, meshName, &part, 1u, &mesh));

    CNA_ModelBoneHandle bone = CNA_INVALID_HANDLE;
    CNA_StringView boneName = { "Root", 4u };
    SHOW(cna_model_bone_create(0, boneName, &bone));

    CNA_ModelHandle model = CNA_INVALID_HANDLE;
    SHOW(cna_model_create(device, &bone, 1u, &mesh, 1u, &model));

    printf("\n-- the control: destroying a hand-built model with one part --\n");
    fflush(stdout);
    SHOW(cna_model_destroy(model));

    SHOW(cna_model_mesh_destroy(mesh));
    SHOW(cna_model_mesh_part_destroy(part));
    SHOW(cna_model_bone_destroy(bone));
    SHOW(cna_graphics_device_destroy(device));
    printf("\nprobe reached the end\n");
    return 0;
}
