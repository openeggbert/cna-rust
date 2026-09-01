/*
 * Probe: is a GraphicsResource's name CNA's or the caller's?
 *
 * The Rust projection keeps `Name` and `Tag` in a Rust-side Mutex and never
 * tells CNA. This asks what CNA thinks a freshly created texture is called,
 * whether set_name round-trips, and what ToString answers before and after --
 * which decides whether the twelve graphics_resource.h routes are worth
 * binding or whether the Rust state is the whole truth.
 */
#include <CNA/C/cna.h>
#include <stdio.h>
#include <string.h>

static CNA_StringView view(const char* s)
{
    CNA_StringView v;
    v.data = s;
    v.byte_length = (uint64_t)strlen(s);
    return v;
}

static void show_name(const char* label, CNA_Handle resource)
{
    char buffer[256];
    uint64_t count = 0U;
    CNA_Result r = cna_graphics_resource_copy_name(resource, buffer, sizeof buffer, &count);
    buffer[count < sizeof buffer ? count : sizeof buffer - 1] = '\0';
    printf("%-28s copy_name -> %d, %llu bytes, \"%s\"\n",
           label, (int)r, (unsigned long long)count, buffer);
}

static void show_string(const char* label, CNA_Handle resource)
{
    char buffer[256];
    uint64_t count = 0U;
    CNA_Result r = cna_graphics_resource_copy_string(resource, buffer, sizeof buffer, &count);
    buffer[count < sizeof buffer ? count : sizeof buffer - 1] = '\0';
    printf("%-28s copy_string -> %d, \"%s\"\n", label, (int)r, buffer);
}

int main(void)
{
    CNA_PresentationParameters pp;
    CNA_Handle device = CNA_INVALID_HANDLE;
    CNA_Handle texture = CNA_INVALID_HANDLE;
    CNA_Handle back = CNA_INVALID_HANDLE;
    CNA_Bool disposed = 0;
    CNA_Handle tag = CNA_INVALID_HANDLE;

    if (cna_presentation_parameters_init(&pp) != CNA_RESULT_SUCCESS) { return 2; }
    pp.back_buffer_width = 64; pp.back_buffer_height = 64;
    CNA_Result dev = cna_graphics_device_create(0u, CNA_GRAPHICS_PROFILE_HI_DEF, &pp, &device);
    printf("device create -> %d\n", (int)dev);
    if (dev != CNA_RESULT_SUCCESS) { return 1; }

    CNA_Texture2DCreateInfo info;
    memset(&info, 0, sizeof info);
    info.struct_size = (uint32_t)sizeof info;
    info.struct_version = 1U;
    info.width = 4U;
    info.height = 4U;
    info.mip_map = CNA_FALSE;
    info.format = CNA_SURFACE_FORMAT_COLOR;
    CNA_Result t = cna_texture2d_create(device, &info, &texture);
    printf("texture create -> %d\n", (int)t);
    if (t != CNA_RESULT_SUCCESS) { return 1; }

    show_name("fresh texture", texture);
    show_string("fresh texture", texture);

    printf("set_name -> %d\n",
           (int)cna_graphics_resource_set_name(texture, view("hero diffuse")));
    show_name("after set_name", texture);
    show_string("after set_name", texture);

    CNA_Result r = cna_graphics_resource_get_is_disposed(texture, &disposed);
    printf("get_is_disposed -> %d, disposed=%d\n", (int)r, (int)disposed);
    r = cna_graphics_resource_get_graphics_device(texture, &back);
    printf("get_graphics_device -> %d, same=%d\n", (int)r, (int)(back == device));
    r = cna_graphics_resource_get_tag(texture, &tag);
    printf("get_tag -> %d, tag=%llu\n", (int)r, (unsigned long long)tag);

    printf("dispose -> %d\n", (int)cna_graphics_resource_dispose(texture));
    r = cna_graphics_resource_get_is_disposed(texture, &disposed);
    printf("get_is_disposed after -> %d, disposed=%d\n", (int)r, (int)disposed);
    show_name("after dispose", texture);

    cna_texture2d_destroy(texture);
    cna_graphics_device_destroy(device);
    printf("clean exit\n");
    return 0;
}
