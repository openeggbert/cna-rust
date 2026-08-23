#![allow(
    non_snake_case,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

use core::mem::size_of;
use core::ops::Deref;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::vec::IntoIter;

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::value::{Matrix, Quaternion, Vector2, Vector3, Vector4};

use super::resource::{ResourceKind, ResourceState};
use super::{
    GraphicsDevice, GraphicsResource, RenderTarget2D, RenderTargetCube, Texture, Texture2D,
    TextureCube,
};

const ANNOTATION: u8 = 0;
const PARAMETER: u8 = 1;
const PASS: u8 = 2;
const TECHNIQUE: u8 = 3;
const ANNOTATION_COLLECTION: u8 = 4;
const PARAMETER_COLLECTION: u8 = 5;
const PASS_COLLECTION: u8 = 6;
const TECHNIQUE_COLLECTION: u8 = 7;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum EffectParameterClass {
    Scalar = 0,
    Vector = 1,
    Matrix = 2,
    Object = 3,
    Struct = 4,
}

impl EffectParameterClass {
    fn from_native(value: sys::CNA_EffectParameterClass) -> Result<Self> {
        match value {
            sys::CNA_EFFECT_PARAMETER_CLASS_SCALAR => Ok(Self::Scalar),
            sys::CNA_EFFECT_PARAMETER_CLASS_VECTOR => Ok(Self::Vector),
            sys::CNA_EFFECT_PARAMETER_CLASS_MATRIX => Ok(Self::Matrix),
            sys::CNA_EFFECT_PARAMETER_CLASS_OBJECT => Ok(Self::Object),
            sys::CNA_EFFECT_PARAMETER_CLASS_STRUCT => Ok(Self::Struct),
            _ => Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an unknown effect parameter class".to_owned(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum EffectParameterType {
    Void = 0,
    Bool = 1,
    Int32 = 2,
    Single = 3,
    String = 4,
    Texture = 5,
    Texture1D = 6,
    Texture2D = 7,
    Texture3D = 8,
    TextureCube = 9,
}

impl EffectParameterType {
    fn from_native(value: sys::CNA_EffectParameterType) -> Result<Self> {
        match value {
            sys::CNA_EFFECT_PARAMETER_TYPE_VOID => Ok(Self::Void),
            sys::CNA_EFFECT_PARAMETER_TYPE_BOOL => Ok(Self::Bool),
            sys::CNA_EFFECT_PARAMETER_TYPE_INT32 => Ok(Self::Int32),
            sys::CNA_EFFECT_PARAMETER_TYPE_SINGLE => Ok(Self::Single),
            sys::CNA_EFFECT_PARAMETER_TYPE_STRING => Ok(Self::String),
            sys::CNA_EFFECT_PARAMETER_TYPE_TEXTURE => Ok(Self::Texture),
            sys::CNA_EFFECT_PARAMETER_TYPE_TEXTURE1D => Ok(Self::Texture1D),
            sys::CNA_EFFECT_PARAMETER_TYPE_TEXTURE2D => Ok(Self::Texture2D),
            sys::CNA_EFFECT_PARAMETER_TYPE_TEXTURE3D => Ok(Self::Texture3D),
            sys::CNA_EFFECT_PARAMETER_TYPE_TEXTURE_CUBE => Ok(Self::TextureCube),
            _ => Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an unknown effect parameter type".to_owned(),
            }),
        }
    }
}

/// CNA extension descriptor for constructing a reflected annotation graph.
#[derive(Clone, Debug)]
pub struct EffectAnnotationDescriptor {
    pub name: String,
    pub semantic: String,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: EffectParameterClass,
    pub parameter_type: EffectParameterType,
    pub data: Vec<f32>,
    pub cached_string: String,
}

/// CNA extension descriptor for constructing one reflected parameter.
#[derive(Clone, Debug)]
pub struct EffectParameterDescriptor {
    pub name: String,
    pub semantic: String,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: EffectParameterClass,
    pub parameter_type: EffectParameterType,
    pub annotations: Vec<EffectAnnotationDescriptor>,
}

/// CNA extension descriptor for a reflected technique and its pass names.
#[derive(Clone, Debug)]
pub struct EffectTechniqueDescriptor {
    pub name: String,
    pub passes: Vec<String>,
}

struct EffectViewState {
    owner: Arc<ResourceState>,
    handle: Mutex<sys::CNA_Handle>,
    kind: u8,
}

impl EffectViewState {
    fn new(owner: Arc<ResourceState>, handle: sys::CNA_Handle, kind: u8) -> Arc<Self> {
        Arc::new(Self {
            owner,
            handle: Mutex::new(handle),
            kind,
        })
    }

    fn require_handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.require_handle()?;
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            Err(CnaError::InvalidInput("effect child view is disposed"))
        } else {
            Ok(handle)
        }
    }

    fn owner(&self) -> &Arc<ResourceState> {
        &self.owner
    }
}

impl Drop for EffectViewState {
    fn drop(&mut self) {
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle != sys::CNA_INVALID_HANDLE {
            let _ = self
                .owner
                .device()
                .state
                .native()
                .destroy_effect_view(handle, self.kind);
        }
    }
}

/// Native XNA Effect with independently owned compiled state.
pub struct Effect {
    state: Arc<ResourceState>,
    parameters: Mutex<Option<Arc<EffectParameterCollection>>>,
    techniques: Mutex<Option<Arc<EffectTechniqueCollection>>>,
    reflection_blueprint: Mutex<Option<Arc<EffectReflectionBlueprint>>>,
}

struct EffectReflectionBlueprint {
    parameters: Vec<EffectParameterDescriptor>,
    techniques: Vec<EffectTechniqueDescriptor>,
}

/// XNA base relationship used by `EffectMaterial`.
pub trait EffectBase: GraphicsResource {
    fn AsEffect(&self) -> &Effect;
}

#[allow(non_snake_case)]
impl Effect {
    pub fn new(cloneSource: &Self) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        let device = cloneSource.state.device();
        device
            .state
            .native()
            .clone_effect(cloneSource.handle()?, &mut handle)?;
        let clone = Self::from_handle(device, handle);
        if let Some(blueprint) = cloneSource.reflection_blueprint() {
            clone.adopt_reflection_blueprint(blueprint)?;
        }
        Ok(clone)
    }

    pub fn from_graphics_device_and_effect_code(
        graphicsDevice: &GraphicsDevice,
        effectCode: &[u8],
    ) -> Result<Self> {
        if effectCode.is_empty() {
            return Err(CnaError::InvalidInput("Effect bytecode cannot be empty"));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_compiled_effect(
            graphicsDevice.handle()?,
            effectCode,
            &mut handle,
        )?;
        Ok(Self::from_handle(graphicsDevice, handle))
    }

    pub(crate) fn create_empty(graphics_device: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphics_device
            .state
            .native()
            .create_empty_effect(graphics_device.handle()?, &mut handle)?;
        Ok(Self::from_handle(graphics_device, handle))
    }

    pub(crate) fn create_reflection(
        graphics_device: &GraphicsDevice,
        parameters: &[EffectParameterDescriptor],
        techniques: &[EffectTechniqueDescriptor],
    ) -> Result<Self> {
        let effect = Self::create_empty(graphics_device)?;
        effect.populate_reflection(parameters, techniques)?;
        *effect
            .reflection_blueprint
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(Arc::new(EffectReflectionBlueprint {
                parameters: parameters.to_vec(),
                techniques: techniques.to_vec(),
            }));
        Ok(effect)
    }

    fn populate_reflection(
        &self,
        parameters: &[EffectParameterDescriptor],
        techniques: &[EffectTechniqueDescriptor],
    ) -> Result<()> {
        let native = self.state.device().state.native();
        let mut parameter_collection_handle = sys::CNA_INVALID_HANDLE;
        native.effect_parameters(self.handle()?, &mut parameter_collection_handle)?;
        let parameter_collection = EffectViewState::new(
            Arc::clone(&self.state),
            parameter_collection_handle,
            PARAMETER_COLLECTION,
        );
        for descriptor in parameters {
            let info = sys::CNA_EffectParameterCreateInfo {
                struct_size: size_of::<sys::CNA_EffectParameterCreateInfo>() as u32,
                struct_version: 1,
                name: string_view(&descriptor.name)?,
                semantic: string_view(&descriptor.semantic)?,
                row_count: descriptor.row_count,
                column_count: descriptor.column_count,
                parameter_class: descriptor.parameter_class as u32,
                parameter_type: descriptor.parameter_type as u32,
            };
            let mut parameter_handle = sys::CNA_INVALID_HANDLE;
            native.add_effect_parameter(
                parameter_collection.require_handle()?,
                &info,
                &mut parameter_handle,
            )?;
            let parameter =
                EffectViewState::new(Arc::clone(&self.state), parameter_handle, PARAMETER);
            let mut annotations_handle = sys::CNA_INVALID_HANDLE;
            native.effect_parameter_annotations(
                parameter.require_handle()?,
                &mut annotations_handle,
            )?;
            let annotations = EffectViewState::new(
                Arc::clone(&self.state),
                annotations_handle,
                ANNOTATION_COLLECTION,
            );
            for annotation in &descriptor.annotations {
                let data_count = u64::try_from(annotation.data.len())
                    .map_err(|_| CnaError::InvalidInput("effect annotation data is too large"))?;
                let create = sys::CNA_EffectAnnotationCreateInfo {
                    struct_size: size_of::<sys::CNA_EffectAnnotationCreateInfo>() as u32,
                    struct_version: 1,
                    name: string_view(&annotation.name)?,
                    semantic: string_view(&annotation.semantic)?,
                    row_count: annotation.row_count,
                    column_count: annotation.column_count,
                    parameter_class: annotation.parameter_class as u32,
                    parameter_type: annotation.parameter_type as u32,
                    data: if annotation.data.is_empty() {
                        core::ptr::null()
                    } else {
                        annotation.data.as_ptr()
                    },
                    data_count,
                    cached_string: string_view(&annotation.cached_string)?,
                };
                let mut annotation_handle = sys::CNA_INVALID_HANDLE;
                native.create_effect_annotation(&create, &mut annotation_handle)?;
                let annotation_view =
                    EffectViewState::new(Arc::clone(&self.state), annotation_handle, ANNOTATION);
                native.add_effect_annotation(
                    annotations.require_handle()?,
                    annotation_view.require_handle()?,
                )?;
            }
        }

        let mut technique_collection_handle = sys::CNA_INVALID_HANDLE;
        native.effect_techniques(self.handle()?, &mut technique_collection_handle)?;
        let technique_collection = EffectViewState::new(
            Arc::clone(&self.state),
            technique_collection_handle,
            TECHNIQUE_COLLECTION,
        );
        for (index, descriptor) in techniques.iter().enumerate() {
            let mut technique_handle = sys::CNA_INVALID_HANDLE;
            native.add_effect_technique(
                technique_collection.require_handle()?,
                string_view(&descriptor.name)?,
                &mut technique_handle,
            )?;
            let technique =
                EffectViewState::new(Arc::clone(&self.state), technique_handle, TECHNIQUE);
            let mut pass_collection_handle = sys::CNA_INVALID_HANDLE;
            native.effect_technique_passes(
                technique.require_handle()?,
                &mut pass_collection_handle,
            )?;
            let pass_collection = EffectViewState::new(
                Arc::clone(&self.state),
                pass_collection_handle,
                PASS_COLLECTION,
            );
            for pass_name in &descriptor.passes {
                let mut pass_handle = sys::CNA_INVALID_HANDLE;
                native.add_effect_pass(
                    pass_collection.require_handle()?,
                    string_view(pass_name)?,
                    0,
                    &mut pass_handle,
                )?;
                let _pass = EffectViewState::new(Arc::clone(&self.state), pass_handle, PASS);
            }
            if index == 0 {
                native.set_current_effect_technique(self.handle()?, technique.require_handle()?)?;
            }
        }
        Ok(())
    }

    fn from_handle(device: &GraphicsDevice, handle: sys::CNA_Handle) -> Self {
        Self {
            state: ResourceState::new(device, handle, ResourceKind::Effect),
            parameters: Mutex::new(None),
            techniques: Mutex::new(None),
            reflection_blueprint: Mutex::new(None),
        }
    }

    fn reflection_blueprint(&self) -> Option<Arc<EffectReflectionBlueprint>> {
        self.reflection_blueprint
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn adopt_reflection_blueprint(&self, blueprint: Arc<EffectReflectionBlueprint>) -> Result<()> {
        let native = self.state.device().state.native();
        let mut parameters_handle = sys::CNA_INVALID_HANDLE;
        native.effect_parameters(self.handle()?, &mut parameters_handle)?;
        let parameters = EffectViewState::new(
            Arc::clone(&self.state),
            parameters_handle,
            PARAMETER_COLLECTION,
        );
        let parameter_count = collection_count(&parameters, PARAMETER)?;
        let mut techniques_handle = sys::CNA_INVALID_HANDLE;
        native.effect_techniques(self.handle()?, &mut techniques_handle)?;
        let techniques = EffectViewState::new(
            Arc::clone(&self.state),
            techniques_handle,
            TECHNIQUE_COLLECTION,
        );
        let technique_count = collection_count(&techniques, TECHNIQUE)?;
        let expected_techniques = blueprint.techniques.len().saturating_add(1);
        if parameter_count == 0 && technique_count == 1 {
            self.populate_reflection(&blueprint.parameters, &blueprint.techniques)?;
        } else if parameter_count != blueprint.parameters.len()
            || technique_count != expected_techniques
        {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA cloned only part of a reflected Effect graph".to_owned(),
            });
        }
        *self
            .reflection_blueprint
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(blueprint);
        Ok(())
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }

    pub(crate) fn is_same_device(&self, device: &GraphicsDevice) -> bool {
        self.state.device().is_same_device(device)
    }

    pub fn Parameters(&self) -> Result<Arc<EffectParameterCollection>> {
        let mut cached = self
            .parameters
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cached.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .device()
            .state
            .native()
            .effect_parameters(self.handle()?, &mut handle)?;
        let value = Arc::new(EffectParameterCollection::from_handle(
            Arc::clone(&self.state),
            handle,
        )?);
        *cached = Some(Arc::clone(&value));
        Ok(value)
    }

    pub fn Techniques(&self) -> Result<Arc<EffectTechniqueCollection>> {
        let mut cached = self
            .techniques
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cached.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .device()
            .state
            .native()
            .effect_techniques(self.handle()?, &mut handle)?;
        let value = Arc::new(EffectTechniqueCollection::from_handle(
            Arc::clone(&self.state),
            handle,
        )?);
        *cached = Some(Arc::clone(&value));
        Ok(value)
    }

    pub fn CurrentTechnique(&self) -> Result<Arc<EffectTechnique>> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .device()
            .state
            .native()
            .current_effect_technique(self.handle()?, &mut handle)?;
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("Effect has no current technique"));
        }
        let temporary = EffectTechnique::from_handle(Arc::clone(&self.state), handle)?;
        let name = temporary.Name()?;
        self.Techniques()?
            .Item(&name)?
            .ok_or_else(|| CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "current effect technique is absent from Techniques".to_owned(),
            })
    }

    pub fn SetCurrentTechnique(&mut self, value: &EffectTechnique) -> Result<()> {
        if !Arc::ptr_eq(&self.state, value.state.owner()) {
            return Err(CnaError::InvalidInput(
                "effect technique belongs to a different Effect",
            ));
        }
        self.state
            .device()
            .state
            .native()
            .set_current_effect_technique(self.handle()?, value.state.require_handle()?)
    }

    pub fn Clone(&self) -> Result<Self> {
        Self::new(self)
    }

    pub fn OnApply(&self) -> Result<()> {
        self.state
            .device()
            .state
            .native()
            .apply_effect(self.handle()?)
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
}

impl GraphicsResource for Effect {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        Some(self.state.device())
    }
    fn IsDisposed(&self) -> bool {
        self.state.handle().is_none()
    }
    fn Name(&self) -> String {
        self.state.name()
    }
    fn SetName(&mut self, value: &str) {
        self.state.set_name(value);
    }
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.state.tag()
    }
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.state.set_tag(value);
    }
    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.add_disposing_handler(handler)
    }
    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.remove_disposing_handler(registration)
    }
    fn Dispose(&mut self, value: bool) -> Result<()> {
        Effect::Dispose(self, value)
    }
}

impl ContentDisposable for Effect {
    fn DisposeContent(&self) -> Result<()> {
        self.state.dispose_native()
    }
}

impl ContentLoadable for Effect {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        let _ = self.state.dispose_native();
    }
}

/// XNA material Effect subtype cloned through CNA's material constructor.
pub struct EffectMaterial {
    effect: Effect,
}

#[allow(non_snake_case)]
impl EffectMaterial {
    pub fn new(cloneSource: &Effect) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        let device = cloneSource.state.device();
        device
            .state
            .native()
            .create_effect_material(cloneSource.handle()?, &mut handle)?;
        let effect = Effect::from_handle(device, handle);
        if let Some(blueprint) = cloneSource.reflection_blueprint() {
            effect.adopt_reflection_blueprint(blueprint)?;
        }
        Ok(Self { effect })
    }
}

impl EffectBase for EffectMaterial {
    fn AsEffect(&self) -> &Effect {
        &self.effect
    }
}

impl Deref for EffectMaterial {
    type Target = Effect;
    fn deref(&self) -> &Self::Target {
        &self.effect
    }
}

impl GraphicsResource for EffectMaterial {
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
        self.effect.Dispose(value)
    }
}

impl Drop for EffectMaterial {
    fn drop(&mut self) {
        let _ = self.effect.state.dispose_native();
    }
}

pub struct EffectAnnotation {
    state: Arc<EffectViewState>,
    info: sys::CNA_EffectAnnotationInfo,
    name: String,
    semantic: String,
}

#[allow(non_snake_case)]
impl EffectAnnotation {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, ANNOTATION);
        let native = state.owner.device().state.native();
        let mut info = sys::CNA_EffectAnnotationInfo {
            struct_size: size_of::<sys::CNA_EffectAnnotationInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_EffectAnnotationInfo::default()
        };
        native.effect_annotation_info(state.require_handle()?, &mut info)?;
        let name = native.effect_annotation_name(state.require_handle()?)?;
        let semantic = native.effect_annotation_semantic(state.require_handle()?)?;
        Ok(Self {
            state,
            info,
            name,
            semantic,
        })
    }

    pub fn ParameterType(&self) -> Result<EffectParameterType> {
        EffectParameterType::from_native(self.info.parameter_type)
    }
    pub fn ParameterClass(&self) -> Result<EffectParameterClass> {
        EffectParameterClass::from_native(self.info.parameter_class)
    }
    pub fn ColumnCount(&self) -> Result<i32> {
        self.state.require_handle()?;
        Ok(self.info.column_count)
    }
    pub fn RowCount(&self) -> Result<i32> {
        self.state.require_handle()?;
        Ok(self.info.row_count)
    }
    pub fn Semantic(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.semantic.clone())
    }
    pub fn Name(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.name.clone())
    }
    pub fn GetValueBoolean(&self) -> Result<bool> {
        Ok(self
            .state
            .owner
            .device()
            .state
            .native()
            .annotation_boolean(self.state.require_handle()?)?
            != sys::CNA_FALSE)
    }
    pub fn GetValueInt32(&self) -> Result<i32> {
        self.state
            .owner
            .device()
            .state
            .native()
            .annotation_int32(self.state.require_handle()?)
    }
    pub fn GetValueSingle(&self) -> Result<f32> {
        self.state
            .owner
            .device()
            .state
            .native()
            .annotation_single(self.state.require_handle()?)
    }
    pub fn GetValueVector2(&self) -> Result<Vector2> {
        let value = self
            .state
            .owner
            .device()
            .state
            .native()
            .annotation_vector2(self.state.require_handle()?)?;
        Ok(Vector2::from_x_and_y(value.x, value.y))
    }
    pub fn GetValueVector3(&self) -> Result<Vector3> {
        Ok(from_native_vector3(
            self.state
                .owner
                .device()
                .state
                .native()
                .annotation_vector3(self.state.require_handle()?)?,
        ))
    }
    pub fn GetValueVector4(&self) -> Result<Vector4> {
        Ok(from_native_vector4(
            self.state
                .owner
                .device()
                .state
                .native()
                .annotation_vector4(self.state.require_handle()?)?,
        ))
    }
    pub fn GetValueMatrix(&self) -> Result<Matrix> {
        Ok(from_native_matrix(
            self.state
                .owner
                .device()
                .state
                .native()
                .annotation_matrix(self.state.require_handle()?)?,
        ))
    }
    pub fn GetValueString(&self) -> Result<String> {
        self.state
            .owner
            .device()
            .state
            .native()
            .effect_annotation_string(self.state.require_handle()?)
    }
}

#[derive(Clone)]
pub struct EffectAnnotationCollection {
    state: Arc<EffectViewState>,
    items: Arc<Vec<Arc<EffectAnnotation>>>,
}

#[allow(non_snake_case)]
impl EffectAnnotationCollection {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, ANNOTATION_COLLECTION);
        let count = collection_count(&state, ANNOTATION)?;
        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let child = state
                .owner
                .device()
                .state
                .native()
                .effect_collection_get_at(state.require_handle()?, ANNOTATION, index as u64)?;
            items.push(Arc::new(EffectAnnotation::from_handle(
                Arc::clone(state.owner()),
                child,
            )?));
        }
        Ok(Self {
            state,
            items: Arc::new(items),
        })
    }

    pub fn Item(&self, name: &str) -> Result<Option<Arc<EffectAnnotation>>> {
        let found = verified_find(&self.state, ANNOTATION, name, false)?;
        let cached = self
            .items
            .iter()
            .find(|value| value.name == name)
            .map(Arc::clone);
        verified_cached_lookup(found, cached)
    }
    pub fn Count(&self) -> Result<i32> {
        self.state.require_handle()?;
        checked_count(self.items.len())
    }
    pub fn GetEnumerator(&self) -> Result<IntoIter<Arc<EffectAnnotation>>> {
        self.state.require_handle()?;
        Ok(self.items.as_ref().clone().into_iter())
    }
    pub(crate) fn item_at(&self, index: i32) -> Result<Arc<EffectAnnotation>> {
        collection_item_at(&self.state, &self.items, index)
    }
}

impl IntoIterator for EffectAnnotationCollection {
    type Item = Arc<EffectAnnotation>;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.as_ref().clone().into_iter()
    }
}

pub struct EffectParameter {
    state: Arc<EffectViewState>,
    info: sys::CNA_EffectParameterInfo,
    name: String,
    semantic: String,
    elements: Mutex<Option<Arc<EffectParameterCollection>>>,
    structure_members: Mutex<Option<Arc<EffectParameterCollection>>>,
    annotations: Mutex<Option<Arc<EffectAnnotationCollection>>>,
    retained_texture: Mutex<Option<Arc<dyn Texture>>>,
}

#[allow(non_snake_case)]
impl EffectParameter {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, PARAMETER);
        let native = state.owner.device().state.native();
        let mut info = sys::CNA_EffectParameterInfo {
            struct_size: size_of::<sys::CNA_EffectParameterInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_EffectParameterInfo::default()
        };
        native.effect_parameter_info(state.require_handle()?, &mut info)?;
        let name = native.effect_parameter_name(state.require_handle()?)?;
        let semantic = native.effect_parameter_semantic(state.require_handle()?)?;
        Ok(Self {
            state,
            info,
            name,
            semantic,
            elements: Mutex::new(None),
            structure_members: Mutex::new(None),
            annotations: Mutex::new(None),
            retained_texture: Mutex::new(None),
        })
    }

    fn native(&self) -> &crate::native::Native {
        self.state.owner.device().state.native()
    }

    pub fn ParameterType(&self) -> Result<EffectParameterType> {
        EffectParameterType::from_native(self.info.parameter_type)
    }
    pub fn ParameterClass(&self) -> Result<EffectParameterClass> {
        EffectParameterClass::from_native(self.info.parameter_class)
    }
    pub fn Elements(&self) -> Result<Arc<EffectParameterCollection>> {
        self.child_parameters(false)
    }
    pub fn StructureMembers(&self) -> Result<Arc<EffectParameterCollection>> {
        self.child_parameters(true)
    }
    fn child_parameters(&self, structure: bool) -> Result<Arc<EffectParameterCollection>> {
        let cache = if structure {
            &self.structure_members
        } else {
            &self.elements
        };
        let mut cache = cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cache.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.native().effect_parameter_child_collection(
            self.state.require_handle()?,
            structure,
            &mut handle,
        )?;
        let value = Arc::new(EffectParameterCollection::from_handle(
            Arc::clone(self.state.owner()),
            handle,
        )?);
        *cache = Some(Arc::clone(&value));
        Ok(value)
    }
    pub fn ColumnCount(&self) -> Result<i32> {
        self.state.require_handle()?;
        Ok(self.info.column_count)
    }
    pub fn RowCount(&self) -> Result<i32> {
        self.state.require_handle()?;
        Ok(self.info.row_count)
    }
    pub fn Annotations(&self) -> Result<Arc<EffectAnnotationCollection>> {
        let mut cache = self
            .annotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cache.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.native()
            .effect_parameter_annotations(self.state.require_handle()?, &mut handle)?;
        let value = Arc::new(EffectAnnotationCollection::from_handle(
            Arc::clone(self.state.owner()),
            handle,
        )?);
        *cache = Some(Arc::clone(&value));
        Ok(value)
    }
    pub fn Semantic(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.semantic.clone())
    }
    pub fn Name(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.name.clone())
    }

    pub fn SetValue(&self, value: Option<Arc<dyn Texture>>) -> Result<()> {
        let handle = value
            .as_ref()
            .map(|texture| texture_handle(texture.as_ref(), self.state.owner.device()))
            .transpose()?
            .unwrap_or(sys::CNA_INVALID_HANDLE);
        self.native().set_effect_parameter_texture(
            self.state.require_handle()?,
            sys::CNA_EFFECT_TEXTURE_BASE,
            handle,
        )?;
        *self
            .retained_texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn SetValueWithValueAsString(&self, value: &str) -> Result<()> {
        self.native()
            .set_effect_parameter_string(self.state.require_handle()?, string_view(value)?)
    }
    pub fn SetValueWithValueAsMatrixArray(&self, value: &[Matrix]) -> Result<()> {
        self.set_matrices(value, sys::CNA_EFFECT_VALUE_MATRIX)
    }
    pub fn SetValueWithValueAsMatrix(&self, value: Matrix) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_MATRIX, native_matrix(value))
    }
    pub fn SetValueWithValueAsQuaternionArray(&self, value: &[Quaternion]) -> Result<()> {
        self.set_many(
            sys::CNA_EFFECT_VALUE_QUATERNION,
            value
                .iter()
                .copied()
                .map(native_quaternion)
                .collect::<Vec<_>>(),
        )
    }
    pub fn SetValueWithValueAsQuaternion(&self, value: Quaternion) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_QUATERNION, native_quaternion(value))
    }
    pub fn SetValueWithValueAsVector4Array(&self, value: &[Vector4]) -> Result<()> {
        self.set_many(
            sys::CNA_EFFECT_VALUE_VECTOR4,
            value
                .iter()
                .copied()
                .map(native_vector4)
                .collect::<Vec<_>>(),
        )
    }
    pub fn SetValueWithValueAsVector4(&self, value: Vector4) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_VECTOR4, native_vector4(value))
    }
    pub fn SetValueWithValueAsVector3Array(&self, value: &[Vector3]) -> Result<()> {
        self.set_many(
            sys::CNA_EFFECT_VALUE_VECTOR3,
            value
                .iter()
                .copied()
                .map(native_vector3)
                .collect::<Vec<_>>(),
        )
    }
    pub fn SetValueWithValueAsVector3(&self, value: Vector3) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_VECTOR3, native_vector3(value))
    }
    pub fn SetValueWithValueAsVector2Array(&self, value: &[Vector2]) -> Result<()> {
        self.set_many(
            sys::CNA_EFFECT_VALUE_VECTOR2,
            value
                .iter()
                .map(|v| sys::CNA_Vector2 { x: v.X, y: v.Y })
                .collect::<Vec<_>>(),
        )
    }
    pub fn SetValueWithValueAsVector2(&self, value: Vector2) -> Result<()> {
        self.set_one(
            sys::CNA_EFFECT_VALUE_VECTOR2,
            sys::CNA_Vector2 {
                x: value.X,
                y: value.Y,
            },
        )
    }
    pub fn SetValueWithValueAsSingleArray(&self, value: &[f32]) -> Result<()> {
        self.set_slice(sys::CNA_EFFECT_VALUE_SINGLE, value)
    }
    pub fn SetValueWithValueAsSingle(&self, value: f32) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_SINGLE, value)
    }
    pub fn SetValueWithValueAsInt32Array(&self, value: &[i32]) -> Result<()> {
        self.set_slice(sys::CNA_EFFECT_VALUE_INT32, value)
    }
    pub fn SetValueWithValueAsInt32(&self, value: i32) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_INT32, value)
    }
    pub fn SetValueWithValueAsBooleanArray(&self, value: &[bool]) -> Result<()> {
        self.set_many(
            sys::CNA_EFFECT_VALUE_BOOLEAN,
            value
                .iter()
                .map(|v| if *v { sys::CNA_TRUE } else { sys::CNA_FALSE })
                .collect::<Vec<_>>(),
        )
    }
    pub fn SetValueWithValueAsBoolean(&self, value: bool) -> Result<()> {
        self.set_one(
            sys::CNA_EFFECT_VALUE_BOOLEAN,
            if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
        )
    }
    pub fn SetValueTranspose(&self, value: &[Matrix]) -> Result<()> {
        self.set_matrices(value, sys::CNA_EFFECT_VALUE_MATRIX_TRANSPOSE)
    }
    pub fn SetValueTransposeWithValue(&self, value: Matrix) -> Result<()> {
        self.set_one(sys::CNA_EFFECT_VALUE_MATRIX_TRANSPOSE, native_matrix(value))
    }

    pub fn GetValueBoolean(&self) -> Result<bool> {
        Ok(self.get_one::<sys::CNA_Bool>(sys::CNA_EFFECT_VALUE_BOOLEAN)? != sys::CNA_FALSE)
    }
    pub fn GetValueBooleanArray(&self, count: i32) -> Result<Vec<bool>> {
        Ok(self
            .get_many::<sys::CNA_Bool>(sys::CNA_EFFECT_VALUE_BOOLEAN, count)?
            .into_iter()
            .map(|value| value != sys::CNA_FALSE)
            .collect())
    }
    pub fn GetValueInt32(&self) -> Result<i32> {
        self.get_one(sys::CNA_EFFECT_VALUE_INT32)
    }
    pub fn GetValueInt32Array(&self, count: i32) -> Result<Vec<i32>> {
        self.get_many(sys::CNA_EFFECT_VALUE_INT32, count)
    }
    pub fn GetValueSingle(&self) -> Result<f32> {
        self.get_one(sys::CNA_EFFECT_VALUE_SINGLE)
    }
    pub fn GetValueSingleArray(&self, count: i32) -> Result<Vec<f32>> {
        self.get_many(sys::CNA_EFFECT_VALUE_SINGLE, count)
    }
    pub fn GetValueVector2(&self) -> Result<Vector2> {
        let value: sys::CNA_Vector2 = self.get_one(sys::CNA_EFFECT_VALUE_VECTOR2)?;
        Ok(Vector2::from_x_and_y(value.x, value.y))
    }
    pub fn GetValueVector2Array(&self, count: i32) -> Result<Vec<Vector2>> {
        Ok(self
            .get_many::<sys::CNA_Vector2>(sys::CNA_EFFECT_VALUE_VECTOR2, count)?
            .into_iter()
            .map(|v| Vector2::from_x_and_y(v.x, v.y))
            .collect())
    }
    pub fn GetValueVector3(&self) -> Result<Vector3> {
        Ok(from_native_vector3(
            self.get_one(sys::CNA_EFFECT_VALUE_VECTOR3)?,
        ))
    }
    pub fn GetValueVector3Array(&self, count: i32) -> Result<Vec<Vector3>> {
        Ok(self
            .get_many::<sys::CNA_Vector3>(sys::CNA_EFFECT_VALUE_VECTOR3, count)?
            .into_iter()
            .map(from_native_vector3)
            .collect())
    }
    pub fn GetValueVector4(&self) -> Result<Vector4> {
        Ok(from_native_vector4(
            self.get_one(sys::CNA_EFFECT_VALUE_VECTOR4)?,
        ))
    }
    pub fn GetValueVector4Array(&self, count: i32) -> Result<Vec<Vector4>> {
        Ok(self
            .get_many::<sys::CNA_Vector4>(sys::CNA_EFFECT_VALUE_VECTOR4, count)?
            .into_iter()
            .map(from_native_vector4)
            .collect())
    }
    pub fn GetValueQuaternion(&self) -> Result<Quaternion> {
        Ok(from_native_quaternion(
            self.get_one(sys::CNA_EFFECT_VALUE_QUATERNION)?,
        ))
    }
    pub fn GetValueQuaternionArray(&self, count: i32) -> Result<Vec<Quaternion>> {
        Ok(self
            .get_many::<sys::CNA_Quaternion>(sys::CNA_EFFECT_VALUE_QUATERNION, count)?
            .into_iter()
            .map(from_native_quaternion)
            .collect())
    }
    pub fn GetValueMatrix(&self) -> Result<Matrix> {
        Ok(from_native_matrix(
            self.get_one(sys::CNA_EFFECT_VALUE_MATRIX)?,
        ))
    }
    pub fn GetValueMatrixArray(&self, count: i32) -> Result<Vec<Matrix>> {
        Ok(self
            .get_many::<sys::CNA_Matrix>(sys::CNA_EFFECT_VALUE_MATRIX, count)?
            .into_iter()
            .map(from_native_matrix)
            .collect())
    }
    pub fn GetValueMatrixTranspose(&self) -> Result<Matrix> {
        Ok(from_native_matrix(
            self.get_one(sys::CNA_EFFECT_VALUE_MATRIX_TRANSPOSE)?,
        ))
    }
    pub fn GetValueMatrixTransposeArray(&self, count: i32) -> Result<Vec<Matrix>> {
        Ok(self
            .get_many::<sys::CNA_Matrix>(sys::CNA_EFFECT_VALUE_MATRIX_TRANSPOSE, count)?
            .into_iter()
            .map(from_native_matrix)
            .collect())
    }
    pub fn GetValueString(&self) -> Result<String> {
        self.native()
            .effect_parameter_string(self.state.require_handle()?)
    }
    pub fn GetValueTexture2D(&self) -> Result<Option<Arc<dyn Texture>>> {
        self.get_texture(sys::CNA_EFFECT_TEXTURE_2D)
    }
    pub fn GetValueTextureCube(&self) -> Result<Option<Arc<dyn Texture>>> {
        self.get_texture(sys::CNA_EFFECT_TEXTURE_CUBE)
    }
    pub fn GetValueTexture3D(&self) -> Result<Option<Arc<dyn Texture>>> {
        self.get_texture(sys::CNA_EFFECT_TEXTURE_3D)
    }

    fn get_texture(
        &self,
        texture_type: sys::CNA_EffectTextureType,
    ) -> Result<Option<Arc<dyn Texture>>> {
        let native_handle = self
            .native()
            .effect_parameter_texture(self.state.require_handle()?, texture_type)?;
        if native_handle == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        let retained = self
            .retained_texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::UnsupportedRuntime(
                "CNA returned an Effect texture not assigned through this safe wrapper",
            ))?;
        if texture_handle(retained.as_ref(), self.state.owner.device())? != native_handle {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "Effect texture identity changed outside the safe wrapper".to_owned(),
            });
        }
        Ok(Some(retained))
    }

    fn set_one<T: Copy>(&self, value_type: sys::CNA_EffectValueType, value: T) -> Result<()> {
        self.native()
            .set_effect_parameter_value(self.state.require_handle()?, value_type, &value)
    }
    fn set_slice<T: Copy>(&self, value_type: sys::CNA_EffectValueType, value: &[T]) -> Result<()> {
        self.native()
            .set_effect_parameter_values(self.state.require_handle()?, value_type, value)
    }
    fn set_many<T: Copy>(&self, value_type: sys::CNA_EffectValueType, value: Vec<T>) -> Result<()> {
        self.set_slice(value_type, &value)
    }
    fn set_matrices(&self, value: &[Matrix], value_type: sys::CNA_EffectValueType) -> Result<()> {
        self.set_many(
            value_type,
            value.iter().copied().map(native_matrix).collect::<Vec<_>>(),
        )
    }
    fn get_one<T: Copy + Default>(&self, value_type: sys::CNA_EffectValueType) -> Result<T> {
        self.native()
            .effect_parameter_value(self.state.require_handle()?, value_type)
    }
    fn get_many<T: Copy + Default>(
        &self,
        value_type: sys::CNA_EffectValueType,
        count: i32,
    ) -> Result<Vec<T>> {
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("effect array count cannot be negative"))?;
        self.native()
            .effect_parameter_values(self.state.require_handle()?, value_type, count)
    }
}

#[derive(Clone)]
pub struct EffectParameterCollection {
    state: Arc<EffectViewState>,
    items: Arc<Vec<Arc<EffectParameter>>>,
}

#[allow(non_snake_case)]
impl EffectParameterCollection {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, PARAMETER_COLLECTION);
        let count = collection_count(&state, PARAMETER)?;
        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let child = state
                .owner
                .device()
                .state
                .native()
                .effect_collection_get_at(state.require_handle()?, PARAMETER, index as u64)?;
            items.push(Arc::new(EffectParameter::from_handle(
                Arc::clone(state.owner()),
                child,
            )?));
        }
        Ok(Self {
            state,
            items: Arc::new(items),
        })
    }
    pub fn Item(&self, name: &str) -> Result<Option<Arc<EffectParameter>>> {
        let found = verified_find(&self.state, PARAMETER, name, false)?;
        let cached = self
            .items
            .iter()
            .find(|value| value.name == name)
            .map(Arc::clone);
        verified_cached_lookup(found, cached)
    }
    pub fn Count(&self) -> Result<i32> {
        self.state.require_handle()?;
        checked_count(self.items.len())
    }
    pub fn GetParameterBySemantic(&self, semantic: &str) -> Result<Option<Arc<EffectParameter>>> {
        let found = verified_find(&self.state, PARAMETER, semantic, true)?;
        let cached = self
            .items
            .iter()
            .find(|value| value.semantic == semantic)
            .map(Arc::clone);
        verified_cached_lookup(found, cached)
    }
    pub fn GetEnumerator(&self) -> Result<IntoIter<Arc<EffectParameter>>> {
        self.state.require_handle()?;
        Ok(self.items.as_ref().clone().into_iter())
    }
    pub(crate) fn item_at(&self, index: i32) -> Result<Arc<EffectParameter>> {
        collection_item_at(&self.state, &self.items, index)
    }
}

impl IntoIterator for EffectParameterCollection {
    type Item = Arc<EffectParameter>;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.as_ref().clone().into_iter()
    }
}

pub struct EffectPass {
    state: Arc<EffectViewState>,
    name: String,
    annotations: Mutex<Option<Arc<EffectAnnotationCollection>>>,
}

#[allow(non_snake_case)]
impl EffectPass {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, PASS);
        let name = state
            .owner
            .device()
            .state
            .native()
            .effect_pass_name(state.require_handle()?)?;
        Ok(Self {
            state,
            name,
            annotations: Mutex::new(None),
        })
    }
    pub fn Annotations(&self) -> Result<Arc<EffectAnnotationCollection>> {
        let mut cache = self
            .annotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cache.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .owner
            .device()
            .state
            .native()
            .effect_pass_annotations(self.state.require_handle()?, &mut handle)?;
        let value = Arc::new(EffectAnnotationCollection::from_handle(
            Arc::clone(self.state.owner()),
            handle,
        )?);
        *cache = Some(Arc::clone(&value));
        Ok(value)
    }
    pub fn Name(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.name.clone())
    }
    pub fn Apply(&self) -> Result<()> {
        self.state
            .owner
            .device()
            .state
            .native()
            .apply_effect_pass(self.state.require_handle()?)
    }
}

#[derive(Clone)]
pub struct EffectPassCollection {
    state: Arc<EffectViewState>,
    items: Arc<Vec<Arc<EffectPass>>>,
}

#[allow(non_snake_case)]
impl EffectPassCollection {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, PASS_COLLECTION);
        let count = collection_count(&state, PASS)?;
        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let child = state
                .owner
                .device()
                .state
                .native()
                .effect_collection_get_at(state.require_handle()?, PASS, index as u64)?;
            items.push(Arc::new(EffectPass::from_handle(
                Arc::clone(state.owner()),
                child,
            )?));
        }
        Ok(Self {
            state,
            items: Arc::new(items),
        })
    }
    pub fn Item(&self, name: &str) -> Result<Option<Arc<EffectPass>>> {
        let found = verified_find(&self.state, PASS, name, false)?;
        let cached = self
            .items
            .iter()
            .find(|value| value.name == name)
            .map(Arc::clone);
        verified_cached_lookup(found, cached)
    }
    pub fn Count(&self) -> Result<i32> {
        self.state.require_handle()?;
        checked_count(self.items.len())
    }
    pub fn GetEnumerator(&self) -> Result<IntoIter<Arc<EffectPass>>> {
        self.state.require_handle()?;
        Ok(self.items.as_ref().clone().into_iter())
    }
    pub(crate) fn item_at(&self, index: i32) -> Result<Arc<EffectPass>> {
        collection_item_at(&self.state, &self.items, index)
    }
}

impl IntoIterator for EffectPassCollection {
    type Item = Arc<EffectPass>;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.as_ref().clone().into_iter()
    }
}

pub struct EffectTechnique {
    state: Arc<EffectViewState>,
    name: String,
    annotations: Mutex<Option<Arc<EffectAnnotationCollection>>>,
    passes: Mutex<Option<Arc<EffectPassCollection>>>,
}

#[allow(non_snake_case)]
impl EffectTechnique {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, TECHNIQUE);
        let name = state
            .owner
            .device()
            .state
            .native()
            .effect_technique_name(state.require_handle()?)?;
        Ok(Self {
            state,
            name,
            annotations: Mutex::new(None),
            passes: Mutex::new(None),
        })
    }
    pub fn Annotations(&self) -> Result<Arc<EffectAnnotationCollection>> {
        let mut cache = self
            .annotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cache.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .owner
            .device()
            .state
            .native()
            .effect_technique_annotations(self.state.require_handle()?, &mut handle)?;
        let value = Arc::new(EffectAnnotationCollection::from_handle(
            Arc::clone(self.state.owner()),
            handle,
        )?);
        *cache = Some(Arc::clone(&value));
        Ok(value)
    }
    pub fn Passes(&self) -> Result<Arc<EffectPassCollection>> {
        let mut cache = self
            .passes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = cache.as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state
            .owner
            .device()
            .state
            .native()
            .effect_technique_passes(self.state.require_handle()?, &mut handle)?;
        let value = Arc::new(EffectPassCollection::from_handle(
            Arc::clone(self.state.owner()),
            handle,
        )?);
        *cache = Some(Arc::clone(&value));
        Ok(value)
    }
    pub fn Name(&self) -> Result<String> {
        self.state.require_handle()?;
        Ok(self.name.clone())
    }
}

#[derive(Clone)]
pub struct EffectTechniqueCollection {
    state: Arc<EffectViewState>,
    items: Arc<Vec<Arc<EffectTechnique>>>,
}

#[allow(non_snake_case)]
impl EffectTechniqueCollection {
    fn from_handle(owner: Arc<ResourceState>, handle: sys::CNA_Handle) -> Result<Self> {
        let state = EffectViewState::new(owner, handle, TECHNIQUE_COLLECTION);
        let count = collection_count(&state, TECHNIQUE)?;
        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let child = state
                .owner
                .device()
                .state
                .native()
                .effect_collection_get_at(state.require_handle()?, TECHNIQUE, index as u64)?;
            items.push(Arc::new(EffectTechnique::from_handle(
                Arc::clone(state.owner()),
                child,
            )?));
        }
        Ok(Self {
            state,
            items: Arc::new(items),
        })
    }
    pub fn Item(&self, name: &str) -> Result<Option<Arc<EffectTechnique>>> {
        let found = verified_find(&self.state, TECHNIQUE, name, false)?;
        let cached = self
            .items
            .iter()
            .find(|value| value.name == name)
            .map(Arc::clone);
        verified_cached_lookup(found, cached)
    }
    pub fn Count(&self) -> Result<i32> {
        self.state.require_handle()?;
        checked_count(self.items.len())
    }
    pub fn GetEnumerator(&self) -> Result<IntoIter<Arc<EffectTechnique>>> {
        self.state.require_handle()?;
        Ok(self.items.as_ref().clone().into_iter())
    }
    pub(crate) fn item_at(&self, index: i32) -> Result<Arc<EffectTechnique>> {
        collection_item_at(&self.state, &self.items, index)
    }
}

impl IntoIterator for EffectTechniqueCollection {
    type Item = Arc<EffectTechnique>;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.as_ref().clone().into_iter()
    }
}

fn collection_count(state: &EffectViewState, kind: u8) -> Result<usize> {
    usize::try_from(
        state
            .owner
            .device()
            .state
            .native()
            .effect_collection_count(state.require_handle()?, kind)?,
    )
    .map_err(|_| CnaError::Native {
        code: sys::CNA_RESULT_OVERFLOW,
        message: "effect collection count exceeds Rust address space".to_owned(),
    })
}

fn verified_find(state: &EffectViewState, kind: u8, value: &str, semantic: bool) -> Result<bool> {
    let found = state.owner.device().state.native().effect_collection_find(
        state.require_handle()?,
        kind,
        string_view(value)?,
        semantic,
    )?;
    if let Some(handle) = found {
        state
            .owner
            .device()
            .state
            .native()
            .destroy_effect_view(handle, kind)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn verified_cached_lookup<T>(found: bool, cached: Option<Arc<T>>) -> Result<Option<Arc<T>>> {
    if found == cached.is_some() {
        Ok(cached)
    } else {
        Err(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            message: "CNA effect lookup disagreed with the stable Rust collection cache".to_owned(),
        })
    }
}

fn checked_count(count: usize) -> Result<i32> {
    i32::try_from(count).map_err(|_| CnaError::Native {
        code: sys::CNA_RESULT_OVERFLOW,
        message: "effect collection count exceeds XNA Int32".to_owned(),
    })
}

fn collection_item_at<T>(state: &EffectViewState, items: &[Arc<T>], index: i32) -> Result<Arc<T>> {
    state.require_handle()?;
    let index = usize::try_from(index)
        .map_err(|_| CnaError::InvalidInput("effect collection index cannot be negative"))?;
    items
        .get(index)
        .map(Arc::clone)
        .ok_or(CnaError::InvalidInput(
            "effect collection index is out of range",
        ))
}

pub(crate) fn native_matrix(value: Matrix) -> sys::CNA_Matrix {
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

fn from_native_matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix::new(
        value.m11, value.m12, value.m13, value.m14, value.m21, value.m22, value.m23, value.m24,
        value.m31, value.m32, value.m33, value.m34, value.m41, value.m42, value.m43, value.m44,
    )
}

fn native_quaternion(value: Quaternion) -> sys::CNA_Quaternion {
    sys::CNA_Quaternion {
        x: value.X,
        y: value.Y,
        z: value.Z,
        w: value.W,
    }
}
fn from_native_quaternion(value: sys::CNA_Quaternion) -> Quaternion {
    Quaternion::from_x_and_y_and_z_and_w(value.x, value.y, value.z, value.w)
}
fn native_vector3(value: Vector3) -> sys::CNA_Vector3 {
    sys::CNA_Vector3 {
        x: value.X,
        y: value.Y,
        z: value.Z,
    }
}
fn from_native_vector3(value: sys::CNA_Vector3) -> Vector3 {
    Vector3::from_x_and_y_and_z(value.x, value.y, value.z)
}
fn native_vector4(value: Vector4) -> sys::CNA_Vector4 {
    sys::CNA_Vector4 {
        x: value.X,
        y: value.Y,
        z: value.Z,
        w: value.W,
    }
}
fn from_native_vector4(value: sys::CNA_Vector4) -> Vector4 {
    Vector4::from_x_and_y_and_z_and_w(value.x, value.y, value.z, value.w)
}

fn string_view(value: &str) -> Result<sys::CNA_StringView> {
    Ok(sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: u64::try_from(value.len())
            .map_err(|_| CnaError::InvalidInput("effect string is too large"))?,
    })
}

fn texture_handle(texture: &dyn Texture, device: &GraphicsDevice) -> Result<sys::CNA_Handle> {
    if texture
        .GraphicsDevice()
        .map_or(true, |owner| !owner.is_same_device(device))
    {
        return Err(CnaError::InvalidInput(
            "effect texture belongs to a different graphics device",
        ));
    }
    let any = texture.as_any();
    if let Some(value) = any.downcast_ref::<Texture2D>() {
        value.handle()
    } else if let Some(value) = any.downcast_ref::<TextureCube>() {
        value.handle()
    } else if let Some(value) = any.downcast_ref::<RenderTarget2D>() {
        value.handle()
    } else if let Some(value) = any.downcast_ref::<RenderTargetCube>() {
        value.handle()
    } else {
        Err(CnaError::UnsupportedRuntime(
            "this concrete Texture has no reviewed CNA Effect binding route",
        ))
    }
}
