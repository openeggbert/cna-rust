//! CNA's skinned-model extension: a skeleton, its animation clips and the
//! renderable parts that ride on it.
//!
//! None of this is XNA. `Microsoft.Xna.Framework.Graphics.Model` has bones and
//! meshes but no skinning, no clips and no way to evaluate a pose; the skinned
//! model is CNA's own type and lives here rather than in the strict projection.
//!
//! ## Ownership
//!
//! [`SkinnedModel`] is `OWNED`: it holds a handle it releases exactly once, and
//! it needs no device and no game -- upstream creates it standalone, so nothing
//! here registers with a device.
//!
//! Its parts are consumed rather than borrowed. [`SkinnedModel::add_part`]
//! *takes* the mesh part, because upstream counts a live model against it and
//! refuses a part that already belongs to one -- a part is a model's or it is
//! the caller's, never both. The vertex buffer, index buffer and texture are
//! `RETAINED_DEPENDENCY`: upstream keeps its own reference, but the *Rust*
//! value destroying the native object while the model still points at it is
//! refused and strands it, so the model keeps a share of each.
//!
//! [`SkinnedModel::part_has_texture_at`] reports the texture as *presence*
//! rather than handing one back, because the route behind it answers two
//! handles with opposite ownership: the mesh part is a **new** handle the
//! caller releases, and the texture is the model's **own**, which a caller
//! that released it would free twice.

#![allow(clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::engine::NativeMeshPart;
use crate::graphics::{IndexBuffer, Texture2D, VertexBuffer};
use crate::native::Native;
use crate::value::{Matrix, Quaternion, Vector3};

/// Which index space a clip's bone indices are in.
///
/// The two are deliberately distinct and must never be interchanged: a joint's
/// palette slot has nothing to do with its position in the scene, and a rigid
/// scene node has no palette slot at all.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ClipTargetSpace {
    /// Indices into the skinning palette.
    #[default]
    JointPalette,
    /// Indices into the scene graph.
    SceneNode,
}

impl ClipTargetSpace {
    #[must_use]
    pub const fn to_native(self) -> sys::CNA_ClipTargetSpaceEXT {
        match self {
            Self::JointPalette => sys::CNA_CLIP_TARGET_SPACE_JOINT_PALETTE_EXT,
            Self::SceneNode => sys::CNA_CLIP_TARGET_SPACE_SCENE_NODE_EXT,
        }
    }

    #[must_use]
    pub const fn from_native(value: sys::CNA_ClipTargetSpaceEXT) -> Option<Self> {
        match value {
            sys::CNA_CLIP_TARGET_SPACE_JOINT_PALETTE_EXT => Some(Self::JointPalette),
            sys::CNA_CLIP_TARGET_SPACE_SCENE_NODE_EXT => Some(Self::SceneNode),
            _ => None,
        }
    }
}

/// One bone pose at one instant.
///
/// Constructible, unlike most values in this crate: a caller authors keyframes
/// rather than reading them back from CNA, so a `#[non_exhaustive]` that forced
/// every clip through a builder would buy nothing. The types CNA *fills* --
/// [`ClipInfo`] and [`OwnedResourceCounts`] -- stay closed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    /// When, in seconds.
    pub time_seconds: f64,
    /// Bone-local translation.
    pub translation: Vector3,
    /// Bone-local rotation.
    pub rotation: Quaternion,
    /// Bone-local scale.
    pub scale: Vector3,
}

impl Keyframe {
    fn to_native(self) -> sys::CNA_KeyframeEXT {
        sys::CNA_KeyframeEXT {
            time_seconds: self.time_seconds,
            translation: sys::CNA_Vector3 {
                x: self.translation.X,
                y: self.translation.Y,
                z: self.translation.Z,
            },
            rotation: sys::CNA_Quaternion {
                x: self.rotation.X,
                y: self.rotation.Y,
                z: self.rotation.Z,
                w: self.rotation.W,
            },
            scale: sys::CNA_Vector3 {
                x: self.scale.X,
                y: self.scale.Y,
                z: self.scale.Z,
            },
        }
    }

    const fn from_native(value: sys::CNA_KeyframeEXT) -> Self {
        Self {
            time_seconds: value.time_seconds,
            translation: Vector3 {
                X: value.translation.x,
                Y: value.translation.y,
                Z: value.translation.z,
            },
            rotation: Quaternion {
                X: value.rotation.x,
                Y: value.rotation.y,
                Z: value.rotation.z,
                W: value.rotation.w,
            },
            scale: Vector3 {
                X: value.scale.x,
                Y: value.scale.y,
                Z: value.scale.z,
            },
        }
    }
}

/// Every keyframe driving one bone.
///
/// A bone index outside the skeleton is **skipped rather than refused**, which
/// is upstream's own behaviour: a clip authored against a longer skeleton still
/// plays the bones it does have.
#[derive(Clone, Debug, PartialEq)]
pub struct BoneTrack {
    /// Which bone the track drives.
    pub bone_index: i32,
    /// Its keyframes, in time order.
    pub keyframes: Vec<Keyframe>,
}

/// One animation clip: a duration and the tracks that fill it.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClip {
    /// How long the clip runs, in seconds.
    pub duration_seconds: f64,
    /// The bone tracks it drives.
    pub tracks: Vec<BoneTrack>,
}

/// What one clip's shape is, without its keyframes.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ClipInfo {
    /// How long the clip runs, in seconds.
    pub duration_seconds: f64,
    /// How many bone tracks it has.
    pub track_count: u64,
}

/// A skeleton, its clips and the parts that ride on it.
pub struct SkinnedModel {
    handle: Mutex<sys::CNA_SkinnedModelEXTHandle>,
    native: Arc<Native>,
    /// The mesh parts this model has taken over, kept so their Rust values are
    /// released *after* the model that references them. CNA counts a live
    /// model against each part, so a part destroyed first is refused and
    /// stranded; holding them here makes that order impossible to get wrong.
    parts: Mutex<Vec<(String, NativeMeshPart)>>,
    /// The device resources upstream retains on the caller's behalf. CNA holds
    /// its own reference, but the *Rust* value destroying the native object
    /// while the model still references it is refused and strands it, so the
    /// model keeps a share of each. Buffers clone; a texture does not, so it is
    /// moved in.
    retained: Mutex<RetainedResources>,
}

/// The device resources a [`SkinnedModel`] holds a share of.
#[derive(Default)]
struct RetainedResources {
    vertex_buffers: Vec<VertexBuffer>,
    index_buffers: Vec<IndexBuffer>,
    textures: Vec<Texture2D>,
}

impl SkinnedModel {
    /// Creates an empty model: no bones, no clips, no parts.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.engine.skinned_model_ext_create_default)(&mut handle) })?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
            parts: Mutex::new(Vec::new()),
            retained: Mutex::new(RetainedResources::default()),
        })
    }

    /// Creates a model from a skeleton and a set of named clips.
    pub fn with_skeleton(
        parent_bone_indices: &[i32],
        bind_pose_local: &[Matrix],
        inverse_bind_pose_global: &[Matrix],
        clips: &[(String, AnimationClip)],
    ) -> Result<Self> {
        let native = Native::process()?;
        let bone_count = i32::try_from(parent_bone_indices.len())
            .map_err(|_| CnaError::InvalidInput("more bones than a skeleton can hold"))?;
        if bind_pose_local.len() != parent_bone_indices.len()
            || inverse_bind_pose_global.len() != parent_bone_indices.len()
        {
            return Err(CnaError::InvalidInput(
                "a skeleton needs one parent index and two matrices per bone",
            ));
        }
        let bind: Vec<sys::CNA_Matrix> = bind_pose_local.iter().copied().map(matrix).collect();
        let inverse: Vec<sys::CNA_Matrix> = inverse_bind_pose_global
            .iter()
            .copied()
            .map(matrix)
            .collect();
        // The keyframe and track arrays have to outlive the call, so they are
        // built here and borrowed by the descriptors below rather than being
        // temporaries inside an expression.
        let staged = StagedClips::new(clips);
        let descriptor = sys::CNA_SkinnedModelEXTDescriptor {
            bone_count,
            reserved: 0,
            parent_bone_indices: parent_bone_indices.as_ptr(),
            bind_pose_local: bind.as_ptr(),
            inverse_bind_pose_global: inverse.as_ptr(),
            clips: staged.named.as_ptr(),
            clip_count: staged.named.len() as u64,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: every array the descriptor points at outlives the call, and
        // the output is a live local.
        native.check(unsafe {
            (native.engine.skinned_model_ext_create)(&descriptor, &mut handle)
        })?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
            parts: Mutex::new(Vec::new()),
            retained: Mutex::new(RetainedResources::default()),
        })
    }

    fn get(&self) -> Result<sys::CNA_SkinnedModelEXTHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the skinned model has been released"));
        }
        Ok(handle)
    }

    /// Replaces the skeleton, keeping the clips and parts.
    pub fn set_skeleton(
        &self,
        parent_bone_indices: &[i32],
        bind_pose_local: &[Matrix],
        inverse_bind_pose_global: &[Matrix],
    ) -> Result<()> {
        let handle = self.get()?;
        let bone_count = i32::try_from(parent_bone_indices.len())
            .map_err(|_| CnaError::InvalidInput("more bones than a skeleton can hold"))?;
        let bind: Vec<sys::CNA_Matrix> = bind_pose_local.iter().copied().map(matrix).collect();
        let inverse: Vec<sys::CNA_Matrix> = inverse_bind_pose_global
            .iter()
            .copied()
            .map(matrix)
            .collect();
        // SAFETY: the handle is owned and all three arrays are borrowed for the
        // call with the bone count they were sized against.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_set_skeleton)(
                handle,
                bone_count,
                parent_bone_indices.as_ptr(),
                bind.as_ptr(),
                inverse.as_ptr(),
            )
        })
    }

    /// How many bones the skeleton has.
    pub fn bone_count(&self) -> Result<u64> {
        self.count(self.native.engine.skinned_model_ext_get_bone_count)
    }

    /// How many clips the model carries.
    pub fn clip_count(&self) -> Result<u64> {
        self.count(self.native.engine.skinned_model_ext_get_clip_count)
    }

    /// How many parts it carries.
    pub fn part_count(&self) -> Result<u64> {
        self.count(self.native.engine.skinned_model_ext_get_part_count)
    }

    fn count(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
    ) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    /// The parent index of every bone.
    pub fn parent_bone_indices(&self) -> Result<Vec<i32>> {
        let handle = self.get()?;
        let required = self.bone_count()?;
        let mut buffer = vec![0_i32; required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable values, which is the capacity passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_copy_parent_bone_indices)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        buffer.truncate(count as usize);
        Ok(buffer)
    }

    /// The local bind pose of every bone.
    pub fn bind_pose_local(&self) -> Result<Vec<Matrix>> {
        self.matrices(self.native.engine.skinned_model_ext_copy_bind_pose_local)
    }

    /// The inverse global bind pose of every bone.
    pub fn inverse_bind_pose_global(&self) -> Result<Vec<Matrix>> {
        self.matrices(
            self.native
                .engine
                .skinned_model_ext_copy_inverse_bind_pose_global,
        )
    }

    fn matrices(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_Handle,
            *mut sys::CNA_Matrix,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        let required = self.bone_count()?;
        let mut buffer = vec![sys::CNA_Matrix::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable matrices, which is the capacity passed alongside it.
        self.native
            .check(unsafe { route(handle, buffer.as_mut_ptr(), required, &mut count) })?;
        buffer.truncate(count as usize);
        Ok(buffer.into_iter().map(from_matrix).collect())
    }

    /// Adds a clip under a name, replacing any clip already under it.
    pub fn set_clip(&self, name: &str, clip: &AnimationClip) -> Result<()> {
        let handle = self.get()?;
        let staged = StagedClip::new(clip);
        // SAFETY: the handle is owned, the name borrows a Rust string that
        // outlives the call, and every array the descriptor points at is kept
        // alive by `staged` for the same span.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_set_clip)(
                handle,
                view(name),
                &staged.descriptor,
            )
        })
    }

    /// Removes the clip under a name.
    pub fn remove_clip(&self, name: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the name is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_remove_clip)(handle, view(name))
        })
    }

    /// The name of the clip at an index.
    pub fn clip_name_at(&self, index: u64) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinned_model_ext_get_clip_name_byte_count_at)(
                handle, index, &mut required
            )
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.skinned_model_ext_copy_clip_name_at)(
                    handle,
                    index,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// The shape of the clip under a name, or `None` when there is none.
    pub fn clip_info(&self, name: &str) -> Result<Option<ClipInfo>> {
        let handle = self.get()?;
        let mut found = 0_u8;
        let mut duration = 0.0_f64;
        let mut tracks = 0_u64;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_get_clip_info)(
                handle,
                view(name),
                &mut found,
                &mut duration,
                &mut tracks,
            )
        })?;
        Ok((found != 0).then_some(ClipInfo {
            duration_seconds: duration,
            track_count: tracks,
        }))
    }

    /// One track of the clip under a name.
    pub fn clip_track(&self, name: &str, track_index: u64) -> Result<BoneTrack> {
        let handle = self.get()?;
        let mut bone_index = 0_i32;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.skinned_model_ext_copy_clip_track)(
                handle,
                view(name),
                track_index,
                &mut bone_index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let mut buffer = vec![sys::CNA_KeyframeEXT::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable keyframes, which is the capacity passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_copy_clip_track)(
                handle,
                view(name),
                track_index,
                &mut bone_index,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        buffer.truncate(count as usize);
        Ok(BoneTrack {
            bone_index,
            keyframes: buffer.into_iter().map(Keyframe::from_native).collect(),
        })
    }

    /// The final skinning matrices for a clip at a playback position.
    ///
    /// This is what [`SkinnedPbrEffect::set_bone_transforms`] wants: without
    /// it, the effect has a bone palette and no way to fill it.
    ///
    /// [`SkinnedPbrEffect::set_bone_transforms`]: crate::extensions::pbr::SkinnedPbrEffect::set_bone_transforms
    pub fn compute_bone_transforms(
        &self,
        clip_name: &str,
        position_seconds: f64,
        loop_clip: bool,
    ) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        let required = self.bone_count()?;
        let mut buffer = vec![sys::CNA_Matrix::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // the destination holds `required` writable matrices.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_compute_bone_transforms)(
                handle,
                view(clip_name),
                position_seconds,
                u8::from(loop_clip),
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        buffer.truncate(count as usize);
        Ok(buffer.into_iter().map(from_matrix).collect())
    }

    /// Adds a renderable part, **taking** the mesh part.
    ///
    /// Upstream counts a live model against the part and refuses a part that
    /// already belongs to one, so a part is a model's or it is the caller's --
    /// never both. Rather than leave that to a comment, the part is moved in:
    /// the model holds its Rust value and releases it after itself, which is
    /// the only order CNA accepts.
    ///
    /// The vertex and index buffers are cloned into the model rather than
    /// merely read: they are `Arc`-backed, and upstream's own retention does
    /// not stop the *Rust* value from destroying the native object while the
    /// model still points at it. A texture does not clone, so it is moved in
    /// for the same reason.
    ///
    /// On failure the part and the texture both come back in the error,
    /// unconsumed.
    pub fn add_part(
        &self,
        name: &str,
        vertex_buffer: &VertexBuffer,
        index_buffer: &IndexBuffer,
        part: NativeMeshPart,
        texture: Option<Texture2D>,
    ) -> std::result::Result<(), PartNotAdded> {
        let outcome = (|| -> Result<()> {
            let handle = self.get()?;
            let texture_handle = match texture.as_ref() {
                Some(texture) => texture.handle()?,
                None => sys::CNA_INVALID_HANDLE,
            };
            // SAFETY: every handle is live for the call and the name borrows a
            // Rust string that outlives it.
            self.native.check(unsafe {
                (self.native.engine.skinned_model_ext_add_part)(
                    handle,
                    view(name),
                    vertex_buffer.handle()?,
                    index_buffer.handle()?,
                    part.native_handle()?,
                    texture_handle,
                )
            })
        })();
        match outcome {
            Ok(()) => {
                self.parts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((name.to_owned(), part));
                let mut retained = self
                    .retained
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                retained.vertex_buffers.push(vertex_buffer.clone());
                retained.index_buffers.push(index_buffer.clone());
                if let Some(texture) = texture {
                    retained.textures.push(texture);
                }
                Ok(())
            }
            Err(error) => Err(PartNotAdded {
                part,
                texture,
                error,
            }),
        }
    }

    /// Moves every part out of another model of the same skeleton.
    ///
    /// Replace-by-name: a part whose name this model already has is replaced
    /// rather than duplicated, and `other` is left with none. The Rust values
    /// move with them, so each part is still released after the model that
    /// holds it.
    pub fn attach_parts(&self, other: &Self) -> Result<()> {
        let handle = self.get()?;
        let source = other.get()?;
        // SAFETY: both handles are owned and live for the call.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_attach_parts)(handle, source)
        })?;
        self.take_parts_from(other);
        Ok(())
    }

    /// Moves the Rust side of `other`'s parts into this model, replacing any
    /// this model already holds under the same name.
    fn take_parts_from(&self, other: &Self) {
        let moved = core::mem::take(
            &mut *other
                .parts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let mut mine = self
            .parts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (name, part) in moved {
            mine.retain(|(existing, _)| *existing != name);
            mine.push((name, part));
        }
    }

    /// Removes the part under a name, releasing it.
    pub fn remove_part(&self, name: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the name is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_remove_part)(handle, view(name))
        })?;
        self.parts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(existing, _)| existing != name);
        Ok(())
    }

    /// The name of the part at an index.
    pub fn part_name_at(&self, index: u64) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinned_model_ext_get_part_name_byte_count_at)(
                handle, index, &mut required
            )
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.skinned_model_ext_copy_part_name_at)(
                    handle,
                    index,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// Whether the part at an index carries a texture.
    ///
    /// The texture *handle* is deliberately not published. The same call hands
    /// back two handles with opposite ownership: the mesh part is a **new**
    /// handle the caller releases, while the texture is the model's **own**,
    /// which a caller that released it would free twice. Rather than publish
    /// one and hide the other, this reports the texture as presence and
    /// releases the part alias it was given.
    pub fn part_has_texture_at(&self, index: u64) -> Result<bool> {
        let handle = self.get()?;
        let mut part = sys::CNA_INVALID_HANDLE;
        let mut has_texture = 0_u8;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_get_part_at)(
                handle,
                index,
                &mut part,
                &mut has_texture,
                &mut texture,
            )
        })?;
        if part != sys::CNA_INVALID_HANDLE {
            // SAFETY: the part handle is the alias CNA just published, released
            // exactly once here. The texture handle is the model's own and is
            // deliberately left alone.
            let _ = unsafe { (self.native.engine.model_mesh_part_destroy)(part) };
        }
        Ok(has_texture != 0)
    }

    /// How many of each resource the model owns, in CNA's own order.
    pub fn owned_resource_counts(&self) -> Result<OwnedResourceCounts> {
        let handle = self.get()?;
        let mut counts = OwnedResourceCounts::default();
        // SAFETY: the handle is owned and all four outputs are live locals.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinned_model_ext_get_owned_resource_counts)(
                handle,
                &mut counts.vertex_buffers,
                &mut counts.index_buffers,
                &mut counts.parts,
                &mut counts.textures,
            )
        })?;
        Ok(counts)
    }

    /// Moves this model's contents into a new one, leaving this one empty.
    ///
    /// C++ move construction, exposed in C and therefore here. The source stays
    /// a valid, usable model -- it simply has nothing in it afterwards -- so
    /// this is a content transfer rather than a handle transfer, and a failure
    /// leaves both sides as they were.
    pub fn move_out(&self) -> Result<Self> {
        let source = self.get()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the source handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_create_move)(source, &mut handle)
        })?;
        let moved = Self {
            handle: Mutex::new(handle),
            native: Arc::clone(&self.native),
            parts: Mutex::new(Vec::new()),
            retained: Mutex::new(RetainedResources::default()),
        };
        moved.take_parts_from(self);
        Ok(moved)
    }

    /// Moves another model's contents into this one, leaving it empty.
    pub fn move_assign_from(&self, other: &Self) -> Result<()> {
        let destination = self.get()?;
        let source = other.get()?;
        // SAFETY: both handles are owned and live for the call.
        self.native.check(unsafe {
            (self.native.engine.skinned_model_ext_move_assign)(destination, source)
        })?;
        // Move-assignment replaces this model's contents outright, so its own
        // parts go with them rather than being merged.
        self.parts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.take_parts_from(other);
        Ok(())
    }

    /// Releases the model now rather than at drop.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *guard;
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here; the slot is cleared only once the
        // destroy has succeeded.
        self.native
            .check(unsafe { (self.native.engine.skinned_model_ext_destroy)(handle) })?;
        *guard = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for SkinnedModel {
    fn drop(&mut self) {
        // The model first, then the parts it holds: CNA counts a live model
        // against each part and refuses a part released while one references
        // it, so the reverse order strands every part.
        let _ = self.release();
        self.parts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        retained.vertex_buffers.clear();
        retained.index_buffers.clear();
        retained.textures.clear();
    }
}

/// A mesh part a model refused to take over.
pub struct PartNotAdded {
    /// The part, still owned by the caller.
    pub part: NativeMeshPart,
    /// The texture that would have gone with it, if there was one.
    pub texture: Option<Texture2D>,
    /// Why the model refused it.
    pub error: CnaError,
}

impl core::fmt::Debug for PartNotAdded {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PartNotAdded")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Display for PartNotAdded {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "the mesh part was not added: {}", self.error)
    }
}

impl std::error::Error for PartNotAdded {}

/// How many of each resource a [`SkinnedModel`] owns.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct OwnedResourceCounts {
    /// Vertex buffers.
    pub vertex_buffers: u64,
    /// Index buffers.
    pub index_buffers: u64,
    /// Mesh parts.
    pub parts: u64,
    /// Textures.
    pub textures: u64,
}

/// One clip's keyframes and tracks, kept alive while CNA reads them.
struct StagedClip {
    descriptor: sys::CNA_AnimationClipEXTDescriptor,
    _keyframes: Vec<Vec<sys::CNA_KeyframeEXT>>,
    _tracks: Vec<sys::CNA_BoneTrackEXTDescriptor>,
}

impl StagedClip {
    fn new(clip: &AnimationClip) -> Self {
        let keyframes: Vec<Vec<sys::CNA_KeyframeEXT>> = clip
            .tracks
            .iter()
            .map(|track| track.keyframes.iter().copied().map(Keyframe::to_native).collect())
            .collect();
        let tracks: Vec<sys::CNA_BoneTrackEXTDescriptor> = clip
            .tracks
            .iter()
            .zip(keyframes.iter())
            .map(|(track, frames)| sys::CNA_BoneTrackEXTDescriptor {
                bone_index: track.bone_index,
                reserved: 0,
                keyframes: frames.as_ptr(),
                keyframe_count: frames.len() as u64,
            })
            .collect();
        let descriptor = sys::CNA_AnimationClipEXTDescriptor {
            duration_seconds: clip.duration_seconds,
            tracks: tracks.as_ptr(),
            track_count: tracks.len() as u64,
        };
        Self {
            descriptor,
            _keyframes: keyframes,
            _tracks: tracks,
        }
    }
}

/// Several named clips, kept alive while CNA reads them.
struct StagedClips {
    named: Vec<sys::CNA_NamedAnimationClipEXTDescriptor>,
    _staged: Vec<StagedClip>,
    _names: Vec<String>,
}

impl StagedClips {
    fn new(clips: &[(String, AnimationClip)]) -> Self {
        let names: Vec<String> = clips.iter().map(|(name, _)| name.clone()).collect();
        let staged: Vec<StagedClip> = clips.iter().map(|(_, clip)| StagedClip::new(clip)).collect();
        let named = names
            .iter()
            .zip(staged.iter())
            .map(|(name, clip)| sys::CNA_NamedAnimationClipEXTDescriptor {
                name: view(name),
                clip: clip.descriptor,
            })
            .collect();
        Self {
            named,
            _staged: staged,
            _names: names,
        }
    }
}

fn view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

const fn matrix(value: Matrix) -> sys::CNA_Matrix {
    sys::CNA_Matrix {
        m11: value.M11,
        m12: value.M12,
        m13: value.M13,
        m14: value.M14,
        m21: value.M21,
        m22: value.M22,
        m23: value.M23,
        m24: value.M24,
        m31: value.M31,
        m32: value.M32,
        m33: value.M33,
        m34: value.M34,
        m41: value.M41,
        m42: value.M42,
        m43: value.M43,
        m44: value.M44,
    }
}

const fn from_matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix {
        M11: value.m11,
        M12: value.m12,
        M13: value.m13,
        M14: value.m14,
        M21: value.m21,
        M22: value.m22,
        M23: value.m23,
        M24: value.m24,
        M31: value.m31,
        M32: value.m32,
        M33: value.m33,
        M34: value.m34,
        M41: value.m41,
        M42: value.m42,
        M43: value.m43,
        M44: value.m44,
    }
}

/// CNA's count-then-copy text protocol, where the count is a separate route.
fn read_text(
    required: u64,
    mut route: impl FnMut(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
) -> Result<Option<String>> {
    if required == 0 {
        return Ok(Some(String::new()));
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
    Ok(String::from_utf8(buffer).ok())
}

/// The skeleton and clips an [`AnimationPlayer`] plays.
///
/// `OWNED`, and immutable except for the two extension knobs -- the clip target
/// space and the skeleton root identity. Everything else is fixed at
/// construction, because a player holds the data and a skeleton that changed
/// underneath it would invalidate the pose it is halfway through computing.
pub struct SkinningData {
    handle: Mutex<sys::CNA_SkinningDataHandle>,
    native: Arc<Native>,
}

impl SkinningData {
    /// Creates skinning data from a skeleton, an optional root prefix and a set
    /// of named clips.
    ///
    /// `skeleton_root_prefix` is either empty or one matrix per bone: a partial
    /// prefix is refused rather than padded, because a prefix that covered only
    /// some bones would place the rest of the skeleton somewhere arbitrary.
    pub fn new(
        skeleton_hierarchy: &[i32],
        bind_pose: &[Matrix],
        inverse_bind_pose: &[Matrix],
        skeleton_root_prefix: &[Matrix],
        clips: &[(String, AnimationClip)],
    ) -> Result<Self> {
        let native = Native::process()?;
        let bone_count = i32::try_from(skeleton_hierarchy.len())
            .map_err(|_| CnaError::InvalidInput("more bones than a skeleton can hold"))?;
        if bind_pose.len() != skeleton_hierarchy.len()
            || inverse_bind_pose.len() != skeleton_hierarchy.len()
        {
            return Err(CnaError::InvalidInput(
                "a skeleton needs one parent index and two matrices per bone",
            ));
        }
        let bind: Vec<sys::CNA_Matrix> = bind_pose.iter().copied().map(matrix).collect();
        let inverse: Vec<sys::CNA_Matrix> =
            inverse_bind_pose.iter().copied().map(matrix).collect();
        let prefix: Vec<sys::CNA_Matrix> =
            skeleton_root_prefix.iter().copied().map(matrix).collect();
        let staged = StagedClips::new(clips);
        let descriptor = sys::CNA_SkinningDataDescriptor {
            bone_count,
            reserved: 0,
            skeleton_hierarchy: skeleton_hierarchy.as_ptr(),
            bind_pose: bind.as_ptr(),
            inverse_bind_pose: inverse.as_ptr(),
            skeleton_root_prefix: if prefix.is_empty() {
                core::ptr::null()
            } else {
                prefix.as_ptr()
            },
            skeleton_root_prefix_count: prefix.len() as u64,
            clips: staged.named.as_ptr(),
            clip_count: staged.named.len() as u64,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: every array the descriptor points at outlives the call, and
        // the output is a live local.
        native.check(unsafe { (native.engine.skinning_data_create)(&descriptor, &mut handle) })?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    fn get(&self) -> Result<sys::CNA_SkinningDataHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the skinning data has been released"));
        }
        Ok(handle)
    }

    /// CNA's own name for the type, as the content pipeline writes it.
    pub fn type_name(&self) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_type_name_byte_count)(handle, &mut required)
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.skinning_data_copy_type_name)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// How many bones the skeleton has.
    pub fn bone_count(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_bone_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The parent index of every bone.
    pub fn skeleton_hierarchy(&self) -> Result<Vec<i32>> {
        let handle = self.get()?;
        let required = self.bone_count()?;
        let mut buffer = vec![0_i32; required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable values.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_copy_skeleton_hierarchy)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        buffer.truncate(count as usize);
        Ok(buffer)
    }

    /// The local bind pose of every bone.
    pub fn bind_pose(&self) -> Result<Vec<Matrix>> {
        self.matrices(self.native.engine.skinning_data_copy_bind_pose)
    }

    /// The inverse global bind pose of every bone.
    pub fn inverse_bind_pose(&self) -> Result<Vec<Matrix>> {
        self.matrices(self.native.engine.skinning_data_copy_inverse_bind_pose)
    }

    /// The root prefix, empty when there is none.
    pub fn skeleton_root_prefix(&self) -> Result<Vec<Matrix>> {
        self.matrices(self.native.engine.skinning_data_copy_skeleton_root_prefix)
    }

    fn matrices(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_Handle,
            *mut sys::CNA_Matrix,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe { route(handle, core::ptr::null_mut(), 0, &mut required) };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        if required == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_Matrix::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable matrices.
        self.native
            .check(unsafe { route(handle, buffer.as_mut_ptr(), required, &mut count) })?;
        buffer.truncate(count as usize);
        Ok(buffer.into_iter().map(from_matrix).collect())
    }

    /// How many clips it carries.
    pub fn clip_count(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_clip_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The name of the clip at an index.
    pub fn clip_name_at(&self, index: u64) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_clip_name_byte_count_at)(
                handle, index, &mut required,
            )
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.skinning_data_copy_clip_name_at)(
                    handle,
                    index,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// The shape of the clip under a name, or `None` when there is none.
    pub fn clip_info(&self, name: &str) -> Result<Option<ClipInfo>> {
        let handle = self.get()?;
        let mut found = 0_u8;
        let mut duration = 0.0_f64;
        let mut tracks = 0_u64;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_clip_info)(
                handle,
                view(name),
                &mut found,
                &mut duration,
                &mut tracks,
            )
        })?;
        Ok((found != 0).then_some(ClipInfo {
            duration_seconds: duration,
            track_count: tracks,
        }))
    }

    /// One track of the clip under a name.
    pub fn clip_track(&self, name: &str, track_index: u64) -> Result<BoneTrack> {
        let handle = self.get()?;
        let mut bone_index = 0_i32;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.skinning_data_copy_clip_track)(
                handle,
                view(name),
                track_index,
                &mut bone_index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let mut buffer = vec![sys::CNA_KeyframeEXT::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable keyframes.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_copy_clip_track)(
                handle,
                view(name),
                track_index,
                &mut bone_index,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        buffer.truncate(count as usize);
        Ok(BoneTrack {
            bone_index,
            keyframes: buffer.into_iter().map(Keyframe::from_native).collect(),
        })
    }

    /// Which index space the clip at an index targets.
    pub fn clip_target_space(&self, index: u64) -> Result<ClipTargetSpace> {
        let handle = self.get()?;
        let mut value: sys::CNA_ClipTargetSpaceEXT = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_get_clip_target_space_ext)(handle, index, &mut value)
        })?;
        ClipTargetSpace::from_native(value)
            .ok_or(CnaError::InvalidInput("native clip target space is unknown"))
    }

    /// Sets it.
    pub fn set_clip_target_space(&self, index: u64, value: ClipTargetSpace) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_set_clip_target_space_ext)(
                handle,
                index,
                value.to_native(),
            )
        })
    }

    /// Which scene node the skeleton hangs from, or a negative value for none.
    pub fn skeleton_root_node_index(&self) -> Result<i32> {
        let handle = self.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinning_data_get_skeleton_root_node_index_ext)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets it.
    pub fn set_skeleton_root_node_index(&self, value: i32) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinning_data_set_skeleton_root_node_index_ext)(handle, value)
        })
    }

    /// The name of that node, empty when there is none.
    pub fn skeleton_root_name(&self) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .skinning_data_get_skeleton_root_name_byte_count_ext)(
                handle, &mut required
            )
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.skinning_data_copy_skeleton_root_name_ext)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// Sets it.
    pub fn set_skeleton_root_name(&self, name: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the name is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.skinning_data_set_skeleton_root_name_ext)(handle, view(name))
        })
    }

    /// Releases the data now rather than at drop.
    ///
    /// Refused while a player still holds it: upstream retains the data, and a
    /// player whose skeleton vanished would compute a pose from nothing.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *guard;
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here; the slot is cleared only once the
        // destroy has succeeded.
        self.native
            .check(unsafe { (self.native.engine.skinning_data_destroy)(handle) })?;
        *guard = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for SkinningData {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Plays one clip of a [`SkinningData`] and keeps the three transform arrays a
/// skinned draw needs.
///
/// `OWNED`. The data is `RETAINED_DEPENDENCY`: upstream retains it, and this
/// value holds the Rust side so the data outlives the player that reads it.
///
/// The three arrays are different things and are not interchangeable: the bone
/// transforms are each bone's local pose, the world transforms are those
/// composed down the hierarchy, and the skin transforms are the world
/// transforms times the inverse bind pose -- the last is what a shader wants.
pub struct AnimationPlayer {
    handle: Mutex<sys::CNA_AnimationPlayerHandle>,
    native: Arc<Native>,
    /// The data upstream retains. Held so the Rust value cannot release it
    /// while this player still reads it.
    data: Arc<SkinningData>,
}

impl AnimationPlayer {
    /// Creates a player over some skinning data.
    pub fn new(data: &Arc<SkinningData>) -> Result<Self> {
        let native = Arc::clone(&data.native);
        let handle_in = data.get()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the data handle is live for the call and the output is a live
        // local.
        native.check(unsafe { (native.engine.animation_player_create)(handle_in, &mut handle) })?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
            data: Arc::clone(data),
        })
    }

    /// The data this player is keeping alive.
    #[must_use]
    pub fn skinning_data(&self) -> &Arc<SkinningData> {
        &self.data
    }

    fn get(&self) -> Result<sys::CNA_AnimationPlayerHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput(
                "the animation player has been released",
            ));
        }
        Ok(handle)
    }

    /// Starts a clip by name, from the beginning.
    pub fn start_clip(&self, name: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the name is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.animation_player_start_clip)(handle, view(name)) })
    }

    /// Advances or seeks the current clip and recomputes every transform.
    ///
    /// `relative_to_current_time` chooses between the two: `true` adds to the
    /// current position, `false` seeks to it.
    pub fn update(
        &self,
        time_seconds: f64,
        relative_to_current_time: bool,
        loop_clip: bool,
    ) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.animation_player_update)(
                handle,
                time_seconds,
                u8::from(relative_to_current_time),
                u8::from(loop_clip),
            )
        })
    }

    /// Where in the clip the player is, in seconds.
    pub fn current_position(&self) -> Result<f64> {
        let handle = self.get()?;
        let mut value = 0.0_f64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.animation_player_get_current_position)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The shape of the clip being played, or `None` before one is started.
    pub fn current_clip(&self) -> Result<Option<ClipInfo>> {
        let handle = self.get()?;
        let mut has_clip = 0_u8;
        let mut duration = 0.0_f64;
        let mut tracks = 0_u64;
        // SAFETY: the handle is owned and all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.engine.animation_player_get_current_clip_info)(
                handle,
                &mut has_clip,
                &mut duration,
                &mut tracks,
            )
        })?;
        Ok((has_clip != 0).then_some(ClipInfo {
            duration_seconds: duration,
            track_count: tracks,
        }))
    }

    /// The name of the clip being played, empty before one is started.
    pub fn current_clip_name(&self) -> Result<String> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .animation_player_get_current_clip_name_byte_count)(
                handle, &mut required
            )
        })?;
        read_text(required, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            unsafe {
                (self.native.engine.animation_player_copy_current_clip_name)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
        .and_then(|text| text.ok_or(CnaError::InvalidInput("CNA text is not valid UTF-8")))
    }

    /// Each bone's local pose.
    pub fn bone_transforms(&self) -> Result<Vec<Matrix>> {
        self.transforms(self.native.engine.animation_player_copy_bone_transforms)
    }

    /// Those poses composed down the hierarchy.
    pub fn world_transforms(&self) -> Result<Vec<Matrix>> {
        self.transforms(self.native.engine.animation_player_copy_world_transforms)
    }

    /// The world transforms times the inverse bind pose: what a shader wants.
    pub fn skin_transforms(&self) -> Result<Vec<Matrix>> {
        self.transforms(self.native.engine.animation_player_copy_skin_transforms)
    }

    fn transforms(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_Handle,
            *mut sys::CNA_Matrix,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<Vec<Matrix>> {
        let handle = self.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe { route(handle, core::ptr::null_mut(), 0, &mut required) };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        if required == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_Matrix::default(); required as usize];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `required`
        // writable matrices.
        self.native
            .check(unsafe { route(handle, buffer.as_mut_ptr(), required, &mut count) })?;
        buffer.truncate(count as usize);
        Ok(buffer.into_iter().map(from_matrix).collect())
    }

    /// Releases the player now rather than at drop.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *guard;
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here.
        self.native
            .check(unsafe { (self.native.engine.animation_player_destroy)(handle) })?;
        *guard = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for AnimationPlayer {
    fn drop(&mut self) {
        // The player first, then the data it holds: upstream retains the data
        // for as long as a player reads it, and releasing the data first is
        // refused.
        let _ = self.release();
    }
}
