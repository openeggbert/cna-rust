/* RUST-EXT-015g: what the models.h object graph actually does with ownership.
 *
 * The header says every navigation route hands back an "owned view". That
 * settles who calls destroy; it does not say whether a view keeps answering
 * once the model it views is gone, and that is the difference between a
 * plain Rust type and one that needs a lifetime parameter. Measured, not
 * assumed. */
#include <CNA/C/models.h>
#include <CNA/C/core.h>
#include <CNA/C/display.h>
#include <CNA/C/graphics_device.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
#define SHOW(expr) do { CNA_Result _r = (expr); printf("%-58s -> %s\n", #expr, rs(_r)); } while (0)

int main(void) {
    /* A model with one bone and no meshes, built by hand so the probe needs
     * no graphics device and no asset. */
    CNA_ModelBoneHandle bone = CNA_INVALID_HANDLE;
    CNA_StringView nm = { "root", 4u };
    SHOW(cna_model_bone_create(0, nm, &bone));

    /* cna_model_create refuses an invalid device even with no meshes, so the
     * probe needs a real one. Only HEADLESS makes a windowless device; the
     * GL renderers refuse for want of a platform surface, which is a renderer
     * property and not what this probe is measuring. */
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
    if (dev != CNA_RESULT_SUCCESS) {
        printf("SKIP: this renderer cannot make a windowless device\n");
        return 0;
    }

    CNA_ModelHandle model = CNA_INVALID_HANDLE;
    SHOW(cna_model_create(device, &bone, 1u, NULL, 0u, &model));

    printf("\n-- q1: is a collection handle distinct per call? --\n");
    CNA_ModelBoneCollectionHandle c1 = CNA_INVALID_HANDLE, c2 = CNA_INVALID_HANDLE;
    SHOW(cna_model_get_bones(model, &c1));
    SHOW(cna_model_get_bones(model, &c2));
    printf("c1 == c2: %s\n", (c1 == c2) ? "yes (shared)" : "no (fresh handle per call)");

    printf("\n-- q2: is a bone view distinct from the bone passed to create? --\n");
    CNA_ModelBoneHandle view = CNA_INVALID_HANDLE;
    SHOW(cna_model_bone_collection_get_at(c1, 0u, &view));
    printf("view == bone: %s\n", (view == bone) ? "yes (same handle)" : "no (fresh view handle)");

    printf("\n-- q3: does a view answer after its collection is destroyed? --\n");
    SHOW(cna_model_bone_collection_destroy(c2));
    int32_t idx = 12345;
    SHOW(cna_model_bone_get_index(view, &idx));
    printf("index = %lld\n", (long long)idx);

    printf("\n-- q4: does a view answer after the MODEL is destroyed? --\n");
    SHOW(cna_model_destroy(model));
    fflush(stdout);
    idx = 12345;
    CNA_Result after = cna_model_bone_get_index(view, &idx);
    printf("cna_model_bone_get_index(view) after model destroy -> %s, index=%lld\n",
           rs(after), (long long)idx);

    printf("\n-- q6: does the view still carry heap state (the name)? --\n");
    uint64_t nbytes = 0u;
    CNA_Result nb = cna_model_bone_get_name_byte_count(view, &nbytes);
    printf("name_byte_count after model destroy -> %s, bytes=%llu\n",
           rs(nb), (unsigned long long)nbytes);
    if (nb == CNA_RESULT_SUCCESS && nbytes > 0u) {
        char buf[64];
        uint64_t written = 0u;
        CNA_Result cp = cna_model_bone_copy_name(view, buf, sizeof buf, &written);
        buf[written < sizeof buf ? written : sizeof buf - 1] = 0;
        printf("copy_name -> %s, \"%s\"\n", rs(cp), buf);
    }

    printf("\n-- q5: can the surviving view still be destroyed? --\n");
    SHOW(cna_model_bone_collection_destroy(c1));
    SHOW(cna_model_bone_destroy(view));
    SHOW(cna_model_bone_destroy(bone));

    SHOW(cna_graphics_device_destroy(device));

    printf("\nprobe reached the end\n");
    return 0;
}
