#![allow(non_snake_case, non_upper_case_globals, clippy::missing_errors_doc)]

use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::Result;
use crate::extensions::events::EventHandler;
use crate::native::{
    BasicBoolProperty, BasicFloatProperty, BasicVector3Property, StockEffectKind,
    StockMatrixProperty,
};
use crate::value::{Matrix, Vector3};

use super::effect::{from_native_matrix, from_native_vector3, native_matrix, native_vector3};
use super::resource::ResourceState;
use super::{
    CompareFunction, Effect, EffectBase, EffectParameter, GraphicsDevice, GraphicsResource,
    Texture2D, TextureCube,
};

/// XNA fog contract shared by stock effects.
pub trait IEffectFog {
    fn FogColor(&self) -> Result<Vector3>;
    fn SetFogColor(&mut self, value: Vector3) -> Result<()>;
    fn FogEnabled(&self) -> Result<bool>;
    fn SetFogEnabled(&mut self, value: bool) -> Result<()>;
    fn FogStart(&self) -> Result<f32>;
    fn SetFogStart(&mut self, value: f32) -> Result<()>;
    fn FogEnd(&self) -> Result<f32>;
    fn SetFogEnd(&mut self, value: f32) -> Result<()>;
}

/// XNA matrix contract shared by stock effects.
pub trait IEffectMatrices {
    fn World(&self) -> Result<Matrix>;
    fn SetWorld(&mut self, value: Matrix) -> Result<()>;
    fn View(&self) -> Result<Matrix>;
    fn SetView(&mut self, value: Matrix) -> Result<()>;
    fn Projection(&self) -> Result<Matrix>;
    fn SetProjection(&mut self, value: Matrix) -> Result<()>;
}

/// XNA lighting contract shared by stock effects.
pub trait IEffectLights {
    fn AmbientLightColor(&self) -> Result<Vector3>;
    fn SetAmbientLightColor(&mut self, value: Vector3) -> Result<()>;
    fn DirectionalLight0(&self) -> Result<&DirectionalLight>;
    fn DirectionalLight1(&self) -> Result<&DirectionalLight>;
    fn DirectionalLight2(&self) -> Result<&DirectionalLight>;
    fn LightingEnabled(&self) -> Result<bool>;
    fn SetLightingEnabled(&mut self, value: bool) -> Result<()>;
    fn EnableDefaultLighting(&self) -> Result<()>;
}

/// Stable child façade for one native stock-effect directional light.
pub struct DirectionalLight {
    owner: Arc<ResourceState>,
    handle: Mutex<sys::CNA_DirectionalLightHandle>,
}

impl DirectionalLight {
    pub fn new(
        directionParameter: &EffectParameter,
        diffuseColorParameter: &EffectParameter,
        specularColorParameter: &EffectParameter,
        cloneSource: &Self,
    ) -> Result<Self> {
        // These parameter accesses preserve XNA's requirement that the supplied
        // parameter views are live before the new light is assembled.
        directionParameter.Name()?;
        diffuseColorParameter.Name()?;
        specularColorParameter.Name()?;
        let native = cloneSource.owner.device().state.native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        native.create_directional_light(&mut handle)?;
        let value = Self {
            owner: Arc::clone(&cloneSource.owner),
            handle: Mutex::new(handle),
        };
        value.SetDirection(cloneSource.Direction()?)?;
        value.SetDiffuseColor(cloneSource.DiffuseColor()?)?;
        value.SetSpecularColor(cloneSource.SpecularColor()?)?;
        value.SetEnabled(cloneSource.Enabled()?)?;
        Ok(value)
    }

    fn from_effect(effect: &Effect, index: u32) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        effect
            .state_arc()
            .device()
            .state
            .native()
            .effect_directional_light(effect.handle()?, index, &mut handle)?;
        Ok(Self {
            owner: effect.state_arc(),
            handle: Mutex::new(handle),
        })
    }

    fn handle(&self) -> Result<sys::CNA_DirectionalLightHandle> {
        self.owner.require_handle()?;
        Ok(*self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
    }

    pub fn Direction(&self) -> Result<Vector3> {
        self.vector3(0)
    }
    pub fn SetDirection(&self, value: Vector3) -> Result<()> {
        self.set_vector3(0, value)
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.vector3(1)
    }
    pub fn SetDiffuseColor(&self, value: Vector3) -> Result<()> {
        self.set_vector3(1, value)
    }
    pub fn SpecularColor(&self) -> Result<Vector3> {
        self.vector3(2)
    }
    pub fn SetSpecularColor(&self, value: Vector3) -> Result<()> {
        self.set_vector3(2, value)
    }
    pub fn Enabled(&self) -> Result<bool> {
        self.owner
            .device()
            .state
            .native()
            .directional_light_enabled(self.handle()?)
    }
    pub fn SetEnabled(&self, value: bool) -> Result<()> {
        self.owner
            .device()
            .state
            .native()
            .set_directional_light_enabled(self.handle()?, value)
    }

    fn vector3(&self, property: u8) -> Result<Vector3> {
        self.owner
            .device()
            .state
            .native()
            .directional_light_vector3(self.handle()?, property)
            .map(from_native_vector3)
    }

    fn set_vector3(&self, property: u8, value: Vector3) -> Result<()> {
        self.owner
            .device()
            .state
            .native()
            .set_directional_light_vector3(self.handle()?, property, native_vector3(value))
    }

    fn dispose_native(&self) -> Result<()> {
        let mut handle = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        self.owner
            .device()
            .state
            .native()
            .destroy_directional_light(*handle)?;
        *handle = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for DirectionalLight {
    fn drop(&mut self) {
        let _ = self.dispose_native();
    }
}

fn dispose_lights(lights: &[DirectionalLight; 3]) -> Result<()> {
    let mut first_error = None;
    for light in lights {
        if let Err(error) = light.dispose_native() {
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// CNA-backed XNA `BasicEffect`, including its stable child lights.
pub struct BasicEffect {
    lights: [DirectionalLight; 3],
    effect: Effect,
    texture: Mutex<Option<Arc<Texture2D>>>,
}

impl BasicEffect {
    pub fn from_device(device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device
            .state
            .native()
            .create_basic_effect(device.handle()?, &mut handle)?;
        let effect = Effect::from_handle(device, handle);
        Self::from_effect(effect, None)
    }

    pub fn new(cloneSource: &Self) -> Result<Self> {
        let effect = cloneSource.effect.Clone()?;
        let texture = cloneSource
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        Self::from_effect(effect, texture)
    }

    fn from_effect(effect: Effect, texture: Option<Arc<Texture2D>>) -> Result<Self> {
        let light0 = DirectionalLight::from_effect(&effect, 0)?;
        let light1 = DirectionalLight::from_effect(&effect, 1)?;
        let light2 = DirectionalLight::from_effect(&effect, 2)?;
        Ok(Self {
            lights: [light0, light1, light2],
            effect,
            texture: Mutex::new(texture),
        })
    }

    fn native(&self) -> &crate::native::Native {
        self.effect.native()
    }

    fn vector3(&self, property: BasicVector3Property) -> Result<Vector3> {
        self.native()
            .basic_vector3(self.effect.handle()?, property)
            .map(from_native_vector3)
    }

    fn set_vector3(&self, property: BasicVector3Property, value: Vector3) -> Result<()> {
        self.native()
            .set_basic_vector3(self.effect.handle()?, property, native_vector3(value))
    }

    fn boolean(&self, property: BasicBoolProperty) -> Result<bool> {
        self.native().basic_bool(self.effect.handle()?, property)
    }

    fn set_boolean(&self, property: BasicBoolProperty, value: bool) -> Result<()> {
        self.native()
            .set_basic_bool(self.effect.handle()?, property, value)
    }

    fn float(&self, property: BasicFloatProperty) -> Result<f32> {
        self.native().basic_float(self.effect.handle()?, property)
    }

    fn set_float(&self, property: BasicFloatProperty, value: f32) -> Result<()> {
        self.native()
            .set_basic_float(self.effect.handle()?, property, value)
    }

    pub fn Alpha(&self) -> Result<f32> {
        self.float(BasicFloatProperty::Alpha)
    }
    pub fn SetAlpha(&mut self, value: f32) -> Result<()> {
        self.set_float(BasicFloatProperty::Alpha, value)
    }
    pub fn AmbientLightColor(&self) -> Result<Vector3> {
        self.vector3(BasicVector3Property::AmbientLightColor)
    }
    pub fn SetAmbientLightColor(&mut self, value: Vector3) -> Result<()> {
        self.set_vector3(BasicVector3Property::AmbientLightColor, value)
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.vector3(BasicVector3Property::DiffuseColor)
    }
    pub fn SetDiffuseColor(&mut self, value: Vector3) -> Result<()> {
        self.set_vector3(BasicVector3Property::DiffuseColor, value)
    }
    pub fn EmissiveColor(&self) -> Result<Vector3> {
        self.vector3(BasicVector3Property::EmissiveColor)
    }
    pub fn SetEmissiveColor(&mut self, value: Vector3) -> Result<()> {
        self.set_vector3(BasicVector3Property::EmissiveColor, value)
    }
    pub fn SpecularColor(&self) -> Result<Vector3> {
        self.vector3(BasicVector3Property::SpecularColor)
    }
    pub fn SetSpecularColor(&mut self, value: Vector3) -> Result<()> {
        self.set_vector3(BasicVector3Property::SpecularColor, value)
    }
    pub fn SpecularPower(&self) -> Result<f32> {
        self.float(BasicFloatProperty::SpecularPower)
    }
    pub fn SetSpecularPower(&mut self, value: f32) -> Result<()> {
        self.set_float(BasicFloatProperty::SpecularPower, value)
    }
    pub fn FogColor(&self) -> Result<Vector3> {
        self.vector3(BasicVector3Property::FogColor)
    }
    pub fn SetFogColor(&mut self, value: Vector3) -> Result<()> {
        self.set_vector3(BasicVector3Property::FogColor, value)
    }
    pub fn FogEnabled(&self) -> Result<bool> {
        self.boolean(BasicBoolProperty::FogEnabled)
    }
    pub fn SetFogEnabled(&mut self, value: bool) -> Result<()> {
        self.set_boolean(BasicBoolProperty::FogEnabled, value)
    }
    pub fn FogStart(&self) -> Result<f32> {
        self.float(BasicFloatProperty::FogStart)
    }
    pub fn SetFogStart(&mut self, value: f32) -> Result<()> {
        self.set_float(BasicFloatProperty::FogStart, value)
    }
    pub fn FogEnd(&self) -> Result<f32> {
        self.float(BasicFloatProperty::FogEnd)
    }
    pub fn SetFogEnd(&mut self, value: f32) -> Result<()> {
        self.set_float(BasicFloatProperty::FogEnd, value)
    }
    pub fn LightingEnabled(&self) -> Result<bool> {
        self.boolean(BasicBoolProperty::LightingEnabled)
    }
    pub fn SetLightingEnabled(&mut self, value: bool) -> Result<()> {
        self.set_boolean(BasicBoolProperty::LightingEnabled, value)
    }
    pub fn PreferPerPixelLighting(&self) -> Result<bool> {
        self.boolean(BasicBoolProperty::PreferPerPixelLighting)
    }
    pub fn SetPreferPerPixelLighting(&mut self, value: bool) -> Result<()> {
        self.set_boolean(BasicBoolProperty::PreferPerPixelLighting, value)
    }
    pub fn TextureEnabled(&self) -> Result<bool> {
        self.boolean(BasicBoolProperty::TextureEnabled)
    }
    pub fn SetTextureEnabled(&mut self, value: bool) -> Result<()> {
        self.set_boolean(BasicBoolProperty::TextureEnabled, value)
    }
    pub fn VertexColorEnabled(&self) -> Result<bool> {
        self.boolean(BasicBoolProperty::VertexColorEnabled)
    }
    pub fn SetVertexColorEnabled(&mut self, value: bool) -> Result<()> {
        self.set_boolean(BasicBoolProperty::VertexColorEnabled, value)
    }
    pub fn Texture(&self) -> Result<Option<Arc<Texture2D>>> {
        self.effect.handle()?;
        Ok(self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTexture(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        let handle = retained_texture_handle(&self.effect, value.as_ref())?;
        self.native()
            .basic_set_texture(self.effect.handle()?, handle)?;
        *self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn DirectionalLight0(&self) -> Result<&DirectionalLight> {
        self.effect.handle()?;
        Ok(&self.lights[0])
    }
    pub fn DirectionalLight1(&self) -> Result<&DirectionalLight> {
        self.effect.handle()?;
        Ok(&self.lights[1])
    }
    pub fn DirectionalLight2(&self) -> Result<&DirectionalLight> {
        self.effect.handle()?;
        Ok(&self.lights[2])
    }
    pub fn EnableDefaultLighting(&self) -> Result<()> {
        self.native().enable_default_lighting(self.effect.handle()?)
    }
    pub fn World(&self) -> Result<Matrix> {
        self.matrix(StockMatrixProperty::World)
    }
    pub fn SetWorld(&mut self, value: Matrix) -> Result<()> {
        self.set_matrix(StockMatrixProperty::World, value)
    }
    pub fn View(&self) -> Result<Matrix> {
        self.matrix(StockMatrixProperty::View)
    }
    pub fn SetView(&mut self, value: Matrix) -> Result<()> {
        self.set_matrix(StockMatrixProperty::View, value)
    }
    pub fn Projection(&self) -> Result<Matrix> {
        self.matrix(StockMatrixProperty::Projection)
    }
    pub fn SetProjection(&mut self, value: Matrix) -> Result<()> {
        self.set_matrix(StockMatrixProperty::Projection, value)
    }
    fn matrix(&self, property: StockMatrixProperty) -> Result<Matrix> {
        self.native()
            .stock_matrix(self.effect.handle()?, property)
            .map(from_native_matrix)
    }
    fn set_matrix(&self, property: StockMatrixProperty, value: Matrix) -> Result<()> {
        self.native()
            .set_stock_matrix(self.effect.handle()?, property, native_matrix(value))
    }
    pub fn Clone(&self) -> Result<Effect> {
        self.effect.Clone()
    }
    pub fn OnApply(&self) -> Result<()> {
        self.effect.OnApply()
    }
}

impl IEffectFog for BasicEffect {
    fn FogColor(&self) -> Result<Vector3> {
        Self::FogColor(self)
    }
    fn SetFogColor(&mut self, value: Vector3) -> Result<()> {
        Self::SetFogColor(self, value)
    }
    fn FogEnabled(&self) -> Result<bool> {
        Self::FogEnabled(self)
    }
    fn SetFogEnabled(&mut self, value: bool) -> Result<()> {
        Self::SetFogEnabled(self, value)
    }
    fn FogStart(&self) -> Result<f32> {
        Self::FogStart(self)
    }
    fn SetFogStart(&mut self, value: f32) -> Result<()> {
        Self::SetFogStart(self, value)
    }
    fn FogEnd(&self) -> Result<f32> {
        Self::FogEnd(self)
    }
    fn SetFogEnd(&mut self, value: f32) -> Result<()> {
        Self::SetFogEnd(self, value)
    }
}

impl IEffectMatrices for BasicEffect {
    fn World(&self) -> Result<Matrix> {
        Self::World(self)
    }
    fn SetWorld(&mut self, value: Matrix) -> Result<()> {
        Self::SetWorld(self, value)
    }
    fn View(&self) -> Result<Matrix> {
        Self::View(self)
    }
    fn SetView(&mut self, value: Matrix) -> Result<()> {
        Self::SetView(self, value)
    }
    fn Projection(&self) -> Result<Matrix> {
        Self::Projection(self)
    }
    fn SetProjection(&mut self, value: Matrix) -> Result<()> {
        Self::SetProjection(self, value)
    }
}

impl IEffectLights for BasicEffect {
    fn AmbientLightColor(&self) -> Result<Vector3> {
        Self::AmbientLightColor(self)
    }
    fn SetAmbientLightColor(&mut self, value: Vector3) -> Result<()> {
        Self::SetAmbientLightColor(self, value)
    }
    fn DirectionalLight0(&self) -> Result<&DirectionalLight> {
        Self::DirectionalLight0(self)
    }
    fn DirectionalLight1(&self) -> Result<&DirectionalLight> {
        Self::DirectionalLight1(self)
    }
    fn DirectionalLight2(&self) -> Result<&DirectionalLight> {
        Self::DirectionalLight2(self)
    }
    fn LightingEnabled(&self) -> Result<bool> {
        Self::LightingEnabled(self)
    }
    fn SetLightingEnabled(&mut self, value: bool) -> Result<()> {
        Self::SetLightingEnabled(self, value)
    }
    fn EnableDefaultLighting(&self) -> Result<()> {
        Self::EnableDefaultLighting(self)
    }
}

impl EffectBase for BasicEffect {
    fn AsEffect(&self) -> &Effect {
        &self.effect
    }

    fn set_model_matrices_for_model(
        &self,
        world: Matrix,
        view: Matrix,
        projection: Matrix,
    ) -> Result<()> {
        self.effect.set_model_matrices(world, view, projection)
    }
}

impl Deref for BasicEffect {
    type Target = Effect;
    fn deref(&self) -> &Self::Target {
        &self.effect
    }
}

impl GraphicsResource for BasicEffect {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        self.effect.GraphicsDevice()
    }
    fn IsDisposed(&self) -> bool {
        self.effect.IsDisposed()
    }
    fn Name(&self) -> String {
        self.effect.Name()
    }
    fn SetName(&mut self, value: &str) {
        self.effect.SetName(value);
    }
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.effect.Tag()
    }
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.effect.SetTag(value);
    }
    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.effect.AddDisposingHandler(handler)
    }
    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.effect.RemoveDisposingHandler(registration)
    }
    fn Dispose(&mut self, value: bool) -> Result<()> {
        let lights = dispose_lights(&self.lights);
        let effect = self.effect.Dispose(value);
        lights.and(effect)
    }
}

impl Drop for BasicEffect {
    fn drop(&mut self) {}
}

impl ContentDisposable for BasicEffect {
    fn DisposeContent(&self) -> Result<()> {
        let lights = dispose_lights(&self.lights);
        let effect = self.effect.state_arc().dispose_native();
        lights.and(effect)
    }
}

impl ContentLoadable for BasicEffect {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}

macro_rules! impl_fog_matrices {
    ($type:ty) => {
        impl $type {
            pub fn FogColor(&self) -> Result<Vector3> {
                self.effect
                    .native()
                    .basic_vector3(self.effect.handle()?, BasicVector3Property::FogColor)
                    .map(from_native_vector3)
            }
            pub fn SetFogColor(&mut self, value: Vector3) -> Result<()> {
                self.effect.native().set_basic_vector3(
                    self.effect.handle()?,
                    BasicVector3Property::FogColor,
                    native_vector3(value),
                )
            }
            pub fn FogEnabled(&self) -> Result<bool> {
                self.effect
                    .native()
                    .basic_bool(self.effect.handle()?, BasicBoolProperty::FogEnabled)
            }
            pub fn SetFogEnabled(&mut self, value: bool) -> Result<()> {
                self.effect.native().set_basic_bool(
                    self.effect.handle()?,
                    BasicBoolProperty::FogEnabled,
                    value,
                )
            }
            pub fn FogStart(&self) -> Result<f32> {
                self.effect
                    .native()
                    .basic_float(self.effect.handle()?, BasicFloatProperty::FogStart)
            }
            pub fn SetFogStart(&mut self, value: f32) -> Result<()> {
                self.effect.native().set_basic_float(
                    self.effect.handle()?,
                    BasicFloatProperty::FogStart,
                    value,
                )
            }
            pub fn FogEnd(&self) -> Result<f32> {
                self.effect
                    .native()
                    .basic_float(self.effect.handle()?, BasicFloatProperty::FogEnd)
            }
            pub fn SetFogEnd(&mut self, value: f32) -> Result<()> {
                self.effect.native().set_basic_float(
                    self.effect.handle()?,
                    BasicFloatProperty::FogEnd,
                    value,
                )
            }
            pub fn World(&self) -> Result<Matrix> {
                self.stock_matrix(StockMatrixProperty::World)
            }
            pub fn SetWorld(&mut self, value: Matrix) -> Result<()> {
                self.set_stock_matrix(StockMatrixProperty::World, value)
            }
            pub fn View(&self) -> Result<Matrix> {
                self.stock_matrix(StockMatrixProperty::View)
            }
            pub fn SetView(&mut self, value: Matrix) -> Result<()> {
                self.set_stock_matrix(StockMatrixProperty::View, value)
            }
            pub fn Projection(&self) -> Result<Matrix> {
                self.stock_matrix(StockMatrixProperty::Projection)
            }
            pub fn SetProjection(&mut self, value: Matrix) -> Result<()> {
                self.set_stock_matrix(StockMatrixProperty::Projection, value)
            }
            fn stock_matrix(&self, property: StockMatrixProperty) -> Result<Matrix> {
                self.effect
                    .native()
                    .stock_matrix(self.effect.handle()?, property)
                    .map(from_native_matrix)
            }
            fn set_stock_matrix(&self, property: StockMatrixProperty, value: Matrix) -> Result<()> {
                self.effect.native().set_stock_matrix(
                    self.effect.handle()?,
                    property,
                    native_matrix(value),
                )
            }
        }
        impl IEffectFog for $type {
            fn FogColor(&self) -> Result<Vector3> {
                Self::FogColor(self)
            }
            fn SetFogColor(&mut self, value: Vector3) -> Result<()> {
                Self::SetFogColor(self, value)
            }
            fn FogEnabled(&self) -> Result<bool> {
                Self::FogEnabled(self)
            }
            fn SetFogEnabled(&mut self, value: bool) -> Result<()> {
                Self::SetFogEnabled(self, value)
            }
            fn FogStart(&self) -> Result<f32> {
                Self::FogStart(self)
            }
            fn SetFogStart(&mut self, value: f32) -> Result<()> {
                Self::SetFogStart(self, value)
            }
            fn FogEnd(&self) -> Result<f32> {
                Self::FogEnd(self)
            }
            fn SetFogEnd(&mut self, value: f32) -> Result<()> {
                Self::SetFogEnd(self, value)
            }
        }
        impl IEffectMatrices for $type {
            fn World(&self) -> Result<Matrix> {
                Self::World(self)
            }
            fn SetWorld(&mut self, value: Matrix) -> Result<()> {
                Self::SetWorld(self, value)
            }
            fn View(&self) -> Result<Matrix> {
                Self::View(self)
            }
            fn SetView(&mut self, value: Matrix) -> Result<()> {
                Self::SetView(self, value)
            }
            fn Projection(&self) -> Result<Matrix> {
                Self::Projection(self)
            }
            fn SetProjection(&mut self, value: Matrix) -> Result<()> {
                Self::SetProjection(self, value)
            }
        }
    };
}

macro_rules! impl_effect_resource {
    ($type:ty) => {
        impl EffectBase for $type {
            fn AsEffect(&self) -> &Effect {
                &self.effect
            }

            fn set_model_matrices_for_model(
                &self,
                world: Matrix,
                view: Matrix,
                projection: Matrix,
            ) -> Result<()> {
                self.effect.set_model_matrices(world, view, projection)
            }
        }
        impl Deref for $type {
            type Target = Effect;
            fn deref(&self) -> &Effect {
                &self.effect
            }
        }
        impl GraphicsResource for $type {
            fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
                self.effect.GraphicsDevice()
            }
            fn IsDisposed(&self) -> bool {
                self.effect.IsDisposed()
            }
            fn Name(&self) -> String {
                self.effect.Name()
            }
            fn SetName(&mut self, value: &str) {
                self.effect.SetName(value);
            }
            fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
                self.effect.Tag()
            }
            fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
                self.effect.SetTag(value);
            }
            fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
                self.effect.AddDisposingHandler(handler)
            }
            fn RemoveDisposingHandler(&self, registration: u64) -> bool {
                self.effect.RemoveDisposingHandler(registration)
            }
            fn Dispose(&mut self, value: bool) -> Result<()> {
                let children = self.dispose_owned_children();
                let effect = self.effect.Dispose(value);
                children.and(effect)
            }
        }
        impl ContentDisposable for $type {
            fn DisposeContent(&self) -> Result<()> {
                let children = self.dispose_owned_children();
                let effect = self.effect.state_arc().dispose_native();
                children.and(effect)
            }
        }
        impl ContentLoadable for $type {
            fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
                Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
            }
        }
    };
}

fn retained_texture_handle(
    effect: &Effect,
    value: Option<&Arc<Texture2D>>,
) -> Result<sys::CNA_Handle> {
    value.map_or(Ok(sys::CNA_INVALID_HANDLE), |texture| {
        let device = texture
            .GraphicsDevice()
            .ok_or(crate::error::CnaError::InvalidInput(
                "texture has no graphics device",
            ))?;
        if !effect.is_same_device(device) {
            return Err(crate::error::CnaError::InvalidInput(
                "texture belongs to a different graphics device",
            ));
        }
        texture.handle()
    })
}

pub struct AlphaTestEffect {
    effect: Effect,
    texture: Mutex<Option<Arc<Texture2D>>>,
}

impl AlphaTestEffect {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn dispose_owned_children(&self) -> Result<()> {
        Ok(())
    }
    pub fn from_device(device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device.state.native().create_stock_effect(
            device.handle()?,
            StockEffectKind::AlphaTest,
            &mut handle,
        )?;
        Ok(Self {
            effect: Effect::from_handle(device, handle),
            texture: Mutex::new(None),
        })
    }
    pub fn new(cloneSource: &Self) -> Result<Self> {
        Ok(Self {
            effect: cloneSource.effect.Clone()?,
            texture: Mutex::new(
                cloneSource
                    .texture
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
        })
    }
    pub fn Alpha(&self) -> Result<f32> {
        self.effect
            .native()
            .stock_alpha(self.effect.handle()?, StockEffectKind::AlphaTest)
    }
    pub fn SetAlpha(&mut self, value: f32) -> Result<()> {
        self.effect.native().set_stock_alpha(
            self.effect.handle()?,
            StockEffectKind::AlphaTest,
            value,
        )
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .stock_diffuse_color(self.effect.handle()?, StockEffectKind::AlphaTest)
            .map(from_native_vector3)
    }
    pub fn SetDiffuseColor(&mut self, value: Vector3) -> Result<()> {
        self.effect.native().set_stock_diffuse_color(
            self.effect.handle()?,
            StockEffectKind::AlphaTest,
            native_vector3(value),
        )
    }
    pub fn VertexColorEnabled(&self) -> Result<bool> {
        self.effect
            .native()
            .stock_vertex_color(self.effect.handle()?, StockEffectKind::AlphaTest)
    }
    pub fn SetVertexColorEnabled(&mut self, value: bool) -> Result<()> {
        self.effect.native().set_stock_vertex_color(
            self.effect.handle()?,
            StockEffectKind::AlphaTest,
            value,
        )
    }
    pub fn AlphaFunction(&self) -> Result<CompareFunction> {
        match self.effect.native().alpha_function(self.effect.handle()?)? {
            0 => Ok(CompareFunction::Always),
            1 => Ok(CompareFunction::Never),
            2 => Ok(CompareFunction::Less),
            3 => Ok(CompareFunction::LessEqual),
            4 => Ok(CompareFunction::Equal),
            5 => Ok(CompareFunction::GreaterEqual),
            6 => Ok(CompareFunction::Greater),
            7 => Ok(CompareFunction::NotEqual),
            _ => Err(crate::error::CnaError::InvalidInput(
                "native alpha comparison function is unknown",
            )),
        }
    }
    pub fn SetAlphaFunction(&mut self, value: CompareFunction) -> Result<()> {
        self.effect
            .native()
            .set_alpha_function(self.effect.handle()?, value as u32)
    }
    pub fn ReferenceAlpha(&self) -> Result<i32> {
        self.effect.native().reference_alpha(self.effect.handle()?)
    }
    pub fn SetReferenceAlpha(&mut self, value: i32) -> Result<()> {
        self.effect
            .native()
            .set_reference_alpha(self.effect.handle()?, value)
    }
    pub fn Texture(&self) -> Result<Option<Arc<Texture2D>>> {
        self.effect.handle()?;
        Ok(self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTexture(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        let handle = retained_texture_handle(&self.effect, value.as_ref())?;
        self.effect.native().stock_set_texture(
            self.effect.handle()?,
            StockEffectKind::AlphaTest,
            0,
            handle,
        )?;
        *self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn Clone(&self) -> Result<Effect> {
        self.effect.Clone()
    }
    pub fn OnApply(&self) -> Result<()> {
        self.effect.OnApply()
    }
}
impl_fog_matrices!(AlphaTestEffect);
impl_effect_resource!(AlphaTestEffect);
impl Drop for AlphaTestEffect {
    fn drop(&mut self) {}
}

pub struct DualTextureEffect {
    effect: Effect,
    textures: Mutex<[Option<Arc<Texture2D>>; 2]>,
}

impl DualTextureEffect {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn dispose_owned_children(&self) -> Result<()> {
        Ok(())
    }
    pub fn from_device(device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device.state.native().create_stock_effect(
            device.handle()?,
            StockEffectKind::DualTexture,
            &mut handle,
        )?;
        Ok(Self {
            effect: Effect::from_handle(device, handle),
            textures: Mutex::new([None, None]),
        })
    }
    pub fn new(cloneSource: &Self) -> Result<Self> {
        Ok(Self {
            effect: cloneSource.effect.Clone()?,
            textures: Mutex::new(
                cloneSource
                    .textures
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
        })
    }
    pub fn Alpha(&self) -> Result<f32> {
        self.effect
            .native()
            .stock_alpha(self.effect.handle()?, StockEffectKind::DualTexture)
    }
    pub fn SetAlpha(&mut self, value: f32) -> Result<()> {
        self.effect.native().set_stock_alpha(
            self.effect.handle()?,
            StockEffectKind::DualTexture,
            value,
        )
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .stock_diffuse_color(self.effect.handle()?, StockEffectKind::DualTexture)
            .map(from_native_vector3)
    }
    pub fn SetDiffuseColor(&mut self, value: Vector3) -> Result<()> {
        self.effect.native().set_stock_diffuse_color(
            self.effect.handle()?,
            StockEffectKind::DualTexture,
            native_vector3(value),
        )
    }
    pub fn VertexColorEnabled(&self) -> Result<bool> {
        self.effect
            .native()
            .stock_vertex_color(self.effect.handle()?, StockEffectKind::DualTexture)
    }
    pub fn SetVertexColorEnabled(&mut self, value: bool) -> Result<()> {
        self.effect.native().set_stock_vertex_color(
            self.effect.handle()?,
            StockEffectKind::DualTexture,
            value,
        )
    }
    fn texture(&self, index: usize) -> Result<Option<Arc<Texture2D>>> {
        self.effect.handle()?;
        Ok(self
            .textures
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index]
            .clone())
    }
    fn set_texture(&mut self, index: usize, value: Option<Arc<Texture2D>>) -> Result<()> {
        let handle = retained_texture_handle(&self.effect, value.as_ref())?;
        self.effect.native().stock_set_texture(
            self.effect.handle()?,
            StockEffectKind::DualTexture,
            u32::try_from(index)
                .map_err(|_| crate::error::CnaError::InvalidInput("texture index exceeds u32"))?,
            handle,
        )?;
        self.textures
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = value;
        Ok(())
    }
    pub fn Texture(&self) -> Result<Option<Arc<Texture2D>>> {
        self.texture(0)
    }
    pub fn SetTexture(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        self.set_texture(0, value)
    }
    pub fn Texture2(&self) -> Result<Option<Arc<Texture2D>>> {
        self.texture(1)
    }
    pub fn SetTexture2(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        self.set_texture(1, value)
    }
    pub fn Clone(&self) -> Result<Effect> {
        self.effect.Clone()
    }
    pub fn OnApply(&self) -> Result<()> {
        self.effect.OnApply()
    }
}
impl_fog_matrices!(DualTextureEffect);
impl_effect_resource!(DualTextureEffect);
impl Drop for DualTextureEffect {
    fn drop(&mut self) {}
}

macro_rules! impl_lighting {
    ($type:ty) => {
        impl $type {
            pub fn AmbientLightColor(&self) -> Result<Vector3> {
                self.effect
                    .native()
                    .basic_vector3(
                        self.effect.handle()?,
                        BasicVector3Property::AmbientLightColor,
                    )
                    .map(from_native_vector3)
            }
            pub fn SetAmbientLightColor(&mut self, value: Vector3) -> Result<()> {
                self.effect.native().set_basic_vector3(
                    self.effect.handle()?,
                    BasicVector3Property::AmbientLightColor,
                    native_vector3(value),
                )
            }
            pub fn DirectionalLight0(&self) -> Result<&DirectionalLight> {
                self.effect.handle()?;
                Ok(&self.lights[0])
            }
            pub fn DirectionalLight1(&self) -> Result<&DirectionalLight> {
                self.effect.handle()?;
                Ok(&self.lights[1])
            }
            pub fn DirectionalLight2(&self) -> Result<&DirectionalLight> {
                self.effect.handle()?;
                Ok(&self.lights[2])
            }
            pub fn EnableDefaultLighting(&self) -> Result<()> {
                self.effect
                    .native()
                    .enable_default_lighting(self.effect.handle()?)
            }
        }
        impl IEffectLights for $type {
            fn AmbientLightColor(&self) -> Result<Vector3> {
                Self::AmbientLightColor(self)
            }
            fn SetAmbientLightColor(&mut self, value: Vector3) -> Result<()> {
                Self::SetAmbientLightColor(self, value)
            }
            fn DirectionalLight0(&self) -> Result<&DirectionalLight> {
                Self::DirectionalLight0(self)
            }
            fn DirectionalLight1(&self) -> Result<&DirectionalLight> {
                Self::DirectionalLight1(self)
            }
            fn DirectionalLight2(&self) -> Result<&DirectionalLight> {
                Self::DirectionalLight2(self)
            }
            fn LightingEnabled(&self) -> Result<bool> {
                self.effect
                    .native()
                    .basic_bool(self.effect.handle()?, BasicBoolProperty::LightingEnabled)
            }
            fn SetLightingEnabled(&mut self, value: bool) -> Result<()> {
                self.effect.native().set_basic_bool(
                    self.effect.handle()?,
                    BasicBoolProperty::LightingEnabled,
                    value,
                )
            }
            fn EnableDefaultLighting(&self) -> Result<()> {
                Self::EnableDefaultLighting(self)
            }
        }
    };
}

fn retained_cube_handle(
    effect: &Effect,
    value: Option<&Arc<TextureCube>>,
) -> Result<sys::CNA_Handle> {
    value.map_or(Ok(sys::CNA_INVALID_HANDLE), |texture| {
        if !effect.is_same_device(
            texture
                .GraphicsDevice()
                .expect("TextureCube always has a device"),
        ) {
            return Err(crate::error::CnaError::InvalidInput(
                "texture belongs to a different graphics device",
            ));
        }
        texture.handle()
    })
}

pub struct EnvironmentMapEffect {
    lights: [DirectionalLight; 3],
    effect: Effect,
    texture: Mutex<Option<Arc<Texture2D>>>,
    environment_map: Mutex<Option<Arc<TextureCube>>>,
}

impl EnvironmentMapEffect {
    fn dispose_owned_children(&self) -> Result<()> {
        dispose_lights(&self.lights)
    }
    pub fn from_device(device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device.state.native().create_stock_effect(
            device.handle()?,
            StockEffectKind::EnvironmentMap,
            &mut handle,
        )?;
        Self::from_effect(Effect::from_handle(device, handle), None, None)
    }
    pub fn new(cloneSource: &Self) -> Result<Self> {
        Self::from_effect(
            cloneSource.effect.Clone()?,
            cloneSource
                .texture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            cloneSource
                .environment_map
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        )
    }
    fn from_effect(
        effect: Effect,
        texture: Option<Arc<Texture2D>>,
        environment_map: Option<Arc<TextureCube>>,
    ) -> Result<Self> {
        let light0 = DirectionalLight::from_effect(&effect, 0)?;
        let light1 = DirectionalLight::from_effect(&effect, 1)?;
        let light2 = DirectionalLight::from_effect(&effect, 2)?;
        Ok(Self {
            lights: [light0, light1, light2],
            effect,
            texture: Mutex::new(texture),
            environment_map: Mutex::new(environment_map),
        })
    }
    pub fn Alpha(&self) -> Result<f32> {
        self.effect
            .native()
            .stock_alpha(self.effect.handle()?, StockEffectKind::EnvironmentMap)
    }
    pub fn SetAlpha(&mut self, value: f32) -> Result<()> {
        self.effect.native().set_stock_alpha(
            self.effect.handle()?,
            StockEffectKind::EnvironmentMap,
            value,
        )
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .stock_diffuse_color(self.effect.handle()?, StockEffectKind::EnvironmentMap)
            .map(from_native_vector3)
    }
    pub fn SetDiffuseColor(&mut self, value: Vector3) -> Result<()> {
        self.effect.native().set_stock_diffuse_color(
            self.effect.handle()?,
            StockEffectKind::EnvironmentMap,
            native_vector3(value),
        )
    }
    pub fn EmissiveColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .environment_emissive(self.effect.handle()?)
            .map(from_native_vector3)
    }
    pub fn SetEmissiveColor(&mut self, value: Vector3) -> Result<()> {
        self.effect
            .native()
            .set_environment_emissive(self.effect.handle()?, native_vector3(value))
    }
    pub fn EnvironmentMapAmount(&self) -> Result<f32> {
        self.effect
            .native()
            .environment_float(self.effect.handle()?, 0)
    }
    pub fn SetEnvironmentMapAmount(&mut self, value: f32) -> Result<()> {
        self.effect
            .native()
            .set_environment_float(self.effect.handle()?, 0, value)
    }
    pub fn EnvironmentMapSpecular(&self) -> Result<Vector3> {
        self.effect
            .native()
            .environment_specular(self.effect.handle()?)
            .map(from_native_vector3)
    }
    pub fn SetEnvironmentMapSpecular(&mut self, value: Vector3) -> Result<()> {
        self.effect
            .native()
            .set_environment_specular(self.effect.handle()?, native_vector3(value))
    }
    pub fn FresnelFactor(&self) -> Result<f32> {
        self.effect
            .native()
            .environment_float(self.effect.handle()?, 1)
    }
    pub fn SetFresnelFactor(&mut self, value: f32) -> Result<()> {
        self.effect
            .native()
            .set_environment_float(self.effect.handle()?, 1, value)
    }
    pub fn Texture(&self) -> Result<Option<Arc<Texture2D>>> {
        self.effect.handle()?;
        Ok(self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTexture(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        let handle = retained_texture_handle(&self.effect, value.as_ref())?;
        self.effect.native().stock_set_texture(
            self.effect.handle()?,
            StockEffectKind::EnvironmentMap,
            0,
            handle,
        )?;
        *self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn EnvironmentMap(&self) -> Result<Option<Arc<TextureCube>>> {
        self.effect.handle()?;
        Ok(self
            .environment_map
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetEnvironmentMap(&mut self, value: Option<Arc<TextureCube>>) -> Result<()> {
        let handle = retained_cube_handle(&self.effect, value.as_ref())?;
        self.effect
            .native()
            .environment_set_map(self.effect.handle()?, handle)?;
        *self
            .environment_map
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn Clone(&self) -> Result<Effect> {
        self.effect.Clone()
    }
    pub fn OnApply(&self) -> Result<()> {
        self.effect.OnApply()
    }
}
impl_fog_matrices!(EnvironmentMapEffect);
impl_lighting!(EnvironmentMapEffect);
impl_effect_resource!(EnvironmentMapEffect);
impl Drop for EnvironmentMapEffect {
    fn drop(&mut self) {}
}

pub struct SkinnedEffect {
    lights: [DirectionalLight; 3],
    effect: Effect,
    texture: Mutex<Option<Arc<Texture2D>>>,
}

impl SkinnedEffect {
    fn dispose_owned_children(&self) -> Result<()> {
        dispose_lights(&self.lights)
    }
    pub const MaxBones: i32 = 72;
    pub fn from_device(device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device.state.native().create_stock_effect(
            device.handle()?,
            StockEffectKind::Skinned,
            &mut handle,
        )?;
        Self::from_effect(Effect::from_handle(device, handle), None)
    }
    pub fn new(cloneSource: &Self) -> Result<Self> {
        Self::from_effect(
            cloneSource.effect.Clone()?,
            cloneSource
                .texture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        )
    }
    fn from_effect(effect: Effect, texture: Option<Arc<Texture2D>>) -> Result<Self> {
        let light0 = DirectionalLight::from_effect(&effect, 0)?;
        let light1 = DirectionalLight::from_effect(&effect, 1)?;
        let light2 = DirectionalLight::from_effect(&effect, 2)?;
        Ok(Self {
            lights: [light0, light1, light2],
            effect,
            texture: Mutex::new(texture),
        })
    }
    pub fn Alpha(&self) -> Result<f32> {
        self.effect
            .native()
            .stock_alpha(self.effect.handle()?, StockEffectKind::Skinned)
    }
    pub fn SetAlpha(&mut self, value: f32) -> Result<()> {
        self.effect
            .native()
            .set_stock_alpha(self.effect.handle()?, StockEffectKind::Skinned, value)
    }
    pub fn DiffuseColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .stock_diffuse_color(self.effect.handle()?, StockEffectKind::Skinned)
            .map(from_native_vector3)
    }
    pub fn SetDiffuseColor(&mut self, value: Vector3) -> Result<()> {
        self.effect.native().set_stock_diffuse_color(
            self.effect.handle()?,
            StockEffectKind::Skinned,
            native_vector3(value),
        )
    }
    pub fn EmissiveColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .skinned_vector3(self.effect.handle()?, 0)
            .map(from_native_vector3)
    }
    pub fn SetEmissiveColor(&mut self, value: Vector3) -> Result<()> {
        self.effect
            .native()
            .set_skinned_vector3(self.effect.handle()?, 0, native_vector3(value))
    }
    pub fn SpecularColor(&self) -> Result<Vector3> {
        self.effect
            .native()
            .skinned_vector3(self.effect.handle()?, 1)
            .map(from_native_vector3)
    }
    pub fn SetSpecularColor(&mut self, value: Vector3) -> Result<()> {
        self.effect
            .native()
            .set_skinned_vector3(self.effect.handle()?, 1, native_vector3(value))
    }
    pub fn SpecularPower(&self) -> Result<f32> {
        self.effect
            .native()
            .skinned_specular_power(self.effect.handle()?)
    }
    pub fn SetSpecularPower(&mut self, value: f32) -> Result<()> {
        self.effect
            .native()
            .set_skinned_specular_power(self.effect.handle()?, value)
    }
    pub fn PreferPerPixelLighting(&self) -> Result<bool> {
        self.effect
            .native()
            .skinned_prefer_pixel(self.effect.handle()?)
    }
    pub fn SetPreferPerPixelLighting(&mut self, value: bool) -> Result<()> {
        self.effect
            .native()
            .set_skinned_prefer_pixel(self.effect.handle()?, value)
    }
    pub fn WeightsPerVertex(&self) -> Result<i32> {
        self.effect.native().skinned_weights(self.effect.handle()?)
    }
    pub fn SetWeightsPerVertex(&mut self, value: i32) -> Result<()> {
        self.effect
            .native()
            .set_skinned_weights(self.effect.handle()?, value)
    }
    pub fn SetBoneTransforms(&self, boneTransforms: &[Matrix]) -> Result<()> {
        let values = boneTransforms
            .iter()
            .copied()
            .map(native_matrix)
            .collect::<Vec<_>>();
        self.effect
            .native()
            .set_skinned_bones(self.effect.handle()?, &values)
    }
    pub fn GetBoneTransforms(&self, count: i32) -> Result<Vec<Matrix>> {
        if !(1..=Self::MaxBones).contains(&count) {
            return Err(crate::error::CnaError::InvalidInput(
                "bone transform count must be between one and MaxBones",
            ));
        }
        let mut values = vec![
            sys::CNA_Matrix::default();
            usize::try_from(count).map_err(|_| {
                crate::error::CnaError::InvalidInput("bone transform count is negative")
            })?
        ];
        let written = self
            .effect
            .native()
            .copy_skinned_bones(self.effect.handle()?, &mut values)?;
        values.truncate(written);
        Ok(values.into_iter().map(from_native_matrix).collect())
    }
    pub fn Texture(&self) -> Result<Option<Arc<Texture2D>>> {
        self.effect.handle()?;
        Ok(self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTexture(&mut self, value: Option<Arc<Texture2D>>) -> Result<()> {
        let handle = retained_texture_handle(&self.effect, value.as_ref())?;
        self.effect.native().stock_set_texture(
            self.effect.handle()?,
            StockEffectKind::Skinned,
            0,
            handle,
        )?;
        *self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn Clone(&self) -> Result<Effect> {
        self.effect.Clone()
    }
    pub fn OnApply(&self) -> Result<()> {
        self.effect.OnApply()
    }
}
impl_fog_matrices!(SkinnedEffect);
impl_lighting!(SkinnedEffect);
impl_effect_resource!(SkinnedEffect);
impl Drop for SkinnedEffect {
    fn drop(&mut self) {}
}
