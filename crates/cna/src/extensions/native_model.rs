//! The model CNA's own content pipeline produces, and the glTF facts it carries.
//!
//! # Why this exists beside [`crate::graphics::Model`]
//!
//! CNA-Rust already has a complete XNA `Model`: [`crate::graphics::Model`] is a
//! Rust object graph with bones, meshes, parts, collections and enumerators,
//! built by this crate's own `.xnb` `ModelReader`. Nothing here replaces it,
//! and none of the C routes that *construct* a model by hand are bound --
//! reaching for a `CNA_ModelHandle` to assemble bones and meshes would be a
//! second, worse spelling of a type the crate already has.
//!
//! What the Rust reader cannot produce is the other half of CNA's content
//! story. CNA's pipeline imports `.gltf` and `.glb`, and its runtime loads the
//! `.cnj` assets that import writes; this crate reads neither. Everything a
//! glTF import knows and an `.xnb` cannot express -- the import report and its
//! diagnostics, the cameras the scene declared, the skins, the material
//! variants -- lives only on a model CNA loaded. That is the gap this module
//! closes, and it is the whole reason a [`NativeModel`] is worth having.
//!
//! So: load an `.xnb` through the strict `ContentManager` and get a
//! [`crate::graphics::Model`]. Load a CNA-pipeline asset through
//! [`NativeModel::load`] and get the imported scene's own facts.
//!
//! # Ownership
//!
//! Every navigation route in `models.h` hands back an owned handle, including
//! the ones the header calls views, and the handles are independently counted
//! rather than aliases into the model. Measured with
//! `tools/reproducers/ext015g_model_ownership.c`:
//!
//! * `cna_model_get_bones` answers a *fresh* collection handle per call.
//! * A bone view is a different handle from the bone the model was built with.
//! * A view keeps answering -- index, name and all -- after
//!   `cna_model_destroy`.
//!
//! That is why [`ModelBoneView`] and [`ModelMeshView`] carry no lifetime
//! parameter: each owns a handle it releases exactly once, and outliving the
//! model is safe. The exception is what a *part* holds. An effect or a buffer
//! is retained by the part rather than owned by the caller, and upstream
//! documents a content-loaded model's as invalid past `cna_model_destroy`, so
//! those are reported as presence and identity rather than handed out as
//! handles this module would have to guess the lifetime of.

#![allow(clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::content::NativeContentManager;
use crate::extensions::models::{ModelAnimations, SkinningData};
use crate::extensions::object_dictionary::ObjectDictionary;
use crate::native::Native;
use crate::value::{BoundingSphere, Matrix, Vector3};

/// How severe one glTF import outcome is.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum GltfImportSeverity {
    /// A note about what the importer did; the result matches the source.
    #[default]
    Information,
    /// The imported model differs observably from the source asset.
    Warning,
}

impl GltfImportSeverity {
    const fn from_native(value: sys::CNA_GltfImportDiagnosticSeverityEXT) -> Option<Self> {
        match value {
            sys::CNA_GLTF_IMPORT_SEVERITY_INFORMATION_EXT => Some(Self::Information),
            sys::CNA_GLTF_IMPORT_SEVERITY_WARNING_EXT => Some(Self::Warning),
            _ => None,
        }
    }

    const fn to_native(self) -> sys::CNA_GltfImportDiagnosticSeverityEXT {
        match self {
            Self::Information => sys::CNA_GLTF_IMPORT_SEVERITY_INFORMATION_EXT,
            Self::Warning => sys::CNA_GLTF_IMPORT_SEVERITY_WARNING_EXT,
        }
    }
}

/// What one glTF import outcome did to the data.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum GltfImportKind {
    /// Something worth recording that changed nothing.
    #[default]
    Information,
    /// The importer generated data the source did not carry.
    GeneratedData,
    /// The source data was invalid and was repaired or ignored.
    InvalidSourceData,
    /// The result approximates the source rather than reproducing it.
    Approximation,
    /// Source data was dropped.
    DroppedData,
    /// A source feature CNA does not implement.
    UnsupportedFeature,
}

impl GltfImportKind {
    const fn from_native(value: sys::CNA_GltfImportDiagnosticKindEXT) -> Option<Self> {
        match value {
            sys::CNA_GLTF_IMPORT_KIND_INFORMATION_EXT => Some(Self::Information),
            sys::CNA_GLTF_IMPORT_KIND_GENERATED_DATA_EXT => Some(Self::GeneratedData),
            sys::CNA_GLTF_IMPORT_KIND_INVALID_SOURCE_DATA_EXT => Some(Self::InvalidSourceData),
            sys::CNA_GLTF_IMPORT_KIND_APPROXIMATION_EXT => Some(Self::Approximation),
            sys::CNA_GLTF_IMPORT_KIND_DROPPED_DATA_EXT => Some(Self::DroppedData),
            sys::CNA_GLTF_IMPORT_KIND_UNSUPPORTED_FEATURE_EXT => Some(Self::UnsupportedFeature),
            _ => None,
        }
    }

    const fn to_native(self) -> sys::CNA_GltfImportDiagnosticKindEXT {
        match self {
            Self::Information => sys::CNA_GLTF_IMPORT_KIND_INFORMATION_EXT,
            Self::GeneratedData => sys::CNA_GLTF_IMPORT_KIND_GENERATED_DATA_EXT,
            Self::InvalidSourceData => sys::CNA_GLTF_IMPORT_KIND_INVALID_SOURCE_DATA_EXT,
            Self::Approximation => sys::CNA_GLTF_IMPORT_KIND_APPROXIMATION_EXT,
            Self::DroppedData => sys::CNA_GLTF_IMPORT_KIND_DROPPED_DATA_EXT,
            Self::UnsupportedFeature => sys::CNA_GLTF_IMPORT_KIND_UNSUPPORTED_FEATURE_EXT,
        }
    }
}

/// What importing a glTF scene produced, in counts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct GltfImportReport {
    /// Nodes imported from the represented source scene.
    pub node_count: u64,
    /// Mesh placements imported from source nodes.
    pub mesh_instance_count: u64,
    /// Distinct source meshes those placements reference.
    pub distinct_mesh_count: u64,
    /// Distinct source meshes referenced by more than one placement.
    pub shared_mesh_count: u64,
    /// Longest imported root-to-leaf node chain.
    pub max_node_depth: u64,
    /// Imported scene nodes referencing a camera.
    pub camera_node_count: u64,
    /// Imported scene nodes referencing a punctual light.
    pub light_node_count: u64,
    /// Punctual lights that reached a CNA effect light slot.
    pub imported_light_count: u64,
    /// Source primitives this model represents, excluding material variants.
    pub primitive_count: u64,
    /// Independent skins this model represents.
    pub skin_count: u64,
    /// Source animations inspected during the import.
    pub animation_count: u64,
    /// Animation clips the model actually retained.
    pub clip_count: u64,
    /// Outcomes available by index; only the ones that occurred.
    pub diagnostic_count: u64,
    /// Warning entries, not the sum of their occurrence counts.
    pub warning_count: u64,
    /// Dropped-data and unsupported-feature occurrences, summed.
    pub dropped_feature_count: u64,
    /// Approximation occurrences, summed.
    pub approximation_count: u64,
    /// Whether any warning is present, so the result may differ from the source.
    pub anything_lost: bool,
}

/// One programmatically reachable outcome of importing a glTF asset.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct GltfImportDiagnostic {
    /// Stable lower-case, hyphen-separated identifier.
    pub code: String,
    /// Whether this is a note or an observable fidelity warning.
    pub severity: GltfImportSeverity,
    /// What the outcome did to the data.
    pub kind: GltfImportKind,
    /// The primitive, node, clip or extension this concerns; may be empty.
    pub subject: String,
    /// Human-readable explanation for a log or a diagnostics overlay.
    pub message: String,
    /// Individually affected names, such as texture maps.
    pub details: Vec<String>,
    /// How many occurrences this entry represents.
    pub count: u64,
    /// Largest measured magnitude, or 0 when none applies.
    pub worst_magnitude: f64,
}

/// A camera an imported scene declared.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ModelCamera {
    /// The source camera's display name; may be empty.
    pub name: String,
    /// Index of the scene node carrying it, or `None` when unknown.
    pub scene_node_index: Option<i32>,
    /// Whether the projection is perspective rather than orthographic.
    pub is_perspective: bool,
    /// Whether the perspective projection has no far plane.
    pub has_infinite_far_plane: bool,
    /// Whether the source declared an aspect ratio of its own.
    pub has_authored_aspect_ratio: bool,
    /// The projection matrix as imported.
    pub projection: Matrix,
    /// The camera's world transform as imported.
    pub world_transform: Matrix,
    /// Aspect ratio; 1 when the source declared none.
    pub aspect_ratio: f32,
    /// Vertical field of view in radians; 0 for an orthographic camera.
    pub field_of_view: f32,
    /// Near plane distance.
    pub near_plane_distance: f32,
    /// Far plane distance; meaningless when the far plane is infinite.
    pub far_plane_distance: f32,
}

/// One skin an imported scene declared, and the meshes it poses.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ModelSkin {
    /// The skin's display name; may be empty.
    pub name: String,
    /// Whether the skin names a skeleton at all.
    pub has_skeleton: bool,
    /// Indices into the model's own mesh collection.
    pub mesh_indices: Vec<u64>,
}

/// A model CNA's content pipeline loaded.
///
/// `OWNED`: it holds a `CNA_ModelHandle` it releases exactly once. See the
/// module documentation for why this is not [`crate::graphics::Model`] and what
/// it is for.
///
/// # Loading one faults the process on teardown
///
/// Every route below answers correctly. The **teardown** does not: CNA faults
/// while destroying a content-loaded model that has at least one mesh part,
/// and it faults again at process exit for a model that was merely leaked.
/// Both were measured; `RUST-UPSTREAM-021` in `docs/upstream-findings.md` has
/// the mechanism, the two probes and the hand-built control that isolates it.
///
/// There is no ordering on this side that avoids it, so nothing here tries.
/// Until CNA is fixed, a process that loads a model with a mesh part will fault
/// before it ends -- which is why this crate's own tests for the type run it in
/// a **child process** and read the results back, rather than loading a model
/// in a test binary that has other work to do.
///
/// A model with no mesh part is unaffected, and so is every hand-built model
/// -- but hand-building one is not what this type is for, and
/// [`crate::graphics::Model`] is the type to reach for anyway.
pub struct NativeModel {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_ModelHandle>,
}

impl NativeModel {
    /// Loads a model through CNA's own content manager.
    ///
    /// The asset name carries no extension, exactly as XNA's
    /// `ContentManager.Load` takes none. Which file that resolves to is CNA's
    /// decision, and it is what makes a `.cnj` asset -- the shape CNA's glTF
    /// import writes -- reachable from Rust at all.
    ///
    /// The asset is cached by name, so loading the same name twice publishes a
    /// second handle over one underlying model rather than re-reading the file.
    pub fn load(content_manager: &NativeContentManager, asset_name: &str) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is borrowed for the call, the name is
        // borrowed and copied by CNA, and the output is a live local.
        native.check(unsafe {
            (native.models.content_manager_load_model)(
                content_manager.handle(),
                string_view(asset_name),
                &mut handle,
            )
        })?;
        Ok(Self {
            native,
            handle: Mutex::new(handle),
        })
    }

    fn get(&self) -> Result<sys::CNA_ModelHandle> {
        handle_of(&self.handle, "the model has been released")
    }

    /// The live handle, for the one other extension that takes a CNA model.
    pub(crate) fn native_handle(&self) -> Result<sys::CNA_ModelHandle> {
        self.get()
    }

    /// Releases the model.
    ///
    /// # This model's teardown faults, and not releasing does not avoid it
    ///
    /// `cna_model_destroy` faults on a content-loaded model that has at least
    /// one mesh part -- `SIGSEGV`, `RUST-UPSTREAM-021` in
    /// `docs/upstream-findings.md`. `~MeshResource` moves an empty
    /// `detachedValue` over a loaded part's `value`, and `~PartResource`
    /// dereferences it two lines later without the null check its own next line
    /// applies to `detachedValue`.
    ///
    /// Leaking the handle was measured and does **not** help: the C API's
    /// handle registry runs the same destructor when the process exits, with
    /// the same stack and the same faulting address. There is no order of
    /// operations on this side that avoids it, which is why this calls the
    /// route rather than guarding it. A guard would only move a fault from a
    /// place a caller can see to one they cannot.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once;
        // the slot is cleared first so a later call cannot repeat it.
        self.native
            .check(unsafe { (self.native.models.model_destroy)(handle) })
    }

    /// Draws every mesh after applying the three transforms.
    pub fn draw(&self, world: Matrix, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the matrices are passed by value.
        self.native.check(unsafe {
            (self.native.models.model_draw)(
                handle,
                matrix(world),
                matrix(view),
                matrix(projection),
            )
        })
    }

    /// The sphere containing every mesh's bounding sphere, or `None` when the
    /// model has no mesh at all.
    pub fn bounding_sphere(&self) -> Result<Option<BoundingSphere>> {
        let handle = self.get()?;
        let mut has_value = 0_u8;
        let mut sphere = sys::CNA_BoundingSphere::default();
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_get_bounding_sphere_ext)(handle, &mut has_value, &mut sphere)
        })?;
        Ok((has_value != 0).then(|| bounding_sphere(sphere)))
    }
}

/// The bone hierarchy and the poses that ride on it.
impl NativeModel {
    /// Every bone, in the model's own order.
    pub fn bones(&self) -> Result<Vec<ModelBoneView>> {
        let collection = self.bone_collection()?;
        let count = collection.count()?;
        (0..count).map(|index| collection.get_at(index)).collect()
    }

    /// How many bones the model has.
    pub fn bone_count(&self) -> Result<u64> {
        self.bone_collection()?.count()
    }

    /// The bone of that name, or `None` when the model has no such bone.
    ///
    /// XNA's `ModelBoneCollection.TryGetValue`.
    pub fn bone_named(&self, name: &str) -> Result<Option<ModelBoneView>> {
        self.bone_collection()?.find(name)
    }

    /// Whether the model's bone collection contains this exact bone.
    ///
    /// Every navigation route answers a fresh handle, so a bone taken from one
    /// call is a different handle from the same bone taken by another. This
    /// asks upstream whether the two name the same bone, which is not a
    /// question handle equality can answer.
    pub fn contains_bone(&self, bone: &ModelBoneView) -> Result<bool> {
        let collection = self.bone_collection()?;
        let mut contains = 0_u8;
        // SAFETY: both handles are owned by live values and the output is a
        // live local.
        self.native.check(unsafe {
            (self.native.models.model_bone_collection_contains)(
                collection.handle,
                bone.handle,
                &mut contains,
            )
        })?;
        Ok(contains != 0)
    }

    /// The root bone, or `None` for a model with no bones.
    pub fn root_bone(&self) -> Result<Option<ModelBoneView>> {
        let handle = self.get()?;
        let mut has_root = 0_u8;
        let mut bone = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_get_root)(handle, &mut has_root, &mut bone)
        })?;
        Ok((has_root != 0).then(|| ModelBoneView::new(&self.native, bone)))
    }

    fn bone_collection(&self) -> Result<BoneCollection> {
        let handle = self.get()?;
        let mut collection = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned collection handle.
        self.native
            .check(unsafe { (self.native.models.model_get_bones)(handle, &mut collection) })?;
        Ok(BoneCollection {
            native: Arc::clone(&self.native),
            handle: collection,
        })
    }

    /// How many matrices the bulk transform routes read and write.
    pub fn bone_transform_count(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_get_bone_transform_count)(handle, &mut count)
        })?;
        Ok(count)
    }

    /// Each bone's transform composed with its parents'.
    pub fn absolute_bone_transforms(&self) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        self.read_transforms(|destination, capacity, out_count| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable matrices.
            unsafe {
                (self.native.models.model_copy_absolute_bone_transforms)(
                    handle,
                    destination,
                    capacity,
                    out_count,
                )
            }
        })
    }

    /// Each bone's own local transform.
    pub fn bone_transforms(&self) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        self.read_transforms(|destination, capacity, out_count| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable matrices.
            unsafe {
                (self.native.models.model_copy_bone_transforms)(
                    handle,
                    destination,
                    capacity,
                    out_count,
                )
            }
        })
    }

    /// Writes one local transform into every bone.
    ///
    /// The slice must cover every bone; a short one is refused rather than
    /// applied in part, which is what makes this atomic on the CNA side too.
    pub fn set_bone_transforms(&self, transforms: &[Matrix]) -> Result<()> {
        let handle = self.get()?;
        let native: Vec<sys::CNA_Matrix> = transforms.iter().copied().map(matrix).collect();
        // SAFETY: the handle is owned and the array is borrowed for the call
        // with the count it was sized against.
        self.native.check(unsafe {
            (self.native.models.model_set_bone_transforms)(
                handle,
                native.as_ptr(),
                native.len() as u64,
            )
        })
    }

    /// Poses the bones from a skeleton's bind pose, answering how many moved.
    pub fn apply_bind_pose_bone_transforms(&self, data: &SkinningData) -> Result<u64> {
        let handle = self.get()?;
        let mut posed = 0_u64;
        // SAFETY: both handles are owned by live values and the output is a
        // live local.
        self.native.check(unsafe {
            (self.native.models.model_apply_bind_pose_bone_transforms_ext)(
                handle,
                data.handle()?,
                &mut posed,
            )
        })?;
        Ok(posed)
    }

    /// Poses the bones from one scene-node clip at a point in time.
    ///
    /// Each track's bone index selects a bone directly, so the clip must be in
    /// scene-node space. A joint-palette clip is refused rather than applied:
    /// its indices address a skinning palette, and using them here would pose
    /// the wrong bones without saying so.
    pub fn apply_clip_to_bones(
        &self,
        animations: &ModelAnimations,
        clip_index: u64,
        time_seconds: f64,
    ) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: both handles are owned by live values.
        self.native.check(unsafe {
            (self.native.models.model_apply_clip_to_bones_ext)(
                handle,
                animations.handle()?,
                clip_index,
                time_seconds,
            )
        })
    }

    /// CNA's count-then-copy protocol for the matrix arrays.
    ///
    /// The count comes from `cna_model_get_bone_transform_count` rather than
    /// from the copy route reporting its own requirement: asking a copy route
    /// with a zero capacity is refused as a range error rather than answered,
    /// which the count route exists to avoid.
    fn read_transforms(
        &self,
        mut route: impl FnMut(*mut sys::CNA_Matrix, u64, *mut u64) -> sys::CNA_Result,
    ) -> Result<Vec<Matrix>> {
        let required = self.bone_transform_count()?;
        if required == 0 {
            return Ok(Vec::new());
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("more bone transforms than fit in memory"))?;
        let mut buffer = vec![sys::CNA_Matrix::default(); capacity];
        let mut written = 0_u64;
        self.native
            .check(route(buffer.as_mut_ptr(), required, &mut written))?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer.into_iter().map(value_matrix).collect())
    }
}

/// The meshes, their parts and the effects on them.
impl NativeModel {
    /// Every mesh, in the model's own order.
    pub fn meshes(&self) -> Result<Vec<ModelMeshView>> {
        let collection = self.mesh_collection()?;
        let count = collection.count()?;
        (0..count).map(|index| collection.get_at(index)).collect()
    }

    /// How many meshes the model has.
    pub fn mesh_count(&self) -> Result<u64> {
        self.mesh_collection()?.count()
    }

    /// The mesh of that name, or `None` when the model has no such mesh.
    pub fn mesh_named(&self, name: &str) -> Result<Option<ModelMeshView>> {
        self.mesh_collection()?.find(name)
    }

    /// Whether the model's mesh collection contains this exact mesh.
    ///
    /// The same handle-identity caveat as [`Self::contains_bone`].
    pub fn contains_mesh(&self, mesh: &ModelMeshView) -> Result<bool> {
        let collection = self.mesh_collection()?;
        let mut contains = 0_u8;
        // SAFETY: both handles are owned by live values and the output is a
        // live local.
        self.native.check(unsafe {
            (self.native.models.model_mesh_collection_contains)(
                collection.handle,
                mesh.handle,
                &mut contains,
            )
        })?;
        Ok(contains != 0)
    }

    fn mesh_collection(&self) -> Result<MeshCollection> {
        let handle = self.get()?;
        let mut collection = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned collection handle.
        self.native
            .check(unsafe { (self.native.models.model_get_meshes)(handle, &mut collection) })?;
        Ok(MeshCollection {
            native: Arc::clone(&self.native),
            handle: collection,
        })
    }
}

/// The glTF import report and its diagnostics.
impl NativeModel {
    /// What importing the source scene produced.
    ///
    /// A model that came from anywhere but a glTF import answers a report of
    /// zeroes rather than failing, because "nothing was imported" is the true
    /// answer for such a model and not an error.
    pub fn gltf_import_report(&self) -> Result<GltfImportReport> {
        let handle = self.get()?;
        let mut report = sys::CNA_GltfImportReportEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfImportReportEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_GltfImportReportEXT::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set, as the route requires.
        self.native.check(unsafe {
            (self.native.models.model_get_gltf_import_report_ext)(handle, &mut report)
        })?;
        Ok(GltfImportReport {
            node_count: report.node_count,
            mesh_instance_count: report.mesh_instance_count,
            distinct_mesh_count: report.distinct_mesh_count,
            shared_mesh_count: report.shared_mesh_count,
            max_node_depth: report.max_node_depth,
            camera_node_count: report.camera_node_count,
            light_node_count: report.light_node_count,
            imported_light_count: report.imported_light_count,
            primitive_count: report.primitive_count,
            skin_count: report.skin_count,
            animation_count: report.animation_count,
            clip_count: report.clip_count,
            diagnostic_count: report.diagnostic_count,
            warning_count: report.warning_count,
            dropped_feature_count: report.dropped_feature_count,
            approximation_count: report.approximation_count,
            anything_lost: report.anything_lost != 0,
        })
    }

    /// Replaces the counts, for a caller running its own importer.
    ///
    /// The diagnostic list is not part of this; it is appended entry by entry
    /// with [`Self::add_gltf_import_diagnostic`], because each entry carries
    /// four independent strings and a fixed structure cannot hold an array of
    /// those without a second level of borrowed pointers.
    pub fn set_gltf_import_report(&self, report: &GltfImportReport) -> Result<()> {
        let handle = self.get()?;
        let native = sys::CNA_GltfImportReportEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfImportReportEXT>() as u32,
            struct_version: 1,
            node_count: report.node_count,
            mesh_instance_count: report.mesh_instance_count,
            distinct_mesh_count: report.distinct_mesh_count,
            shared_mesh_count: report.shared_mesh_count,
            max_node_depth: report.max_node_depth,
            camera_node_count: report.camera_node_count,
            light_node_count: report.light_node_count,
            imported_light_count: report.imported_light_count,
            primitive_count: report.primitive_count,
            skin_count: report.skin_count,
            animation_count: report.animation_count,
            clip_count: report.clip_count,
            diagnostic_count: report.diagnostic_count,
            warning_count: report.warning_count,
            dropped_feature_count: report.dropped_feature_count,
            approximation_count: report.approximation_count,
            anything_lost: u8::from(report.anything_lost),
        };
        // SAFETY: the handle is owned and the report is borrowed for the call.
        self.native.check(unsafe {
            (self.native.models.model_set_gltf_import_report_ext)(handle, &native)
        })
    }

    /// Every import outcome, in the order the importer recorded them.
    pub fn gltf_import_diagnostics(&self) -> Result<Vec<GltfImportDiagnostic>> {
        let count = self.gltf_import_report()?.diagnostic_count;
        (0..count)
            .map(|index| self.gltf_import_diagnostic_at(index))
            .collect()
    }

    /// One import outcome by index.
    pub fn gltf_import_diagnostic_at(&self, index: u64) -> Result<GltfImportDiagnostic> {
        let handle = self.get()?;
        let mut entry = sys::CNA_GltfImportDiagnosticEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfImportDiagnosticEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_GltfImportDiagnosticEXT::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native.check(unsafe {
            (self.native.models.model_get_gltf_import_diagnostic_ext)(handle, index, &mut entry)
        })?;

        let severity = GltfImportSeverity::from_native(entry.severity).ok_or(
            CnaError::InvalidInput("CNA reported an unknown glTF diagnostic severity"),
        )?;
        let kind = GltfImportKind::from_native(entry.kind).ok_or(CnaError::InvalidInput(
            "CNA reported an unknown glTF diagnostic kind",
        ))?;

        let models = &self.native.models;
        let details = (0..entry.detail_count)
            .map(|detail| {
                self.diagnostic_text(
                    |out| {
                        // SAFETY: owned handle, live output.
                        unsafe {
                            (models.model_get_gltf_import_diagnostic_detail_byte_count_ext)(
                                handle, index, detail, out,
                            )
                        }
                    },
                    |destination, capacity, written| {
                        // SAFETY: owned handle, `capacity` writable bytes.
                        unsafe {
                            (models.model_copy_gltf_import_diagnostic_detail_ext)(
                                handle,
                                index,
                                detail,
                                destination,
                                capacity,
                                written,
                            )
                        }
                    },
                )
            })
            .collect::<Result<Vec<String>>>()?;

        Ok(GltfImportDiagnostic {
            code: self.diagnostic_text(
                // SAFETY: owned handle, live output.
                |out| unsafe {
                    (models.model_get_gltf_import_diagnostic_code_byte_count_ext)(
                        handle, index, out,
                    )
                },
                // SAFETY: owned handle, `capacity` writable bytes.
                |destination, capacity, written| unsafe {
                    (models.model_copy_gltf_import_diagnostic_code_ext)(
                        handle,
                        index,
                        destination,
                        capacity,
                        written,
                    )
                },
            )?,
            subject: self.diagnostic_text(
                // SAFETY: owned handle, live output.
                |out| unsafe {
                    (models.model_get_gltf_import_diagnostic_subject_byte_count_ext)(
                        handle, index, out,
                    )
                },
                // SAFETY: owned handle, `capacity` writable bytes.
                |destination, capacity, written| unsafe {
                    (models.model_copy_gltf_import_diagnostic_subject_ext)(
                        handle,
                        index,
                        destination,
                        capacity,
                        written,
                    )
                },
            )?,
            message: self.diagnostic_text(
                // SAFETY: owned handle, live output.
                |out| unsafe {
                    (models.model_get_gltf_import_diagnostic_message_byte_count_ext)(
                        handle, index, out,
                    )
                },
                // SAFETY: owned handle, `capacity` writable bytes.
                |destination, capacity, written| unsafe {
                    (models.model_copy_gltf_import_diagnostic_message_ext)(
                        handle,
                        index,
                        destination,
                        capacity,
                        written,
                    )
                },
            )?,
            severity,
            kind,
            details,
            count: entry.count,
            worst_magnitude: entry.worst_magnitude,
        })
    }

    /// Appends one import outcome, for a caller running its own importer.
    pub fn add_gltf_import_diagnostic(&self, diagnostic: &GltfImportDiagnostic) -> Result<()> {
        let handle = self.get()?;
        // The detail views borrow `diagnostic.details`, so the vector has to
        // outlive the call rather than be a temporary inside the descriptor.
        let details: Vec<sys::CNA_StringView> = diagnostic
            .details
            .iter()
            .map(|detail| string_view(detail))
            .collect();
        let descriptor = sys::CNA_GltfImportDiagnosticDescriptorEXT {
            code: string_view(&diagnostic.code),
            severity: diagnostic.severity.to_native(),
            kind: diagnostic.kind.to_native(),
            subject: string_view(&diagnostic.subject),
            count: diagnostic.count,
            worst_magnitude: diagnostic.worst_magnitude,
            details: if details.is_empty() {
                core::ptr::null()
            } else {
                details.as_ptr()
            },
            detail_count: details.len() as u64,
            message: string_view(&diagnostic.message),
        };
        // SAFETY: the handle is owned, and every string and the detail array
        // the descriptor points at outlives the call.
        self.native.check(unsafe {
            (self.native.models.model_add_gltf_import_diagnostic_ext)(handle, &descriptor)
        })
    }

    fn diagnostic_text(
        &self,
        mut size: impl FnMut(*mut u64) -> sys::CNA_Result,
        copy: impl FnMut(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
    ) -> Result<String> {
        let mut required = 0_u64;
        self.native.check(size(&mut required))?;
        read_text(required, copy)
    }
}

/// The cameras, skins and material variants an import brought over.
impl NativeModel {
    /// How many imported cameras the model carries.
    pub fn camera_count(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.models.model_get_camera_count_ext)(handle, &mut count) })?;
        Ok(count)
    }

    /// Every imported camera, in source order.
    pub fn cameras(&self) -> Result<Vec<ModelCamera>> {
        let count = self.camera_count()?;
        (0..count).map(|index| self.camera_at(index)).collect()
    }

    /// One imported camera by index.
    pub fn camera_at(&self, index: u64) -> Result<ModelCamera> {
        let handle = self.get()?;
        let mut camera = sys::CNA_ModelCameraEXT {
            struct_size: core::mem::size_of::<sys::CNA_ModelCameraEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_ModelCameraEXT::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native.check(unsafe {
            (self.native.models.model_get_camera_ext)(handle, index, &mut camera)
        })?;
        let models = &self.native.models;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (models.model_get_camera_name_byte_count_ext)(handle, index, &mut required)
        })?;
        let name = read_text(required, |destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (models.model_copy_camera_name_ext)(handle, index, destination, capacity, written)
            }
        })?;
        Ok(ModelCamera {
            name,
            // The header spells "unknown" as -1 rather than as a flag.
            scene_node_index: (camera.scene_node_index >= 0).then_some(camera.scene_node_index),
            is_perspective: camera.is_perspective != 0,
            has_infinite_far_plane: camera.has_infinite_far_plane != 0,
            has_authored_aspect_ratio: camera.has_authored_aspect_ratio != 0,
            projection: value_matrix(camera.projection),
            world_transform: value_matrix(camera.world_transform),
            aspect_ratio: camera.aspect_ratio,
            field_of_view: camera.field_of_view,
            near_plane_distance: camera.near_plane_distance,
            far_plane_distance: camera.far_plane_distance,
        })
    }

    /// Appends one camera, for a caller running its own importer.
    pub fn add_camera(&self, camera: &ModelCamera) -> Result<()> {
        let handle = self.get()?;
        let descriptor = sys::CNA_ModelCameraDescriptorEXT {
            name: string_view(&camera.name),
            camera: sys::CNA_ModelCameraEXT {
                struct_size: core::mem::size_of::<sys::CNA_ModelCameraEXT>() as u32,
                struct_version: 1,
                scene_node_index: camera.scene_node_index.unwrap_or(-1),
                is_perspective: u8::from(camera.is_perspective),
                has_infinite_far_plane: u8::from(camera.has_infinite_far_plane),
                has_authored_aspect_ratio: u8::from(camera.has_authored_aspect_ratio),
                projection: matrix(camera.projection),
                world_transform: matrix(camera.world_transform),
                aspect_ratio: camera.aspect_ratio,
                field_of_view: camera.field_of_view,
                near_plane_distance: camera.near_plane_distance,
                far_plane_distance: camera.far_plane_distance,
            },
        };
        // SAFETY: the handle is owned and the descriptor, including the name it
        // borrows, outlives the call.
        self.native
            .check(unsafe { (self.native.models.model_add_camera_ext)(handle, &descriptor) })
    }

    /// Removes every imported camera.
    pub fn clear_cameras(&self) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.models.model_clear_cameras_ext)(handle) })
    }

    /// How many independent skins the model carries.
    pub fn skin_count(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.models.model_get_skin_count_ext)(handle, &mut count) })?;
        Ok(count)
    }

    /// Every skin, in source order.
    pub fn skins(&self) -> Result<Vec<ModelSkin>> {
        let count = self.skin_count()?;
        (0..count).map(|index| self.skin_at(index)).collect()
    }

    /// One skin by index, with the meshes it poses.
    pub fn skin_at(&self, index: u64) -> Result<ModelSkin> {
        let handle = self.get()?;
        let models = &self.native.models;
        let mut has_skeleton = 0_u8;
        let mut mesh_count = 0_u64;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (models.model_get_skin_ext)(handle, index, &mut has_skeleton, &mut mesh_count)
        })?;

        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (models.model_get_skin_name_byte_count_ext)(handle, index, &mut required)
        })?;
        let name = read_text(required, |destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (models.model_copy_skin_name_ext)(handle, index, destination, capacity, written)
            }
        })?;

        let mesh_indices = (0..mesh_count)
            .map(|mesh| {
                let mut model_mesh_index = 0_u64;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe {
                    (models.model_get_skin_mesh_index_ext)(
                        handle,
                        index,
                        mesh,
                        &mut model_mesh_index,
                    )
                })?;
                Ok(model_mesh_index)
            })
            .collect::<Result<Vec<u64>>>()?;

        Ok(ModelSkin {
            name,
            has_skeleton: has_skeleton != 0,
            mesh_indices,
        })
    }

    /// A fresh owned handle on one skin's skeleton.
    ///
    /// `None` when the skin names no skeleton. This is deliberately a *new*
    /// handle rather than the one that was added: the model keeps the skeleton
    /// alive for as long as the skin exists, so releasing either never
    /// releases the other's object.
    ///
    /// # Only for a skin this side added
    ///
    /// Upstream answers `INVALID_STATE` -- "The Model skin's skeleton was not
    /// created through the C API" -- for a skin the *content loader* built, so
    /// a glTF import's own skeleton is not reachable through this route. The
    /// error is passed through rather than folded into `None`: "there is no
    /// skeleton" and "the skeleton exists and cannot be reached" are different
    /// facts, and [`ModelSkin::has_skeleton`] already reports the first.
    /// Recorded as `RUST-UPSTREAM-022` in `docs/upstream-findings.md`.
    pub fn skin_skeleton(&self, index: u64) -> Result<Option<SkinningData>> {
        let handle = self.get()?;
        if !self.skin_at(index)?.has_skeleton {
            return Ok(None);
        }
        let mut data = sys::CNA_INVALID_HANDLE;
        // Past this point a refusal is upstream's answer, not a missing
        // skeleton, so it propagates.
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned handle.
        self.native.check(unsafe {
            (self.native.models.model_create_skin_skeleton_handle_ext)(handle, index, &mut data)
        })?;
        Ok(Some(SkinningData::from_owned_handle(&self.native, data)))
    }

    /// Appends one skin, naming the meshes it poses by model mesh index.
    ///
    /// The skeleton is retained for as long as the skin exists, so the caller's
    /// [`SkinningData`] may be dropped afterwards. Pass `None` for a skin with
    /// no skeleton.
    pub fn add_skin(
        &self,
        name: &str,
        skeleton: Option<&SkinningData>,
        mesh_indices: &[u64],
    ) -> Result<()> {
        let handle = self.get()?;
        let data = match skeleton {
            Some(data) => data.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: both handles are owned by live values, and the index array is
        // borrowed for the call with the count it was sized against.
        self.native.check(unsafe {
            (self.native.models.model_add_skin_ext)(
                handle,
                string_view(name),
                data,
                mesh_indices.as_ptr(),
                mesh_indices.len() as u64,
            )
        })
    }

    /// Removes every skin and releases the skeletons they retained.
    pub fn clear_skins(&self) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.models.model_clear_skins_ext)(handle) })
    }

    /// The material-variant names the imported asset declared, in source order.
    pub fn material_variants(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let models = &self.native.models;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (models.model_get_material_variant_count_ext)(handle, &mut count) })?;
        (0..count)
            .map(|index| {
                let mut required = 0_u64;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe {
                    (models.model_get_material_variant_name_byte_count_ext)(
                        handle,
                        index,
                        &mut required,
                    )
                })?;
                read_text(required, |destination, capacity, written| {
                    // SAFETY: the handle is owned and the destination holds
                    // `capacity` writable bytes.
                    unsafe {
                        (models.model_copy_material_variant_name_ext)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    }
                })
            })
            .collect()
    }

    /// Which material variant is selected, or `None` for the default materials.
    pub fn material_variant(&self) -> Result<Option<u64>> {
        let handle = self.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_get_material_variant_ext)(handle, &mut value)
        })?;
        // Upstream spells "no variant" as -1; Rust spells it `None`.
        Ok(u64::try_from(value).ok())
    }

    /// Selects a material variant, or restores the default materials with
    /// `None`.
    pub fn set_material_variant(&self, index: Option<u64>) -> Result<()> {
        let handle = self.get()?;
        let value = match index {
            None => -1_i32,
            Some(index) => i32::try_from(index)
                .map_err(|_| CnaError::InvalidInput("material variant index is out of range"))?,
        };
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.models.model_set_material_variant_ext)(handle, value) })
    }
}

impl Drop for NativeModel {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// One bone of a loaded model.
///
/// Owns its own handle. A bone view outlives the model it came from -- measured,
/// not assumed -- so there is no lifetime here to get wrong.
pub struct ModelBoneView {
    native: Arc<Native>,
    handle: sys::CNA_ModelBoneHandle,
}

impl ModelBoneView {
    fn new(native: &Arc<Native>, handle: sys::CNA_ModelBoneHandle) -> Self {
        Self {
            native: Arc::clone(native),
            handle,
        }
    }

    /// The bone's index in the model's bone collection.
    pub fn index(&self) -> Result<i32> {
        let mut index = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.models.model_bone_get_index)(self.handle, &mut index) })?;
        Ok(index)
    }

    /// The bone's name.
    pub fn name(&self) -> Result<String> {
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_bone_get_name_byte_count)(self.handle, &mut required)
        })?;
        read_text(required, |destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.models.model_bone_copy_name)(
                    self.handle,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }

    /// The bone's local transform.
    pub fn transform(&self) -> Result<Matrix> {
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_bone_get_transform)(self.handle, &mut value)
        })?;
        Ok(value_matrix(value))
    }

    /// Replaces the bone's local transform.
    pub fn set_transform(&self, value: Matrix) -> Result<()> {
        // SAFETY: the handle is owned and the matrix is passed by value.
        self.native.check(unsafe {
            (self.native.models.model_bone_set_transform)(self.handle, matrix(value))
        })
    }

    /// The parent bone, or `None` for a root.
    pub fn parent(&self) -> Result<Option<Self>> {
        let mut has_parent = 0_u8;
        let mut parent = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_bone_get_parent)(self.handle, &mut has_parent, &mut parent)
        })?;
        Ok((has_parent != 0).then(|| Self::new(&self.native, parent)))
    }

    /// The bone's children, in the model's own order.
    pub fn children(&self) -> Result<Vec<Self>> {
        let mut collection = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned collection handle.
        self.native.check(unsafe {
            (self.native.models.model_bone_get_children)(self.handle, &mut collection)
        })?;
        let collection = BoneCollection {
            native: Arc::clone(&self.native),
            handle: collection,
        };
        let count = collection.count()?;
        (0..count).map(|index| collection.get_at(index)).collect()
    }
}

impl Drop for ModelBoneView {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once. It is
        // the view CNA published, never the model's bone.
        let _ = unsafe { (self.native.models.model_bone_destroy)(self.handle) };
    }
}

/// One mesh of a loaded model.
///
/// Owns its own handle, on the same terms as [`ModelBoneView`].
pub struct ModelMeshView {
    native: Arc<Native>,
    handle: sys::CNA_ModelMeshHandle,
}

impl ModelMeshView {
    fn new(native: &Arc<Native>, handle: sys::CNA_ModelMeshHandle) -> Self {
        Self {
            native: Arc::clone(native),
            handle,
        }
    }

    /// The mesh's name.
    pub fn name(&self) -> Result<String> {
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_mesh_get_name_byte_count)(self.handle, &mut required)
        })?;
        read_text(required, |destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.models.model_mesh_copy_name)(
                    self.handle,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }

    /// The bone this mesh hangs from, or `None` when it hangs from no bone.
    pub fn parent_bone(&self) -> Result<Option<ModelBoneView>> {
        let mut has_parent = 0_u8;
        let mut bone = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_mesh_get_parent_bone)(
                self.handle,
                &mut has_parent,
                &mut bone,
            )
        })?;
        Ok((has_parent != 0).then(|| ModelBoneView::new(&self.native, bone)))
    }

    /// The mesh's bounding sphere.
    pub fn bounding_sphere(&self) -> Result<BoundingSphere> {
        let mut sphere = sys::CNA_BoundingSphere::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_mesh_get_bounding_sphere)(self.handle, &mut sphere)
        })?;
        Ok(bounding_sphere(sphere))
    }

    /// How many parts the mesh has.
    pub fn part_count(&self) -> Result<u64> {
        let parts = self.part_collection()?;
        let mut count = 0_u64;
        // SAFETY: the collection handle is owned and the output is a live
        // local.
        self.native.check(unsafe {
            (self.native.models.model_mesh_part_collection_get_count)(parts.handle, &mut count)
        })?;
        Ok(count)
    }

    /// Every part, in the mesh's own order.
    pub fn parts(&self) -> Result<Vec<ModelMeshPartView>> {
        let parts = self.part_collection()?;
        let count = self.part_count()?;
        (0..count)
            .map(|index| {
                let mut part = sys::CNA_INVALID_HANDLE;
                // SAFETY: the collection handle is owned and the output is a
                // live local receiving a fresh owned view handle.
                self.native.check(unsafe {
                    (self.native.models.model_mesh_part_collection_get_at)(
                        parts.handle,
                        index,
                        &mut part,
                    )
                })?;
                Ok(ModelMeshPartView {
                    native: Arc::clone(&self.native),
                    handle: part,
                })
            })
            .collect()
    }

    /// How many distinct effects the mesh's parts use.
    pub fn effect_count(&self) -> Result<u64> {
        let effects = self.effect_collection()?;
        let mut count = 0_u64;
        // SAFETY: the collection handle is owned and the output is a live
        // local.
        self.native.check(unsafe {
            (self.native.models.model_effect_collection_get_count)(effects.handle, &mut count)
        })?;
        Ok(count)
    }

    /// The identity of each effect the mesh's parts use.
    ///
    /// Identities rather than effects. A model-owned effect refuses
    /// `cna_effect_destroy` -- disposing it would reach inside an asset the
    /// content manager is still caching -- and a loaded model's is documented
    /// as invalid past the model's own destruction. Handing out a Rust value
    /// that owns one would promise a lifetime this module cannot keep, so what
    /// comes back is the handle value, which is enough to tell two parts'
    /// effects apart and to match a part against
    /// [`ModelMeshPartView::effect_identity`].
    pub fn effect_identities(&self) -> Result<Vec<u64>> {
        let effects = self.effect_collection()?;
        let count = self.effect_count()?;
        (0..count)
            .map(|index| {
                let mut effect = sys::CNA_INVALID_HANDLE;
                // SAFETY: the collection handle is owned and the output is a
                // live local.
                self.native.check(unsafe {
                    (self.native.models.model_effect_collection_get_at)(
                        effects.handle,
                        index,
                        &mut effect,
                    )
                })?;
                Ok(effect)
            })
            .collect()
    }

    /// Whether the mesh's effect collection contains that effect identity.
    pub fn contains_effect(&self, identity: u64) -> Result<bool> {
        let effects = self.effect_collection()?;
        let mut contains = 0_u8;
        // SAFETY: the collection handle is owned and the output is a live
        // local; the identity is a handle value CNA validates itself.
        self.native.check(unsafe {
            (self.native.models.model_effect_collection_contains)(
                effects.handle,
                identity,
                &mut contains,
            )
        })?;
        Ok(contains != 0)
    }

    /// Draws the mesh with whatever its effects are currently set to.
    pub fn draw(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.models.model_mesh_draw)(self.handle) })
    }

    fn part_collection(&self) -> Result<PartCollection> {
        let mut collection = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned collection handle.
        self.native.check(unsafe {
            (self.native.models.model_mesh_get_mesh_parts)(self.handle, &mut collection)
        })?;
        Ok(PartCollection {
            native: Arc::clone(&self.native),
            handle: collection,
        })
    }

    fn effect_collection(&self) -> Result<EffectCollection> {
        let mut collection = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned collection handle.
        self.native.check(unsafe {
            (self.native.models.model_mesh_get_effects)(self.handle, &mut collection)
        })?;
        Ok(EffectCollection {
            native: Arc::clone(&self.native),
            handle: collection,
        })
    }
}

impl Drop for ModelMeshView {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once.
        let _ = unsafe { (self.native.models.model_mesh_destroy)(self.handle) };
    }
}

/// One drawable part of a mesh.
///
/// What a part *holds* -- its effect, its vertex buffer, its index buffer -- is
/// reported as presence and identity rather than as owned Rust values. Those
/// handles are the model's, not the caller's, and upstream documents a
/// content-loaded model's as invalid past `cna_model_destroy`; a Rust value
/// owning one would be promising a lifetime this module cannot enforce.
pub struct ModelMeshPartView {
    native: Arc<Native>,
    handle: sys::CNA_ModelMeshPartHandle,
}

impl ModelMeshPartView {
    /// The identity of the part's effect, or `None` when it has none.
    pub fn effect_identity(&self) -> Result<Option<u64>> {
        let mut has_effect = 0_u8;
        let mut effect = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_mesh_part_get_effect)(
                self.handle,
                &mut has_effect,
                &mut effect,
            )
        })?;
        Ok((has_effect != 0).then_some(effect))
    }

    /// The identity of the part's vertex buffer, or `None` when it has none.
    pub fn vertex_buffer_identity(&self) -> Result<Option<u64>> {
        let mut has_buffer = 0_u8;
        let mut buffer = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_mesh_part_get_vertex_buffer)(
                self.handle,
                &mut has_buffer,
                &mut buffer,
            )
        })?;
        Ok((has_buffer != 0).then_some(buffer))
    }

    /// The identity of the part's index buffer, or `None` when it has none.
    pub fn index_buffer_identity(&self) -> Result<Option<u64>> {
        let mut has_buffer = 0_u8;
        let mut buffer = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_mesh_part_get_index_buffer)(
                self.handle,
                &mut has_buffer,
                &mut buffer,
            )
        })?;
        Ok((has_buffer != 0).then_some(buffer))
    }
}

impl Drop for ModelMeshPartView {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once. The
        // part-destroy route lives in the engine table because the morph slice
        // bound it first; it is the same route either way.
        let _ = unsafe { (self.native.engine.model_mesh_part_destroy)(self.handle) };
    }
}

/// A collection handle, held only long enough to read through it.
///
/// These never reach the public API. XNA's `ModelBoneCollection` and friends are
/// collection *objects* because C# needed a type to hang `TryGetValue` and an
/// enumerator on; Rust already has `Vec` and `Iterator`, so the collection is an
/// implementation detail and what a caller gets back is a plain `Vec`.
struct BoneCollection {
    native: Arc<Native>,
    handle: sys::CNA_ModelBoneCollectionHandle,
}

impl BoneCollection {
    fn count(&self) -> Result<u64> {
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_bone_collection_get_count)(self.handle, &mut count)
        })?;
        Ok(count)
    }

    fn get_at(&self, index: u64) -> Result<ModelBoneView> {
        let mut bone = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned view handle.
        self.native.check(unsafe {
            (self.native.models.model_bone_collection_get_at)(self.handle, index, &mut bone)
        })?;
        Ok(ModelBoneView::new(&self.native, bone))
    }

    fn find(&self, name: &str) -> Result<Option<ModelBoneView>> {
        let mut found = 0_u8;
        let mut bone = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_bone_collection_find)(
                self.handle,
                string_view(name),
                &mut found,
                &mut bone,
            )
        })?;
        Ok((found != 0).then(|| ModelBoneView::new(&self.native, bone)))
    }
}

impl Drop for BoneCollection {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.models.model_bone_collection_destroy)(self.handle) };
    }
}

struct MeshCollection {
    native: Arc<Native>,
    handle: sys::CNA_ModelMeshCollectionHandle,
}

impl MeshCollection {
    fn count(&self) -> Result<u64> {
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.models.model_mesh_collection_get_count)(self.handle, &mut count)
        })?;
        Ok(count)
    }

    fn get_at(&self, index: u64) -> Result<ModelMeshView> {
        let mut mesh = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a fresh owned view handle.
        self.native.check(unsafe {
            (self.native.models.model_mesh_collection_get_at)(self.handle, index, &mut mesh)
        })?;
        Ok(ModelMeshView::new(&self.native, mesh))
    }

    fn find(&self, name: &str) -> Result<Option<ModelMeshView>> {
        let mut found = 0_u8;
        let mut mesh = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_mesh_collection_find)(
                self.handle,
                string_view(name),
                &mut found,
                &mut mesh,
            )
        })?;
        Ok((found != 0).then(|| ModelMeshView::new(&self.native, mesh)))
    }
}

impl Drop for MeshCollection {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.models.model_mesh_collection_destroy)(self.handle) };
    }
}

struct PartCollection {
    native: Arc<Native>,
    handle: sys::CNA_ModelMeshPartCollectionHandle,
}

impl Drop for PartCollection {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.models.model_mesh_part_collection_destroy)(self.handle) };
    }
}

struct EffectCollection {
    native: Arc<Native>,
    handle: sys::CNA_ModelEffectCollectionHandle,
}

impl Drop for EffectCollection {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.models.model_effect_collection_destroy)(self.handle) };
    }
}

fn handle_of(slot: &Mutex<sys::CNA_Handle>, released: &'static str) -> Result<sys::CNA_Handle> {
    let handle = *slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if handle == sys::CNA_INVALID_HANDLE {
        return Err(CnaError::InvalidInput(released));
    }
    Ok(handle)
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

/// CNA's count-then-copy text protocol.
fn read_text(
    required: u64,
    mut route: impl FnMut(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
) -> Result<String> {
    if required == 0 {
        return Ok(String::new());
    }
    let capacity =
        usize::try_from(required).map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    let result = route(
        buffer.as_mut_ptr().cast::<core::ffi::c_char>(),
        required,
        &mut written,
    );
    if result != sys::CNA_RESULT_SUCCESS {
        return Err(CnaError::InvalidInput("CNA refused to copy the text"));
    }
    buffer.truncate((written as usize).min(capacity));
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf8(buffer).map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
}

fn bounding_sphere(value: sys::CNA_BoundingSphere) -> BoundingSphere {
    BoundingSphere {
        Center: Vector3 {
            X: value.center.x,
            Y: value.center.y,
            Z: value.center.z,
        },
        Radius: value.radius,
    }
}

fn matrix(value: Matrix) -> sys::CNA_Matrix {
    sys::CNA_Matrix {
        m11: value.M11, m12: value.M12, m13: value.M13, m14: value.M14,
        m21: value.M21, m22: value.M22, m23: value.M23, m24: value.M24,
        m31: value.M31, m32: value.M32, m33: value.M33, m34: value.M34,
        m41: value.M41, m42: value.M42, m43: value.M43, m44: value.M44,
    }
}

fn value_matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix {
        M11: value.m11, M12: value.m12, M13: value.m13, M14: value.m14,
        M21: value.m21, M22: value.m22, M23: value.m23, M24: value.m24,
        M31: value.m31, M32: value.m32, M33: value.m33, M34: value.m34,
        M41: value.m41, M42: value.m42, M43: value.m43, M44: value.m44,
    }
}

/// The `Tag` the content pipeline wrote, beside the C-owned one.
impl NativeModel {
    /// The model's content `Tag`, as a dictionary.
    ///
    /// `None` when the model carries no tag *or* carries one of another shape,
    /// which upstream is explicit is not an error: an unset `Tag` is `null` in
    /// XNA and absent here.
    ///
    /// The dictionary **outlives the model**: it keeps the loaded asset's data
    /// alive on its own, so releasing the model first is safe and does not
    /// invalidate it. That is why it comes back owned rather than borrowed.
    pub fn content_tag(&self) -> Result<Option<ObjectDictionary>> {
        let handle = self.get()?;
        let mut has_tag = 0_u8;
        let mut dictionary = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.models.model_get_content_tag_dictionary_ext)(
                handle,
                &mut has_tag,
                &mut dictionary,
            )
        })?;
        Ok((has_tag != 0).then(|| ObjectDictionary::from_owned_handle(&self.native, dictionary)))
    }

    /// The model's content `Tag`, as an object a caller's own reflective reader
    /// made.
    ///
    /// The other half of a caller-registered reflective type: `ModelReader`'s
    /// tag path takes a reference and refuses a value, so a type registered the
    /// value-shaped way fails the load rather than arriving in the wrong form.
    ///
    /// The pointer is the caller's own -- CNA never dereferences, copies or
    /// frees it -- and stays valid as long as the caller keeps it so. `None`
    /// when the model has no tag or carries one of another shape.
    pub fn content_tag_foreign_object(&self) -> Result<Option<*mut core::ffi::c_void>> {
        let handle = self.get()?;
        let mut has_tag = 0_u8;
        let mut object = core::ptr::null_mut();
        // SAFETY: the handle is owned and both outputs are live locals. The
        // pointer is not dereferenced here.
        self.native.check(unsafe {
            (self.native.models.model_get_content_tag_foreign_object_ext)(
                handle,
                &mut has_tag,
                &mut object,
            )
        })?;
        Ok((has_tag != 0).then_some(object))
    }
}
