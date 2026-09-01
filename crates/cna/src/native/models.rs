//! The `Model` object graph an XNA game loads from content and draws.
//!
//! Separate from [`super::engine`] because these are not engine-layer routes.
//! The engine layer is a build-time choice and its routes answer
//! `NOT_SUPPORTED` when it is compiled out; every route here answers for real
//! against a `CNA_CNAEXT=OFF` library, which is how the ownership probe behind
//! this module was run.
//!
//! # What the handles here own
//!
//! Every navigation route hands back an owned handle, including the ones the
//! header calls views. Three facts decide the safe layer's shape, and all
//! three were measured (`tools/reproducers/ext015g_model_ownership.c`) rather than
//! read off the prose:
//!
//! * `cna_model_get_bones` answers a *fresh* collection handle per call, so
//!   each one has to be destroyed.
//! * A bone view is a different handle from the bone that was passed to
//!   `cna_model_create`; the model retained its own reference.
//! * A view keeps answering -- index, name and all -- after
//!   `cna_model_destroy`. The views are independently counted, not aliases
//!   into the model's storage.
//!
//! That last one is why `ModelBone`, `ModelMesh` and `ModelMeshPart` in the
//! safe layer carry no lifetime parameter. The part *contents* are the
//! exception: an effect or a buffer handle is retained by the part rather than
//! owned by the caller, and a content-loaded model's are documented as invalid
//! past `cna_model_destroy`, so those stay borrowed views.

use cna_sys as sys;

use crate::error::Result;

use super::loader::NativeSource;

/// Every reviewed `models.h` route, resolved once when the tables are filled.
#[derive(Debug)]
pub(crate) struct ModelsApi {
    pub(crate) content_manager_load_model: sys::cna_content_manager_load_model_fn,
    pub(crate) model_add_camera_ext: sys::cna_model_add_camera_ext_fn,
    pub(crate) model_add_gltf_import_diagnostic_ext: sys::cna_model_add_gltf_import_diagnostic_ext_fn,
    pub(crate) model_add_skin_ext: sys::cna_model_add_skin_ext_fn,
    pub(crate) model_apply_bind_pose_bone_transforms_ext: sys::cna_model_apply_bind_pose_bone_transforms_ext_fn,
    pub(crate) model_apply_clip_to_bones_ext: sys::cna_model_apply_clip_to_bones_ext_fn,
    pub(crate) model_bone_collection_contains: sys::cna_model_bone_collection_contains_fn,
    pub(crate) model_bone_collection_destroy: sys::cna_model_bone_collection_destroy_fn,
    pub(crate) model_bone_collection_find: sys::cna_model_bone_collection_find_fn,
    pub(crate) model_bone_collection_get_at: sys::cna_model_bone_collection_get_at_fn,
    pub(crate) model_bone_collection_get_count: sys::cna_model_bone_collection_get_count_fn,
    pub(crate) model_bone_copy_name: sys::cna_model_bone_copy_name_fn,
    pub(crate) model_bone_destroy: sys::cna_model_bone_destroy_fn,
    pub(crate) model_bone_get_children: sys::cna_model_bone_get_children_fn,
    pub(crate) model_bone_get_index: sys::cna_model_bone_get_index_fn,
    pub(crate) model_bone_get_name_byte_count: sys::cna_model_bone_get_name_byte_count_fn,
    pub(crate) model_bone_get_parent: sys::cna_model_bone_get_parent_fn,
    pub(crate) model_bone_get_transform: sys::cna_model_bone_get_transform_fn,
    pub(crate) model_bone_set_transform: sys::cna_model_bone_set_transform_fn,
    pub(crate) model_clear_cameras_ext: sys::cna_model_clear_cameras_ext_fn,
    pub(crate) model_clear_skins_ext: sys::cna_model_clear_skins_ext_fn,
    pub(crate) model_copy_absolute_bone_transforms: sys::cna_model_copy_absolute_bone_transforms_fn,
    pub(crate) model_copy_bone_transforms: sys::cna_model_copy_bone_transforms_fn,
    pub(crate) model_copy_camera_name_ext: sys::cna_model_copy_camera_name_ext_fn,
    pub(crate) model_copy_gltf_import_diagnostic_code_ext: sys::cna_model_copy_gltf_import_diagnostic_code_ext_fn,
    pub(crate) model_copy_gltf_import_diagnostic_detail_ext: sys::cna_model_copy_gltf_import_diagnostic_detail_ext_fn,
    pub(crate) model_copy_gltf_import_diagnostic_message_ext: sys::cna_model_copy_gltf_import_diagnostic_message_ext_fn,
    pub(crate) model_copy_gltf_import_diagnostic_subject_ext: sys::cna_model_copy_gltf_import_diagnostic_subject_ext_fn,
    pub(crate) model_copy_material_variant_name_ext: sys::cna_model_copy_material_variant_name_ext_fn,
    pub(crate) model_copy_skin_name_ext: sys::cna_model_copy_skin_name_ext_fn,
    pub(crate) model_create_skin_skeleton_handle_ext: sys::cna_model_create_skin_skeleton_handle_ext_fn,
    pub(crate) model_destroy: sys::cna_model_destroy_fn,
    pub(crate) model_get_content_tag_dictionary_ext: sys::cna_model_get_content_tag_dictionary_ext_fn,
    pub(crate) model_get_content_tag_foreign_object_ext: sys::cna_model_get_content_tag_foreign_object_ext_fn,
    pub(crate) model_draw: sys::cna_model_draw_fn,
    pub(crate) model_effect_collection_contains: sys::cna_model_effect_collection_contains_fn,
    pub(crate) model_effect_collection_destroy: sys::cna_model_effect_collection_destroy_fn,
    pub(crate) model_effect_collection_get_at: sys::cna_model_effect_collection_get_at_fn,
    pub(crate) model_effect_collection_get_count: sys::cna_model_effect_collection_get_count_fn,
    pub(crate) model_get_bone_transform_count: sys::cna_model_get_bone_transform_count_fn,
    pub(crate) model_get_bones: sys::cna_model_get_bones_fn,
    pub(crate) model_get_bounding_sphere_ext: sys::cna_model_get_bounding_sphere_ext_fn,
    pub(crate) model_get_camera_count_ext: sys::cna_model_get_camera_count_ext_fn,
    pub(crate) model_get_camera_ext: sys::cna_model_get_camera_ext_fn,
    pub(crate) model_get_camera_name_byte_count_ext: sys::cna_model_get_camera_name_byte_count_ext_fn,
    pub(crate) model_get_gltf_import_diagnostic_code_byte_count_ext: sys::cna_model_get_gltf_import_diagnostic_code_byte_count_ext_fn,
    pub(crate) model_get_gltf_import_diagnostic_detail_byte_count_ext: sys::cna_model_get_gltf_import_diagnostic_detail_byte_count_ext_fn,
    pub(crate) model_get_gltf_import_diagnostic_ext: sys::cna_model_get_gltf_import_diagnostic_ext_fn,
    pub(crate) model_get_gltf_import_diagnostic_message_byte_count_ext: sys::cna_model_get_gltf_import_diagnostic_message_byte_count_ext_fn,
    pub(crate) model_get_gltf_import_diagnostic_subject_byte_count_ext: sys::cna_model_get_gltf_import_diagnostic_subject_byte_count_ext_fn,
    pub(crate) model_get_gltf_import_report_ext: sys::cna_model_get_gltf_import_report_ext_fn,
    pub(crate) model_get_material_variant_count_ext: sys::cna_model_get_material_variant_count_ext_fn,
    pub(crate) model_get_material_variant_ext: sys::cna_model_get_material_variant_ext_fn,
    pub(crate) model_get_material_variant_name_byte_count_ext: sys::cna_model_get_material_variant_name_byte_count_ext_fn,
    pub(crate) model_get_meshes: sys::cna_model_get_meshes_fn,
    pub(crate) model_get_root: sys::cna_model_get_root_fn,
    pub(crate) model_get_skin_count_ext: sys::cna_model_get_skin_count_ext_fn,
    pub(crate) model_get_skin_ext: sys::cna_model_get_skin_ext_fn,
    pub(crate) model_get_skin_mesh_index_ext: sys::cna_model_get_skin_mesh_index_ext_fn,
    pub(crate) model_get_skin_name_byte_count_ext: sys::cna_model_get_skin_name_byte_count_ext_fn,
    pub(crate) model_mesh_collection_contains: sys::cna_model_mesh_collection_contains_fn,
    pub(crate) model_mesh_collection_destroy: sys::cna_model_mesh_collection_destroy_fn,
    pub(crate) model_mesh_collection_find: sys::cna_model_mesh_collection_find_fn,
    pub(crate) model_mesh_collection_get_at: sys::cna_model_mesh_collection_get_at_fn,
    pub(crate) model_mesh_collection_get_count: sys::cna_model_mesh_collection_get_count_fn,
    pub(crate) model_mesh_copy_name: sys::cna_model_mesh_copy_name_fn,
    pub(crate) model_mesh_destroy: sys::cna_model_mesh_destroy_fn,
    pub(crate) model_mesh_draw: sys::cna_model_mesh_draw_fn,
    pub(crate) model_mesh_get_bounding_sphere: sys::cna_model_mesh_get_bounding_sphere_fn,
    pub(crate) model_mesh_get_effects: sys::cna_model_mesh_get_effects_fn,
    pub(crate) model_mesh_get_mesh_parts: sys::cna_model_mesh_get_mesh_parts_fn,
    pub(crate) model_mesh_get_name_byte_count: sys::cna_model_mesh_get_name_byte_count_fn,
    pub(crate) model_mesh_get_parent_bone: sys::cna_model_mesh_get_parent_bone_fn,
    pub(crate) model_mesh_part_collection_destroy: sys::cna_model_mesh_part_collection_destroy_fn,
    pub(crate) model_mesh_part_collection_get_at: sys::cna_model_mesh_part_collection_get_at_fn,
    pub(crate) model_mesh_part_collection_get_count: sys::cna_model_mesh_part_collection_get_count_fn,
    pub(crate) model_mesh_part_get_effect: sys::cna_model_mesh_part_get_effect_fn,
    pub(crate) model_mesh_part_get_index_buffer: sys::cna_model_mesh_part_get_index_buffer_fn,
    pub(crate) model_mesh_part_get_vertex_buffer: sys::cna_model_mesh_part_get_vertex_buffer_fn,
    pub(crate) model_set_bone_transforms: sys::cna_model_set_bone_transforms_fn,
    pub(crate) model_set_gltf_import_report_ext: sys::cna_model_set_gltf_import_report_ext_fn,
    pub(crate) model_set_material_variant_ext: sys::cna_model_set_material_variant_ext_fn,
}

impl ModelsApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            content_manager_load_model: symbol!(cna_content_manager_load_model, _),
            model_add_camera_ext: symbol!(cna_model_add_camera_ext, _),
            model_add_gltf_import_diagnostic_ext: symbol!(cna_model_add_gltf_import_diagnostic_ext, _),
            model_add_skin_ext: symbol!(cna_model_add_skin_ext, _),
            model_apply_bind_pose_bone_transforms_ext: symbol!(cna_model_apply_bind_pose_bone_transforms_ext, _),
            model_apply_clip_to_bones_ext: symbol!(cna_model_apply_clip_to_bones_ext, _),
            model_bone_collection_contains: symbol!(cna_model_bone_collection_contains, _),
            model_bone_collection_destroy: symbol!(cna_model_bone_collection_destroy, _),
            model_bone_collection_find: symbol!(cna_model_bone_collection_find, _),
            model_bone_collection_get_at: symbol!(cna_model_bone_collection_get_at, _),
            model_bone_collection_get_count: symbol!(cna_model_bone_collection_get_count, _),
            model_bone_copy_name: symbol!(cna_model_bone_copy_name, _),
            model_bone_destroy: symbol!(cna_model_bone_destroy, _),
            model_bone_get_children: symbol!(cna_model_bone_get_children, _),
            model_bone_get_index: symbol!(cna_model_bone_get_index, _),
            model_bone_get_name_byte_count: symbol!(cna_model_bone_get_name_byte_count, _),
            model_bone_get_parent: symbol!(cna_model_bone_get_parent, _),
            model_bone_get_transform: symbol!(cna_model_bone_get_transform, _),
            model_bone_set_transform: symbol!(cna_model_bone_set_transform, _),
            model_clear_cameras_ext: symbol!(cna_model_clear_cameras_ext, _),
            model_clear_skins_ext: symbol!(cna_model_clear_skins_ext, _),
            model_copy_absolute_bone_transforms: symbol!(cna_model_copy_absolute_bone_transforms, _),
            model_copy_bone_transforms: symbol!(cna_model_copy_bone_transforms, _),
            model_copy_camera_name_ext: symbol!(cna_model_copy_camera_name_ext, _),
            model_copy_gltf_import_diagnostic_code_ext: symbol!(cna_model_copy_gltf_import_diagnostic_code_ext, _),
            model_copy_gltf_import_diagnostic_detail_ext: symbol!(cna_model_copy_gltf_import_diagnostic_detail_ext, _),
            model_copy_gltf_import_diagnostic_message_ext: symbol!(cna_model_copy_gltf_import_diagnostic_message_ext, _),
            model_copy_gltf_import_diagnostic_subject_ext: symbol!(cna_model_copy_gltf_import_diagnostic_subject_ext, _),
            model_copy_material_variant_name_ext: symbol!(cna_model_copy_material_variant_name_ext, _),
            model_copy_skin_name_ext: symbol!(cna_model_copy_skin_name_ext, _),
            model_create_skin_skeleton_handle_ext: symbol!(cna_model_create_skin_skeleton_handle_ext, _),
            model_destroy: symbol!(cna_model_destroy, _),
            model_get_content_tag_dictionary_ext: symbol!(cna_model_get_content_tag_dictionary_ext, _),
            model_get_content_tag_foreign_object_ext: symbol!(cna_model_get_content_tag_foreign_object_ext, _),
            model_draw: symbol!(cna_model_draw, _),
            model_effect_collection_contains: symbol!(cna_model_effect_collection_contains, _),
            model_effect_collection_destroy: symbol!(cna_model_effect_collection_destroy, _),
            model_effect_collection_get_at: symbol!(cna_model_effect_collection_get_at, _),
            model_effect_collection_get_count: symbol!(cna_model_effect_collection_get_count, _),
            model_get_bone_transform_count: symbol!(cna_model_get_bone_transform_count, _),
            model_get_bones: symbol!(cna_model_get_bones, _),
            model_get_bounding_sphere_ext: symbol!(cna_model_get_bounding_sphere_ext, _),
            model_get_camera_count_ext: symbol!(cna_model_get_camera_count_ext, _),
            model_get_camera_ext: symbol!(cna_model_get_camera_ext, _),
            model_get_camera_name_byte_count_ext: symbol!(cna_model_get_camera_name_byte_count_ext, _),
            model_get_gltf_import_diagnostic_code_byte_count_ext: symbol!(cna_model_get_gltf_import_diagnostic_code_byte_count_ext, _),
            model_get_gltf_import_diagnostic_detail_byte_count_ext: symbol!(cna_model_get_gltf_import_diagnostic_detail_byte_count_ext, _),
            model_get_gltf_import_diagnostic_ext: symbol!(cna_model_get_gltf_import_diagnostic_ext, _),
            model_get_gltf_import_diagnostic_message_byte_count_ext: symbol!(cna_model_get_gltf_import_diagnostic_message_byte_count_ext, _),
            model_get_gltf_import_diagnostic_subject_byte_count_ext: symbol!(cna_model_get_gltf_import_diagnostic_subject_byte_count_ext, _),
            model_get_gltf_import_report_ext: symbol!(cna_model_get_gltf_import_report_ext, _),
            model_get_material_variant_count_ext: symbol!(cna_model_get_material_variant_count_ext, _),
            model_get_material_variant_ext: symbol!(cna_model_get_material_variant_ext, _),
            model_get_material_variant_name_byte_count_ext: symbol!(cna_model_get_material_variant_name_byte_count_ext, _),
            model_get_meshes: symbol!(cna_model_get_meshes, _),
            model_get_root: symbol!(cna_model_get_root, _),
            model_get_skin_count_ext: symbol!(cna_model_get_skin_count_ext, _),
            model_get_skin_ext: symbol!(cna_model_get_skin_ext, _),
            model_get_skin_mesh_index_ext: symbol!(cna_model_get_skin_mesh_index_ext, _),
            model_get_skin_name_byte_count_ext: symbol!(cna_model_get_skin_name_byte_count_ext, _),
            model_mesh_collection_contains: symbol!(cna_model_mesh_collection_contains, _),
            model_mesh_collection_destroy: symbol!(cna_model_mesh_collection_destroy, _),
            model_mesh_collection_find: symbol!(cna_model_mesh_collection_find, _),
            model_mesh_collection_get_at: symbol!(cna_model_mesh_collection_get_at, _),
            model_mesh_collection_get_count: symbol!(cna_model_mesh_collection_get_count, _),
            model_mesh_copy_name: symbol!(cna_model_mesh_copy_name, _),
            model_mesh_destroy: symbol!(cna_model_mesh_destroy, _),
            model_mesh_draw: symbol!(cna_model_mesh_draw, _),
            model_mesh_get_bounding_sphere: symbol!(cna_model_mesh_get_bounding_sphere, _),
            model_mesh_get_effects: symbol!(cna_model_mesh_get_effects, _),
            model_mesh_get_mesh_parts: symbol!(cna_model_mesh_get_mesh_parts, _),
            model_mesh_get_name_byte_count: symbol!(cna_model_mesh_get_name_byte_count, _),
            model_mesh_get_parent_bone: symbol!(cna_model_mesh_get_parent_bone, _),
            model_mesh_part_collection_destroy: symbol!(cna_model_mesh_part_collection_destroy, _),
            model_mesh_part_collection_get_at: symbol!(cna_model_mesh_part_collection_get_at, _),
            model_mesh_part_collection_get_count: symbol!(cna_model_mesh_part_collection_get_count, _),
            model_mesh_part_get_effect: symbol!(cna_model_mesh_part_get_effect, _),
            model_mesh_part_get_index_buffer: symbol!(cna_model_mesh_part_get_index_buffer, _),
            model_mesh_part_get_vertex_buffer: symbol!(cna_model_mesh_part_get_vertex_buffer, _),
            model_set_bone_transforms: symbol!(cna_model_set_bone_transforms, _),
            model_set_gltf_import_report_ext: symbol!(cna_model_set_gltf_import_report_ext, _),
            model_set_material_variant_ext: symbol!(cna_model_set_material_variant_ext, _),
        })
    }
}
