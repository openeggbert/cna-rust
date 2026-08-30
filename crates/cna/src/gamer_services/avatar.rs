//! XNA's Avatar object model.
//!
//! # Ownership
//!
//! | Handle | Policy | Why |
//! |---|---|---|
//! | `AvatarDescription` | owned | every `cna_avatar_description_create*` answers an owned handle |
//! | `AvatarAnimation` | owned | `cna_avatar_animation_create` answers an owned handle |
//! | `AvatarRenderer` | owned | `cna_avatar_renderer_create` answers an owned handle |
//!
//! A renderer takes a description at construction and CNA copies what it needs,
//! so the renderer does not retain the description handle and disposing the
//! description does not invalidate the renderer.
//!
//! # What HEADLESS can and cannot do
//!
//! Structure and state are real here: a description validates its bytes, an
//! animation advances its clock and answers bone transforms, and a renderer
//! holds transforms, lighting and a bind pose. Visible output is a different
//! question. A renderer only draws when a real renderer and a model have been
//! given to it through CNA's own `enable_real_rendering` route, which lives in
//! `cna::extensions::gamer_services` because XNA has no such member. Without
//! it, `Draw` reaches CNA and CNA reports what it can do -- nothing here
//! pretends a frame was produced.

#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::disposal::Disposable;
use crate::extensions::events::EventHandler;
use crate::error::{CnaError, Result};
use crate::game::TimeSpan;
use crate::graphics::{from_native_matrix, native_matrix};
use crate::value::{Matrix, Vector3};

use super::async_result::{with_callback, GamerAsyncCallback, GamerAsyncResult, GamerAsyncState};
use super::core::{GamerServicesRuntime, OwnedHandle};
use super::gamer::{Gamer, GamerBase};
use super::values::{AvatarBodyType, AvatarExpression, AvatarRendererState};

/// XNA `Microsoft.Xna.Framework.GamerServices.IAvatarAnimation`.
///
/// The contract `AvatarRenderer.Draw` accepts, so a caller can supply its own
/// animation source rather than only CNA's presets.
pub trait IAvatarAnimation {
    /// XNA `IAvatarAnimation.Update`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Update(&self, elapsedAnimationTime: TimeSpan, r#loop: bool) -> Result<()>;

    /// XNA `IAvatarAnimation.Length`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Length(&self) -> Result<TimeSpan>;

    /// XNA `IAvatarAnimation.CurrentPosition`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn CurrentPosition(&self) -> Result<TimeSpan>;

    /// XNA `IAvatarAnimation.CurrentPosition` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn SetCurrentPosition(&self, value: TimeSpan) -> Result<()>;

    /// XNA `IAvatarAnimation.BoneTransforms`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn BoneTransforms(&self) -> Result<Vec<Matrix>>;

    /// XNA `IAvatarAnimation.Expression`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Expression(&self) -> Result<AvatarExpression>;
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarDescription`.
#[derive(Clone, Debug)]
pub struct AvatarDescription {
    owner: Arc<OwnedHandle>,
}

impl AvatarDescription {
    fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.avatar_description_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    /// XNA `AvatarDescription(byte[])`.
    ///
    /// CNA copies the bytes during the call and validates them: a description
    /// it cannot read answers `IsValid() == false` rather than being rejected,
    /// which is XNA's own behaviour for a corrupt description.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn new(data: &[u8]) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let count = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("the avatar description is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr()
        };
        let mut handle = 0;
        // SAFETY: the slice describes exactly `count` readable bytes, copied
        // during the call.
        runtime.check(unsafe {
            (runtime.native().gamer_services.avatar_description_create)(
                pointer,
                count,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `AvatarDescription.CreateRandom()`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CreateRandom() -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let mut handle = 0;
        // SAFETY: the output receives an owned handle.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .avatar_description_create_random)(&mut handle)
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `AvatarDescription.CreateRandom(bodyType)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CreateRandomWithBodyType(bodyType: AvatarBodyType) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let mut handle = 0;
        // SAFETY: the body-type identity is a plain scalar.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .avatar_description_create_random_for_body_type)(
                bodyType as u32, &mut handle
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `AvatarDescription.BeginGetFromGamer`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including its refusal on a host
    /// with no avatar service.
    pub fn BeginGetFromGamer(
        gamer: &Gamer,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let handle = gamer.handle_for_guide()?;
        let route = runtime
            .native()
            .gamer_services
            .avatar_description_get_from_gamer;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(state, callback, |trampoline, context| {
            let mut description = 0;
            // SAFETY: the gamer handle is live and the context outlives the call.
            runtime
                .check(unsafe { route(handle, trampoline, context, &mut description) })?;
            Ok(Self::adopt(adopted, description))
        })?;
        Ok(result)
    }

    /// XNA `AvatarDescription.EndGetFromGamer`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndGetFromGamer(result: &GamerAsyncResult) -> Result<Self> {
        result.end_once::<Self>()
    }

    fn info(&self) -> Result<sys::CNA_AvatarDescriptionInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_AvatarDescriptionInfo {
            struct_size: core::mem::size_of::<sys::CNA_AvatarDescriptionInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_AvatarDescriptionInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_description_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `AvatarDescription.IsValid`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsValid(&self) -> Result<bool> {
        Ok(self.info()?.is_valid != 0)
    }

    /// XNA `AvatarDescription.Height`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Height(&self) -> Result<f32> {
        Ok(self.info()?.height)
    }

    /// XNA `AvatarDescription.BodyType`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a body type XNA does not declare.
    pub fn BodyType(&self) -> Result<AvatarBodyType> {
        let raw = self.info()?.body_type;
        AvatarBodyType::from_native(raw).ok_or(CnaError::InvalidInput(
            "CNA reported an avatar body type XNA does not declare",
        ))
    }

    /// XNA `AvatarDescription.Description`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Description(&self) -> Result<Vec<u8>> {
        let handle = self.owner.get()?;
        let bytes = self.info()?.description_byte_count;
        let capacity = usize::try_from(bytes)
            .map_err(|_| CnaError::InvalidInput("the avatar description is too large"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: the destination has exactly the reported capacity.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_description_copy_description)(
                handle,
                buffer.as_mut_ptr(),
                bytes,
                &mut written,
            )
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("the avatar description is too large"))?;
        buffer.truncate(written.min(capacity));
        Ok(buffer)
    }

    /// XNA `AvatarDescription.Changed` subscription.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[must_use]
    pub fn AddChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        super::events::add_avatar_description_changed(handler).unwrap_or(0)
    }

    /// XNA `AvatarDescription.Changed` removal.
    #[must_use]
    pub fn RemoveChangedHandler(&self, registration: u64) -> bool {
        super::events::remove_avatar_description_changed(registration).unwrap_or(false)
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.get()
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarAnimation`.
#[derive(Debug)]
pub struct AvatarAnimation {
    owner: Arc<OwnedHandle>,
}

impl AvatarAnimation {
    /// XNA `AvatarAnimation(preset)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn new(animationPreset: super::values::AvatarAnimationPreset) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let mut handle = 0;
        // SAFETY: the preset identity is a plain scalar.
        runtime.check(unsafe {
            (runtime.native().gamer_services.avatar_animation_create)(
                animationPreset as u32,
                &mut handle,
            )
        })?;
        let destroy = runtime.native().gamer_services.avatar_animation_destroy;
        Ok(Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        })
    }

    fn info(&self) -> Result<sys::CNA_AvatarAnimationInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_AvatarAnimationInfo {
            struct_size: core::mem::size_of::<sys::CNA_AvatarAnimationInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_AvatarAnimationInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_animation_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `AvatarAnimation.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        Ok(self.info()?.is_disposed != 0)
    }

    /// XNA `AvatarAnimation.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.owner.release()
    }

    /// XNA `AvatarAnimation.Dispose(bool)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DisposeWithDisposing(&self, disposing: bool) -> Result<()> {
        let _ = disposing;
        self.owner.release()
    }

    /// XNA `AvatarAnimation.Finalize`.
    #[allow(clippy::unused_self)]
    pub fn Finalize(&self) {}

    /// XNA `AvatarAnimation.Update`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Update(&self, elapsedAnimationTime: TimeSpan, r#loop: bool) -> Result<()> {
        IAvatarAnimation::Update(self, elapsedAnimationTime, r#loop)
    }

    /// XNA `AvatarAnimation.Length`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Length(&self) -> Result<TimeSpan> {
        IAvatarAnimation::Length(self)
    }

    /// XNA `AvatarAnimation.CurrentPosition`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CurrentPosition(&self) -> Result<TimeSpan> {
        IAvatarAnimation::CurrentPosition(self)
    }

    /// XNA `AvatarAnimation.CurrentPosition` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetCurrentPosition(&self, value: TimeSpan) -> Result<()> {
        IAvatarAnimation::SetCurrentPosition(self, value)
    }

    /// XNA `AvatarAnimation.BoneTransforms`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BoneTransforms(&self) -> Result<Vec<Matrix>> {
        IAvatarAnimation::BoneTransforms(self)
    }

    /// XNA `AvatarAnimation.Expression`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Expression(&self) -> Result<AvatarExpression> {
        IAvatarAnimation::Expression(self)
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.get()
    }
}

impl IAvatarAnimation for AvatarAnimation {
    fn Update(&self, elapsedAnimationTime: TimeSpan, r#loop: bool) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and both arguments are plain scalars.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_animation_update)(
                handle,
                elapsedAnimationTime.Ticks(),
                u8::from(r#loop).into(),
            )
        })
    }

    fn Length(&self) -> Result<TimeSpan> {
        Ok(TimeSpan::from_ticks(self.info()?.length_ticks))
    }

    fn CurrentPosition(&self) -> Result<TimeSpan> {
        Ok(TimeSpan::from_ticks(self.info()?.current_position_ticks))
    }

    fn SetCurrentPosition(&self, value: TimeSpan) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the position is a tick count.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_animation_set_current_position)(handle, value.Ticks())
        })
    }

    fn BoneTransforms(&self) -> Result<Vec<Matrix>> {
        let handle = self.owner.get()?;
        let count = self.info()?.bone_transform_count;
        let mut transforms = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let mut value = sys::CNA_Matrix::default();
            // SAFETY: the index is inside the reported transform count.
            self.owner.check(unsafe {
                (self
                    .owner
                    .native()
                    .gamer_services
                    .avatar_animation_get_bone_transform_at)(handle, index, &mut value)
            })?;
            transforms.push(from_native_matrix(value));
        }
        Ok(transforms)
    }

    fn Expression(&self) -> Result<AvatarExpression> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_AvatarExpression {
            struct_size: core::mem::size_of::<sys::CNA_AvatarExpression>() as u32,
            struct_version: 1,
            ..sys::CNA_AvatarExpression::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_animation_get_expression)(handle, &mut value)
        })?;
        native_expression(&value)
    }
}

impl Disposable for AvatarAnimation {
    fn Dispose(&mut self) {
        let _ = AvatarAnimation::Dispose(&*self);
    }
}

impl Drop for AvatarAnimation {
    fn drop(&mut self) {
        let _ = self.owner.release();
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarRenderer`.
#[derive(Debug)]
pub struct AvatarRenderer {
    owner: Arc<OwnedHandle>,
    /// `ParentBones` is fixed by the avatar skeleton for a renderer's life,
    /// so it is walked once and kept. `BindPose` is not cached: it depends on
    /// the renderer's state, and caching its refusal would hide that state
    /// changing on a runtime that can reach `Ready`.
    fixed: Mutex<Option<Vec<i32>>>,
}

impl AvatarRenderer {
    /// XNA `AvatarRenderer.BoneCount`. Casing intentionally follows XNA.
    #[allow(non_upper_case_globals)]
    pub const BoneCount: i32 = 71;

    /// XNA `AvatarRenderer(avatarDescription)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn new(avatarDescription: &AvatarDescription) -> Result<Self> {
        Self::create(avatarDescription, false)
    }

    /// XNA `AvatarRenderer(avatarDescription, useLoadingEffect)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn from_avatar_description_and_use_loading_effect(
        avatarDescription: &AvatarDescription,
        useLoadingEffect: bool,
    ) -> Result<Self> {
        Self::create(avatarDescription, useLoadingEffect)
    }

    fn create(description: &AvatarDescription, use_loading_effect: bool) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let handle = description.handle()?;
        let mut renderer = 0;
        // SAFETY: the description handle is live; CNA copies what it needs, so
        // the renderer does not retain it.
        runtime.check(unsafe {
            (runtime.native().gamer_services.avatar_renderer_create)(
                handle,
                u8::from(use_loading_effect).into(),
                &mut renderer,
            )
        })?;
        let destroy = runtime.native().gamer_services.avatar_renderer_destroy;
        Ok(Self {
            owner: OwnedHandle::new(runtime, renderer, destroy),
            fixed: Mutex::new(None),
        })
    }

    fn info(&self) -> Result<sys::CNA_AvatarRendererInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_AvatarRendererInfo {
            struct_size: core::mem::size_of::<sys::CNA_AvatarRendererInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_AvatarRendererInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_renderer_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    fn transforms(&self) -> Result<(Matrix, Matrix, Matrix)> {
        let handle = self.owner.get()?;
        let (mut world, mut view, mut projection) = (
            sys::CNA_Matrix::default(),
            sys::CNA_Matrix::default(),
            sys::CNA_Matrix::default(),
        );
        // SAFETY: the handle is live and all three outputs are initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_renderer_get_transforms)(
                handle, &mut world, &mut view, &mut projection
            )
        })?;
        Ok((
            from_native_matrix(world),
            from_native_matrix(view),
            from_native_matrix(projection),
        ))
    }

    fn set_transforms(&self, world: Matrix, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.owner.get()?;
        let (world, view, projection) = (
            native_matrix(world),
            native_matrix(view),
            native_matrix(projection),
        );
        // SAFETY: the handle is live and all three inputs outlive the call.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_renderer_set_transforms)(handle, &world, &view, &projection)
        })
    }

    fn lighting(&self) -> Result<(Vector3, Vector3, Vector3)> {
        let handle = self.owner.get()?;
        let (mut color, mut direction, mut ambient) = (
            sys::CNA_Vector3::default(),
            sys::CNA_Vector3::default(),
            sys::CNA_Vector3::default(),
        );
        // SAFETY: the handle is live and all three outputs are initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_renderer_get_lighting)(
                handle,
                &mut color,
                &mut direction,
                &mut ambient,
            )
        })?;
        Ok((
            Vector3::from_x_and_y_and_z(color.x, color.y, color.z),
            Vector3::from_x_and_y_and_z(direction.x, direction.y, direction.z),
            Vector3::from_x_and_y_and_z(ambient.x, ambient.y, ambient.z),
        ))
    }

    fn set_lighting(&self, color: Vector3, direction: Vector3, ambient: Vector3) -> Result<()> {
        let handle = self.owner.get()?;
        let to_native = |value: Vector3| sys::CNA_Vector3 {
            x: value.X,
            y: value.Y,
            z: value.Z,
        };
        let (color, direction, ambient) = (to_native(color), to_native(direction), to_native(ambient));
        // SAFETY: the handle is live and all three inputs outlive the call.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_renderer_set_lighting)(
                handle,
                &color,
                &direction,
                &ambient,
            )
        })
    }

    /// XNA `AvatarRenderer.World`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn World(&self) -> Result<Matrix> {
        Ok(self.transforms()?.0)
    }

    /// XNA `AvatarRenderer.World` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetWorld(&self, value: Matrix) -> Result<()> {
        let (_, view, projection) = self.transforms()?;
        self.set_transforms(value, view, projection)
    }

    /// XNA `AvatarRenderer.View`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn View(&self) -> Result<Matrix> {
        Ok(self.transforms()?.1)
    }

    /// XNA `AvatarRenderer.View` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetView(&self, value: Matrix) -> Result<()> {
        let (world, _, projection) = self.transforms()?;
        self.set_transforms(world, value, projection)
    }

    /// XNA `AvatarRenderer.Projection`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Projection(&self) -> Result<Matrix> {
        Ok(self.transforms()?.2)
    }

    /// XNA `AvatarRenderer.Projection` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetProjection(&self, value: Matrix) -> Result<()> {
        let (world, view, _) = self.transforms()?;
        self.set_transforms(world, view, value)
    }

    /// XNA `AvatarRenderer.LightColor`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn LightColor(&self) -> Result<Vector3> {
        Ok(self.lighting()?.0)
    }

    /// XNA `AvatarRenderer.LightColor` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetLightColor(&self, value: Vector3) -> Result<()> {
        let (_, direction, ambient) = self.lighting()?;
        self.set_lighting(value, direction, ambient)
    }

    /// XNA `AvatarRenderer.LightDirection`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn LightDirection(&self) -> Result<Vector3> {
        Ok(self.lighting()?.1)
    }

    /// XNA `AvatarRenderer.LightDirection` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetLightDirection(&self, value: Vector3) -> Result<()> {
        let (color, _, ambient) = self.lighting()?;
        self.set_lighting(color, value, ambient)
    }

    /// XNA `AvatarRenderer.AmbientLightColor`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AmbientLightColor(&self) -> Result<Vector3> {
        Ok(self.lighting()?.2)
    }

    /// XNA `AvatarRenderer.AmbientLightColor` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetAmbientLightColor(&self, value: Vector3) -> Result<()> {
        let (color, direction, _) = self.lighting()?;
        self.set_lighting(color, direction, value)
    }

    /// XNA `AvatarRenderer.State`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a state XNA does not declare.
    pub fn State(&self) -> Result<AvatarRendererState> {
        let raw = self.info()?.state;
        AvatarRendererState::from_native(raw).ok_or(CnaError::InvalidInput(
            "CNA reported an avatar renderer state XNA does not declare",
        ))
    }

    /// XNA `AvatarRenderer.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        Ok(self.info()?.is_disposed != 0)
    }

    /// XNA `AvatarRenderer.ParentBones`.
    ///
    /// Always readable: the parent index of every bone is fixed by the avatar
    /// skeleton and does not depend on the renderer reaching `Ready`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ParentBones(&self) -> Result<Vec<i32>> {
        let mut cached = self
            .fixed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = cached.clone() {
            return Ok(existing);
        }
        let handle = self.owner.get()?;
        let mut parents = Vec::with_capacity(Self::BoneCount as usize);
        for index in 0..Self::BoneCount {
            let mut parent = 0;
            // SAFETY: the index is inside the canonical bone count.
            self.owner.check(unsafe {
                (self
                    .owner
                    .native()
                    .gamer_services
                    .avatar_renderer_get_parent_bone_at)(handle, index, &mut parent)
            })?;
            parents.push(parent);
        }
        *cached = Some(parents.clone());
        Ok(parents)
    }

    /// XNA `AvatarRenderer.BindPose`.
    ///
    /// XNA raises `InvalidOperationException` unless the renderer has reached
    /// `Ready`, and nothing in this runtime ever sets that state, so on this
    /// host the honest answer is CNA's refusal. It is read fresh each time
    /// rather than cached, because a cached refusal would hide the state
    /// changing on a runtime that can reach `Ready`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BindPose(&self) -> Result<Vec<Matrix>> {
        let handle = self.owner.get()?;
        let mut pose = Vec::with_capacity(Self::BoneCount as usize);
        for index in 0..Self::BoneCount {
            let mut transform = sys::CNA_Matrix::default();
            // SAFETY: the index is inside the canonical bone count.
            self.owner.check(unsafe {
                (self
                    .owner
                    .native()
                    .gamer_services
                    .avatar_renderer_get_bind_pose_at)(handle, index, &mut transform)
            })?;
            pose.push(from_native_matrix(transform));
        }
        Ok(pose)
    }

    /// XNA `AvatarRenderer.Draw(animation)`.
    ///
    /// Reaches CNA's draw route. Whether anything appears is the renderer's
    /// question, not this projection's: a build with no real avatar renderer
    /// reports that rather than silently succeeding.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Draw(&self, animation: &AvatarAnimation) -> Result<()> {
        let handle = self.owner.get()?;
        let clip = animation.handle()?;
        // SAFETY: both handles are live for the call.
        self.owner.check(unsafe {
            (self
                .owner
                .native()
                .gamer_services
                .avatar_renderer_draw_animation)(handle, clip)
        })
    }

    /// XNA `AvatarRenderer.Draw(bones, expression)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DrawWithBonesAndExpression(
        &self,
        bones: &[Matrix],
        expression: AvatarExpression,
    ) -> Result<()> {
        let handle = self.owner.get()?;
        let native: Vec<sys::CNA_Matrix> = bones.iter().copied().map(native_matrix).collect();
        let count = u64::try_from(native.len())
            .map_err(|_| CnaError::InvalidInput("the bone array is too large"))?;
        let pointer = if native.is_empty() {
            core::ptr::null()
        } else {
            native.as_ptr()
        };
        let expression = expression_to_native(expression);
        // SAFETY: the array describes exactly `count` matrices and the
        // expression descriptor outlives the call.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.avatar_renderer_draw_bones)(
                handle,
                pointer,
                count,
                &expression,
            )
        })
    }

    /// XNA `AvatarRenderer.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.owner.release()
    }

    /// XNA `AvatarRenderer.Dispose(bool)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DisposeWithDisposing(&self, disposing: bool) -> Result<()> {
        let _ = disposing;
        self.owner.release()
    }

    /// XNA `AvatarRenderer.Finalize`.
    #[allow(clippy::unused_self)]
    pub fn Finalize(&self) {}
}

impl Disposable for AvatarRenderer {
    fn Dispose(&mut self) {
        let _ = AvatarRenderer::Dispose(&*self);
    }
}

impl Drop for AvatarRenderer {
    fn drop(&mut self) {
        let _ = self.owner.release();
    }
}

pub(crate) fn expression_to_native(value: AvatarExpression) -> sys::CNA_AvatarExpression {
    sys::CNA_AvatarExpression {
        struct_size: core::mem::size_of::<sys::CNA_AvatarExpression>() as u32,
        struct_version: 1,
        mouth: value.Mouth() as u32,
        left_eye: value.LeftEye() as u32,
        right_eye: value.RightEye() as u32,
        left_eyebrow: value.LeftEyebrow() as u32,
        right_eyebrow: value.RightEyebrow() as u32,
    }
}

fn native_expression(value: &sys::CNA_AvatarExpression) -> Result<AvatarExpression> {
    let unknown = || CnaError::InvalidInput("CNA reported an avatar expression XNA does not declare");
    let mut expression = AvatarExpression::default();
    expression.SetMouth(super::values::AvatarMouth::from_native(value.mouth).ok_or_else(unknown)?);
    expression.SetLeftEye(super::values::AvatarEye::from_native(value.left_eye).ok_or_else(unknown)?);
    expression
        .SetRightEye(super::values::AvatarEye::from_native(value.right_eye).ok_or_else(unknown)?);
    expression.SetLeftEyebrow(
        super::values::AvatarEyebrow::from_native(value.left_eyebrow).ok_or_else(unknown)?,
    );
    expression.SetRightEyebrow(
        super::values::AvatarEyebrow::from_native(value.right_eyebrow).ok_or_else(unknown)?,
    );
    Ok(expression)
}
