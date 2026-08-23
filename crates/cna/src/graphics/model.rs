#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::value::{BoundingSphere, Matrix};

use super::{
    Effect, EffectBase, GraphicsDevice, GraphicsResource, IndexBuffer, PrimitiveType, VertexBuffer,
};

pub(crate) type TagValue = Arc<dyn Any + Send + Sync>;

struct ModelLifetime {
    alive: AtomicBool,
}

impl ModelLifetime {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            alive: AtomicBool::new(true),
        })
    }

    fn ensure(&self) -> Result<()> {
        if self.alive.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(CnaError::InvalidInput("model graph is no longer live"))
        }
    }

    fn invalidate(&self) {
        self.alive.store(false, Ordering::Release);
    }
}

pub(crate) type ModelEffectReference = Arc<dyn EffectBase>;

fn model_effect(value: &ModelEffectReference) -> &Effect {
    value.AsEffect()
}

fn model_effect_identity(value: &ModelEffectReference) -> Result<u64> {
    model_effect(value).handle()
}

pub(crate) struct ModelBoneSpec {
    pub name: String,
    pub transform: Matrix,
    pub parent: Option<usize>,
}

pub(crate) struct ModelMeshPartSpec {
    pub vertex_buffer: Arc<VertexBuffer>,
    pub index_buffer: Arc<IndexBuffer>,
    pub effect: Option<ModelEffectReference>,
    pub tag: Option<TagValue>,
    pub vertex_offset: i32,
    pub num_vertices: i32,
    pub start_index: i32,
    pub primitive_count: i32,
}

pub(crate) struct ModelMeshSpec {
    pub name: String,
    pub parent_bone: usize,
    pub bounding_sphere: BoundingSphere,
    pub tag: Option<TagValue>,
    pub parts: Vec<ModelMeshPartSpec>,
}

pub(crate) struct ModelMeshPartPendingSpec {
    pub vertex_buffer: Arc<Mutex<Option<Arc<VertexBuffer>>>>,
    pub index_buffer: Arc<Mutex<Option<Arc<IndexBuffer>>>>,
    pub effect: Arc<Mutex<Option<ModelEffectReference>>>,
    pub tag: Option<TagValue>,
    pub vertex_offset: i32,
    pub num_vertices: i32,
    pub start_index: i32,
    pub primitive_count: i32,
}

pub(crate) struct ModelMeshPendingSpec {
    pub name: String,
    pub parent_bone: usize,
    pub bounding_sphere: BoundingSphere,
    pub tag: Option<TagValue>,
    pub parts: Vec<ModelMeshPartPendingSpec>,
}

struct ModelPending {
    device: GraphicsDevice,
    bone_specs: Vec<ModelBoneSpec>,
    mesh_specs: Vec<ModelMeshPendingSpec>,
    root_index: usize,
}

struct ModelGraph {
    bones: Arc<ModelBoneCollection>,
    meshes: Arc<ModelMeshCollection>,
    root: Arc<ModelBone>,
}

/// Managed XNA model graph assembled by the ordinary XNB reader path.
pub struct Model {
    lifetime: Arc<ModelLifetime>,
    graph: OnceLock<ModelGraph>,
    pending: Mutex<Option<ModelPending>>,
    tag: Mutex<Option<TagValue>>,
}

impl Model {
    pub(crate) fn from_pending(
        device: &GraphicsDevice,
        bone_specs: Vec<ModelBoneSpec>,
        mesh_specs: Vec<ModelMeshPendingSpec>,
        root_index: usize,
        tag: Option<TagValue>,
    ) -> Result<Self> {
        validate_bone_graph(&bone_specs, root_index)?;
        Ok(Self {
            lifetime: ModelLifetime::new(),
            graph: OnceLock::new(),
            pending: Mutex::new(Some(ModelPending {
                device: device.clone(),
                bone_specs,
                mesh_specs,
                root_index,
            })),
            tag: Mutex::new(tag),
        })
    }

    pub(crate) fn finalize_pending(&self) -> Result<()> {
        if self.graph.get().is_some() {
            return Ok(());
        }
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(CnaError::InvalidInput("model graph has no pending state"))?;
        let mut mesh_specs = Vec::with_capacity(pending.mesh_specs.len());
        for mesh in pending.mesh_specs {
            let mut parts = Vec::with_capacity(mesh.parts.len());
            for part in mesh.parts {
                let vertex_buffer = part
                    .vertex_buffer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
                    .ok_or(CnaError::InvalidInput(
                        "model mesh part has no vertex buffer",
                    ))?;
                let index_buffer = part
                    .index_buffer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
                    .ok_or(CnaError::InvalidInput(
                        "model mesh part has no index buffer",
                    ))?;
                let effect = part
                    .effect
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if effect.is_none() {
                    return Err(CnaError::InvalidInput("model mesh part has no effect"));
                }
                parts.push(ModelMeshPartSpec {
                    vertex_buffer,
                    index_buffer,
                    effect,
                    tag: part.tag,
                    vertex_offset: part.vertex_offset,
                    num_vertices: part.num_vertices,
                    start_index: part.start_index,
                    primitive_count: part.primitive_count,
                });
            }
            mesh_specs.push(ModelMeshSpec {
                name: mesh.name,
                parent_bone: mesh.parent_bone,
                bounding_sphere: mesh.bounding_sphere,
                tag: mesh.tag,
                parts,
            });
        }
        let graph = Self::build_graph(
            &self.lifetime,
            &pending.device,
            &pending.bone_specs,
            mesh_specs,
            pending.root_index,
        )?;
        self.graph
            .set(graph)
            .map_err(|_| CnaError::InvalidInput("model graph was initialized twice"))
    }

    #[allow(clippy::too_many_lines)]
    fn build_graph(
        lifetime: &Arc<ModelLifetime>,
        device: &GraphicsDevice,
        bone_specs: &[ModelBoneSpec],
        mesh_specs: Vec<ModelMeshSpec>,
        root_index: usize,
    ) -> Result<ModelGraph> {
        validate_bone_graph(bone_specs, root_index)?;
        if bone_specs.is_empty() {
            return Err(CnaError::InvalidInput(
                "a model must contain at least one bone",
            ));
        }
        if root_index >= bone_specs.len() {
            return Err(CnaError::InvalidInput(
                "model root bone index is out of range",
            ));
        }
        let mut bones = Vec::with_capacity(bone_specs.len());
        for (index, spec) in bone_specs.iter().enumerate() {
            let index = i32::try_from(index)
                .map_err(|_| CnaError::InvalidInput("model has too many bones"))?;
            bones.push(Arc::new(ModelBone {
                lifetime: Arc::clone(lifetime),
                index,
                name: spec.name.clone(),
                transform: Mutex::new(spec.transform),
                parent: Mutex::new(None),
                children: OnceLock::new(),
            }));
        }
        for (index, spec) in bone_specs.iter().enumerate() {
            if let Some(parent_index) = spec.parent {
                if parent_index >= index {
                    return Err(CnaError::InvalidInput(
                        "model bone parents must precede their children",
                    ));
                }
                *bones[index]
                    .parent
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some(Arc::downgrade(&bones[parent_index]));
            }
        }
        let mut child_lists = vec![Vec::new(); bones.len()];
        for (index, spec) in bone_specs.iter().enumerate() {
            if let Some(parent_index) = spec.parent {
                child_lists[parent_index].push(Arc::clone(&bones[index]));
            }
        }
        for (bone, children) in bones.iter().zip(child_lists) {
            bone.children
                .set(Arc::new(ModelBoneCollection {
                    lifetime: Arc::clone(lifetime),
                    items: children,
                }))
                .map_err(|_| {
                    CnaError::InvalidInput("model bone children were initialized twice")
                })?;
        }
        let bones_collection = Arc::new(ModelBoneCollection {
            lifetime: Arc::clone(lifetime),
            items: bones.clone(),
        });

        let mut meshes = Vec::with_capacity(mesh_specs.len());
        for spec in mesh_specs {
            let parent_bone = bones.get(spec.parent_bone).ok_or(CnaError::InvalidInput(
                "model mesh parent bone index is out of range",
            ))?;
            let effects = Arc::new(Mutex::new(Vec::new()));
            let mut parts = Vec::with_capacity(spec.parts.len());
            for part in spec.parts {
                if !part.vertex_buffer.is_same_device(device)
                    || !part.index_buffer.is_same_device(device)
                    || part
                        .effect
                        .as_ref()
                        .is_some_and(|effect| !model_effect(effect).is_same_device(device))
                {
                    return Err(CnaError::InvalidInput(
                        "model resources must belong to the model graphics device",
                    ));
                }
                if part.vertex_offset < 0
                    || part.num_vertices < 0
                    || part.start_index < 0
                    || part.primitive_count < 0
                {
                    return Err(CnaError::InvalidInput(
                        "model mesh-part ranges must not be negative",
                    ));
                }
                let effect = part.effect;
                if let Some(value) = &effect {
                    add_unique_effect(&effects, Arc::clone(value))?;
                }
                parts.push(Arc::new(ModelMeshPart {
                    lifetime: Arc::clone(lifetime),
                    effects: Arc::downgrade(&effects),
                    siblings: OnceLock::new(),
                    effect: Mutex::new(effect),
                    vertex_buffer: part.vertex_buffer,
                    index_buffer: part.index_buffer,
                    tag: Mutex::new(part.tag),
                    vertex_offset: part.vertex_offset,
                    num_vertices: part.num_vertices,
                    start_index: part.start_index,
                    primitive_count: part.primitive_count,
                }));
            }
            let parts = Arc::new(ModelMeshPartCollection {
                lifetime: Arc::clone(lifetime),
                items: parts,
            });
            for part in &parts.items {
                part.siblings.set(Arc::downgrade(&parts)).map_err(|_| {
                    CnaError::InvalidInput("model part graph was initialized twice")
                })?;
            }
            meshes.push(Arc::new(ModelMesh {
                lifetime: Arc::clone(lifetime),
                device: device.clone(),
                name: spec.name,
                parent_bone: Arc::clone(parent_bone),
                bounding_sphere: spec.bounding_sphere,
                tag: Mutex::new(spec.tag),
                parts,
                effects: Arc::new(ModelEffectCollection {
                    lifetime: Arc::clone(lifetime),
                    items: effects,
                }),
            }));
        }
        Ok(ModelGraph {
            bones: bones_collection,
            meshes: Arc::new(ModelMeshCollection {
                lifetime: Arc::clone(lifetime),
                items: meshes,
            }),
            root: Arc::clone(&bones[root_index]),
        })
    }

    fn graph(&self) -> Result<&ModelGraph> {
        self.lifetime.ensure()?;
        self.graph
            .get()
            .ok_or(CnaError::InvalidInput("model graph is not finalized"))
    }

    pub fn Bones(&self) -> Result<&ModelBoneCollection> {
        Ok(&self.graph()?.bones)
    }

    pub fn Meshes(&self) -> Result<&ModelMeshCollection> {
        Ok(&self.graph()?.meshes)
    }

    pub fn Root(&self) -> Result<&ModelBone> {
        Ok(&self.graph()?.root)
    }

    pub fn Tag(&self) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.lifetime.ensure()?;
        Ok(self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }

    pub fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {
        self.lifetime.ensure()?;
        *self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }

    pub fn CopyBoneTransformsTo(&self, destinationBoneTransforms: &mut [Matrix]) -> Result<()> {
        self.lifetime.ensure()?;
        let graph = self.graph()?;
        if destinationBoneTransforms.len() < graph.bones.items.len() {
            return Err(CnaError::InvalidInput(
                "destination bone transform slice is too small",
            ));
        }
        for (destination, bone) in destinationBoneTransforms.iter_mut().zip(&graph.bones.items) {
            *destination = bone.Transform()?;
        }
        Ok(())
    }

    pub fn CopyBoneTransformsFrom(&self, sourceBoneTransforms: &[Matrix]) -> Result<()> {
        self.lifetime.ensure()?;
        let graph = self.graph()?;
        if sourceBoneTransforms.len() < graph.bones.items.len() {
            return Err(CnaError::InvalidInput(
                "source bone transform slice is too small",
            ));
        }
        for (source, bone) in sourceBoneTransforms.iter().zip(&graph.bones.items) {
            *bone
                .transform
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = *source;
        }
        Ok(())
    }

    pub fn CopyAbsoluteBoneTransformsTo(
        &self,
        destinationBoneTransforms: &mut [Matrix],
    ) -> Result<()> {
        self.lifetime.ensure()?;
        let graph = self.graph()?;
        if destinationBoneTransforms.len() < graph.bones.items.len() {
            return Err(CnaError::InvalidInput(
                "destination bone transform slice is too small",
            ));
        }
        for (index, bone) in graph.bones.items.iter().enumerate() {
            let transform = bone.Transform()?;
            let parent = bone
                .parent
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .and_then(Weak::upgrade);
            destinationBoneTransforms[index] = if let Some(parent) = parent {
                let parent_index = usize::try_from(parent.index)
                    .map_err(|_| CnaError::InvalidInput("model bone parent index is negative"))?;
                if parent_index >= index {
                    return Err(CnaError::InvalidInput(
                        "model bone parent must precede its child",
                    ));
                }
                transform * destinationBoneTransforms[parent_index]
            } else {
                transform
            };
        }
        Ok(())
    }

    pub fn Draw(&self, world: Matrix, view: Matrix, projection: Matrix) -> Result<()> {
        self.lifetime.ensure()?;
        let graph = self.graph()?;
        let mut absolute = vec![Matrix::Identity; graph.bones.items.len()];
        self.CopyAbsoluteBoneTransformsTo(&mut absolute)?;
        for mesh in &graph.meshes.items {
            let parent_index = usize::try_from(mesh.parent_bone.Index()?)
                .map_err(|_| CnaError::InvalidInput("mesh parent bone index is negative"))?;
            let bone_world = *absolute.get(parent_index).ok_or(CnaError::InvalidInput(
                "mesh parent bone index is out of range",
            ))? * world;
            for effect in mesh.effects.snapshot()? {
                effect.set_model_matrices_for_model(bone_world, view, projection)?;
            }
            mesh.Draw()?;
        }
        Ok(())
    }

    pub(crate) fn invalidate(&self) {
        self.lifetime.invalidate();
    }

    fn release_bound_model_buffers(&self) -> Result<()> {
        let Some(graph) = self.graph.get() else {
            return Ok(());
        };
        let Some(mesh) = graph.meshes.items.first() else {
            return Ok(());
        };
        let mut vertex_handles = Vec::new();
        let mut index_handles = Vec::new();
        for mesh in &graph.meshes.items {
            for part in &mesh.parts.items {
                if let Ok(handle) = part.vertex_buffer.handle() {
                    vertex_handles.push(handle);
                }
                if let Ok(handle) = part.index_buffer.handle() {
                    index_handles.push(handle);
                }
            }
        }
        if mesh
            .device
            .has_bound_buffer_handle(&vertex_handles, &index_handles)
        {
            mesh.device.unbind_all_buffers()?;
        }
        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.lifetime.invalidate();
    }
}

impl ContentDisposable for Model {
    fn DisposeContent(&self) -> Result<()> {
        let release = self.release_bound_model_buffers();
        self.invalidate();
        release
    }
}

impl ContentLoadable for Model {}

pub struct ModelBone {
    lifetime: Arc<ModelLifetime>,
    index: i32,
    name: String,
    transform: Mutex<Matrix>,
    parent: Mutex<Option<Weak<ModelBone>>>,
    children: OnceLock<Arc<ModelBoneCollection>>,
}

impl ModelBone {
    pub fn Children(&self) -> Result<&ModelBoneCollection> {
        self.lifetime.ensure()?;
        self.children
            .get()
            .map(AsRef::as_ref)
            .ok_or(CnaError::InvalidInput(
                "model bone children are not initialized",
            ))
    }
    pub fn Index(&self) -> Result<i32> {
        self.lifetime.ensure()?;
        Ok(self.index)
    }
    pub fn Name(&self) -> Result<String> {
        self.lifetime.ensure()?;
        Ok(self.name.clone())
    }
    pub fn Parent(&self) -> Result<Option<Arc<ModelBone>>> {
        self.lifetime.ensure()?;
        Ok(self
            .parent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade))
    }
    pub fn Transform(&self) -> Result<Matrix> {
        self.lifetime.ensure()?;
        Ok(*self
            .transform
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
    }
    pub fn SetTransform(&mut self, value: Matrix) -> Result<()> {
        self.lifetime.ensure()?;
        *self
            .transform
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
}

pub struct ModelBoneCollection {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelBone>>,
}

impl ModelBoneCollection {
    pub fn Item(&self, boneName: &str) -> Result<&ModelBone> {
        self.lifetime.ensure()?;
        validate_model_name(boneName)?;
        self.items
            .iter()
            .find(|bone| bone.name == boneName)
            .map(AsRef::as_ref)
            .ok_or(CnaError::InvalidInput("model bone name was not found"))
    }
    pub fn TryGetValue(&self, boneName: &str, value: &mut Option<Arc<ModelBone>>) -> Result<bool> {
        self.lifetime.ensure()?;
        validate_model_name(boneName)?;
        *value = self
            .items
            .iter()
            .find(|bone| bone.name == boneName)
            .map(Arc::clone);
        Ok(value.is_some())
    }
    pub fn GetEnumerator(&self) -> Result<ModelBoneCollectionEnumerator> {
        self.lifetime.ensure()?;
        Ok(ModelBoneCollectionEnumerator::new(
            Arc::clone(&self.lifetime),
            self.items.clone(),
        ))
    }
    pub(crate) fn count(&self) -> usize {
        self.items.len()
    }
    pub(crate) fn item_at(&self, index: usize) -> Result<Arc<ModelBone>> {
        self.lifetime.ensure()?;
        self.items
            .get(index)
            .map(Arc::clone)
            .ok_or(CnaError::InvalidInput("model bone index is out of range"))
    }
}

#[derive(Clone)]
pub struct ModelBoneCollectionEnumerator {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelBone>>,
    position: Option<usize>,
    disposed: bool,
}
impl ModelBoneCollectionEnumerator {
    fn new(lifetime: Arc<ModelLifetime>, items: Vec<Arc<ModelBone>>) -> Self {
        Self {
            lifetime,
            items,
            position: None,
            disposed: false,
        }
    }
    pub fn MoveNext(&mut self) -> Result<bool> {
        enumerator_move_next(
            &self.lifetime,
            self.items.len(),
            &mut self.position,
            self.disposed,
        )
    }
    pub fn Current(&self) -> Result<&ModelBone> {
        enumerator_current(&self.lifetime, &self.items, self.position, self.disposed)
    }
    pub fn Dispose(&mut self) -> Result<()> {
        self.disposed = true;
        self.position = None;
        Ok(())
    }
}
impl Drop for ModelBoneCollectionEnumerator {
    fn drop(&mut self) {}
}
impl PartialEq for ModelBoneCollectionEnumerator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifetime, &other.lifetime)
            && self.position == other.position
            && self.disposed == other.disposed
    }
}
impl Iterator for ModelBoneCollectionEnumerator {
    type Item = Arc<ModelBone>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.disposed || self.lifetime.ensure().is_err() {
            return None;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        self.items.get(next).map(Arc::clone)
    }
}

pub struct ModelMesh {
    lifetime: Arc<ModelLifetime>,
    device: GraphicsDevice,
    name: String,
    parent_bone: Arc<ModelBone>,
    bounding_sphere: BoundingSphere,
    tag: Mutex<Option<TagValue>>,
    parts: Arc<ModelMeshPartCollection>,
    effects: Arc<ModelEffectCollection>,
}
impl ModelMesh {
    pub fn BoundingSphere(&self) -> Result<BoundingSphere> {
        self.lifetime.ensure()?;
        Ok(self.bounding_sphere)
    }
    pub fn Effects(&self) -> Result<&ModelEffectCollection> {
        self.lifetime.ensure()?;
        Ok(&self.effects)
    }
    pub fn MeshParts(&self) -> Result<&ModelMeshPartCollection> {
        self.lifetime.ensure()?;
        Ok(&self.parts)
    }
    pub fn Name(&self) -> Result<String> {
        self.lifetime.ensure()?;
        Ok(self.name.clone())
    }
    pub fn ParentBone(&self) -> Result<&ModelBone> {
        self.lifetime.ensure()?;
        Ok(&self.parent_bone)
    }
    pub fn Tag(&self) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.lifetime.ensure()?;
        Ok(self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {
        self.lifetime.ensure()?;
        *self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn Draw(&self) -> Result<()> {
        self.lifetime.ensure()?;
        let mut device = self.device.clone();
        for part in &self.parts.items {
            if part.primitive_count <= 0 {
                continue;
            }
            let effect = part
                .effect
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let Some(effect) = effect else { continue };
            device.SetVertexBufferWithVertexBuffer(&part.vertex_buffer)?;
            device.SetIndices(Some(&part.index_buffer))?;
            let technique = model_effect(&effect).CurrentTechnique()?;
            for pass in technique.Passes()?.GetEnumerator()? {
                pass.Apply()?;
                device.DrawIndexedPrimitives(
                    PrimitiveType::TriangleList,
                    part.vertex_offset,
                    0,
                    part.num_vertices,
                    part.start_index,
                    part.primitive_count,
                )?;
            }
        }
        Ok(())
    }
}

pub struct ModelMeshCollection {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelMesh>>,
}
impl ModelMeshCollection {
    pub fn Item(&self, meshName: &str) -> Result<&ModelMesh> {
        self.lifetime.ensure()?;
        validate_model_name(meshName)?;
        self.items
            .iter()
            .find(|mesh| mesh.name == meshName)
            .map(AsRef::as_ref)
            .ok_or(CnaError::InvalidInput("model mesh name was not found"))
    }
    pub fn TryGetValue(&self, meshName: &str, value: &mut Option<Arc<ModelMesh>>) -> Result<bool> {
        self.lifetime.ensure()?;
        validate_model_name(meshName)?;
        *value = self
            .items
            .iter()
            .find(|mesh| mesh.name == meshName)
            .map(Arc::clone);
        Ok(value.is_some())
    }
    pub fn GetEnumerator(&self) -> Result<ModelMeshCollectionEnumerator> {
        self.lifetime.ensure()?;
        Ok(ModelMeshCollectionEnumerator::new(
            Arc::clone(&self.lifetime),
            self.items.clone(),
        ))
    }
    pub(crate) fn count(&self) -> usize {
        self.items.len()
    }
    pub(crate) fn item_at(&self, index: usize) -> Result<Arc<ModelMesh>> {
        self.lifetime.ensure()?;
        self.items
            .get(index)
            .map(Arc::clone)
            .ok_or(CnaError::InvalidInput("model mesh index is out of range"))
    }
}

#[derive(Clone)]
pub struct ModelMeshCollectionEnumerator {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelMesh>>,
    position: Option<usize>,
    disposed: bool,
}
impl ModelMeshCollectionEnumerator {
    fn new(lifetime: Arc<ModelLifetime>, items: Vec<Arc<ModelMesh>>) -> Self {
        Self {
            lifetime,
            items,
            position: None,
            disposed: false,
        }
    }
    pub fn MoveNext(&mut self) -> Result<bool> {
        enumerator_move_next(
            &self.lifetime,
            self.items.len(),
            &mut self.position,
            self.disposed,
        )
    }
    pub fn Current(&self) -> Result<&ModelMesh> {
        enumerator_current(&self.lifetime, &self.items, self.position, self.disposed)
    }
    pub fn Dispose(&mut self) -> Result<()> {
        self.disposed = true;
        self.position = None;
        Ok(())
    }
}
impl Drop for ModelMeshCollectionEnumerator {
    fn drop(&mut self) {}
}
impl PartialEq for ModelMeshCollectionEnumerator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifetime, &other.lifetime)
            && self.position == other.position
            && self.disposed == other.disposed
    }
}
impl Iterator for ModelMeshCollectionEnumerator {
    type Item = Arc<ModelMesh>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.disposed || self.lifetime.ensure().is_err() {
            return None;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        self.items.get(next).map(Arc::clone)
    }
}

pub struct ModelMeshPart {
    lifetime: Arc<ModelLifetime>,
    effects: Weak<Mutex<Vec<ModelEffectReference>>>,
    siblings: OnceLock<Weak<ModelMeshPartCollection>>,
    effect: Mutex<Option<ModelEffectReference>>,
    vertex_buffer: Arc<VertexBuffer>,
    index_buffer: Arc<IndexBuffer>,
    tag: Mutex<Option<TagValue>>,
    vertex_offset: i32,
    num_vertices: i32,
    start_index: i32,
    primitive_count: i32,
}
impl ModelMeshPart {
    pub fn Effect(&self) -> Result<Option<Arc<dyn EffectBase>>> {
        self.lifetime.ensure()?;
        Ok(self
            .effect
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetEffect(&mut self, value: Option<Arc<dyn EffectBase>>) -> Result<()> {
        self.set_effect_reference(value)
    }
    fn set_effect_reference(&self, value: Option<ModelEffectReference>) -> Result<()> {
        self.lifetime.ensure()?;
        if value.as_ref().is_some_and(|effect| {
            self.vertex_buffer
                .GraphicsDevice()
                .map_or(true, |device| !model_effect(effect).is_same_device(device))
        }) {
            return Err(CnaError::InvalidInput(
                "effect belongs to a different graphics device",
            ));
        }
        let mut current = self
            .effect
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = current.as_ref().map(model_effect_identity).transpose()?;
        let next = value.as_ref().map(model_effect_identity).transpose()?;
        if previous == next {
            return Ok(());
        }
        *current = value;
        drop(current);
        if let (Some(effects), Some(siblings)) = (
            self.effects.upgrade(),
            self.siblings.get().and_then(Weak::upgrade),
        ) {
            let mut rebuilt = Vec::new();
            for part in &siblings.items {
                if let Some(effect) = part
                    .effect
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
                {
                    let identity = model_effect_identity(&effect)?;
                    if !rebuilt
                        .iter()
                        .any(|item| model_effect_identity(item).ok() == Some(identity))
                    {
                        rebuilt.push(effect);
                    }
                }
            }
            *effects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = rebuilt;
        }
        Ok(())
    }
    pub fn IndexBuffer(&self) -> Result<&IndexBuffer> {
        self.lifetime.ensure()?;
        Ok(&self.index_buffer)
    }
    pub fn VertexBuffer(&self) -> Result<&VertexBuffer> {
        self.lifetime.ensure()?;
        Ok(&self.vertex_buffer)
    }
    pub fn NumVertices(&self) -> Result<i32> {
        self.lifetime.ensure()?;
        Ok(self.num_vertices)
    }
    pub fn PrimitiveCount(&self) -> Result<i32> {
        self.lifetime.ensure()?;
        Ok(self.primitive_count)
    }
    pub fn StartIndex(&self) -> Result<i32> {
        self.lifetime.ensure()?;
        Ok(self.start_index)
    }
    pub fn VertexOffset(&self) -> Result<i32> {
        self.lifetime.ensure()?;
        Ok(self.vertex_offset)
    }
    pub fn Tag(&self) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.lifetime.ensure()?;
        Ok(self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) -> Result<()> {
        self.lifetime.ensure()?;
        *self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
}

pub struct ModelMeshPartCollection {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelMeshPart>>,
}
impl ModelMeshPartCollection {
    pub fn GetEnumerator(&self) -> Result<ModelMeshPartCollectionEnumerator> {
        self.lifetime.ensure()?;
        Ok(ModelMeshPartCollectionEnumerator::new(
            Arc::clone(&self.lifetime),
            self.items.clone(),
        ))
    }
    pub(crate) fn count(&self) -> usize {
        self.items.len()
    }
    pub(crate) fn item_at(&self, index: usize) -> Result<Arc<ModelMeshPart>> {
        self.lifetime.ensure()?;
        self.items
            .get(index)
            .map(Arc::clone)
            .ok_or(CnaError::InvalidInput(
                "model mesh-part index is out of range",
            ))
    }
}

#[derive(Clone)]
pub struct ModelMeshPartCollectionEnumerator {
    lifetime: Arc<ModelLifetime>,
    items: Vec<Arc<ModelMeshPart>>,
    position: Option<usize>,
    disposed: bool,
}
impl ModelMeshPartCollectionEnumerator {
    fn new(lifetime: Arc<ModelLifetime>, items: Vec<Arc<ModelMeshPart>>) -> Self {
        Self {
            lifetime,
            items,
            position: None,
            disposed: false,
        }
    }
    pub fn MoveNext(&mut self) -> Result<bool> {
        enumerator_move_next(
            &self.lifetime,
            self.items.len(),
            &mut self.position,
            self.disposed,
        )
    }
    pub fn Current(&self) -> Result<&ModelMeshPart> {
        enumerator_current(&self.lifetime, &self.items, self.position, self.disposed)
    }
    pub fn Dispose(&mut self) -> Result<()> {
        self.disposed = true;
        self.position = None;
        Ok(())
    }
}
impl Drop for ModelMeshPartCollectionEnumerator {
    fn drop(&mut self) {}
}
impl PartialEq for ModelMeshPartCollectionEnumerator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifetime, &other.lifetime)
            && self.position == other.position
            && self.disposed == other.disposed
    }
}
impl Iterator for ModelMeshPartCollectionEnumerator {
    type Item = Arc<ModelMeshPart>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.disposed || self.lifetime.ensure().is_err() {
            return None;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        self.items.get(next).map(Arc::clone)
    }
}

pub struct ModelEffectCollection {
    lifetime: Arc<ModelLifetime>,
    items: Arc<Mutex<Vec<ModelEffectReference>>>,
}
impl ModelEffectCollection {
    fn snapshot(&self) -> Result<Vec<ModelEffectReference>> {
        self.lifetime.ensure()?;
        Ok(self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn GetEnumerator(&self) -> Result<ModelEffectCollectionEnumerator> {
        Ok(ModelEffectCollectionEnumerator {
            lifetime: Arc::clone(&self.lifetime),
            items: self.snapshot()?,
            position: None,
            disposed: false,
        })
    }
    pub(crate) fn count(&self) -> Result<usize> {
        Ok(self.snapshot()?.len())
    }
    pub(crate) fn item_at(&self, index: usize) -> Result<ModelEffectReference> {
        self.snapshot()?
            .get(index)
            .cloned()
            .ok_or(CnaError::InvalidInput("model effect index is out of range"))
    }
}

#[derive(Clone)]
pub struct ModelEffectCollectionEnumerator {
    lifetime: Arc<ModelLifetime>,
    items: Vec<ModelEffectReference>,
    position: Option<usize>,
    disposed: bool,
}
impl ModelEffectCollectionEnumerator {
    pub fn MoveNext(&mut self) -> Result<bool> {
        enumerator_move_next(
            &self.lifetime,
            self.items.len(),
            &mut self.position,
            self.disposed,
        )
    }
    pub fn Current(&self) -> Result<Arc<dyn EffectBase>> {
        self.lifetime.ensure()?;
        if self.disposed {
            return Err(CnaError::InvalidInput("model enumerator is disposed"));
        }
        let index = self
            .position
            .ok_or(CnaError::InvalidInput("model enumerator is not positioned"))?;
        self.items
            .get(index)
            .map(Arc::clone)
            .ok_or(CnaError::InvalidInput("model enumerator is past the end"))
    }
    pub fn Dispose(&mut self) -> Result<()> {
        self.disposed = true;
        self.position = None;
        Ok(())
    }
}
impl Drop for ModelEffectCollectionEnumerator {
    fn drop(&mut self) {}
}
impl PartialEq for ModelEffectCollectionEnumerator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.lifetime, &other.lifetime)
            && self.position == other.position
            && self.disposed == other.disposed
    }
}
impl Iterator for ModelEffectCollectionEnumerator {
    type Item = Arc<dyn EffectBase>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.disposed || self.lifetime.ensure().is_err() {
            return None;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        self.items.get(next).map(Arc::clone)
    }
}

fn add_unique_effect(
    items: &Arc<Mutex<Vec<ModelEffectReference>>>,
    value: ModelEffectReference,
) -> Result<()> {
    let identity = model_effect_identity(&value)?;
    let mut items = items
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !items
        .iter()
        .any(|item| model_effect_identity(item).ok() == Some(identity))
    {
        items.push(value);
    }
    Ok(())
}

fn validate_model_name(value: &str) -> Result<()> {
    if value.is_empty() {
        Err(CnaError::InvalidInput("model name must not be empty"))
    } else {
        Ok(())
    }
}

fn validate_bone_graph(bone_specs: &[ModelBoneSpec], root_index: usize) -> Result<()> {
    if bone_specs.is_empty() {
        return Err(CnaError::InvalidInput(
            "a model must contain at least one bone",
        ));
    }
    if root_index >= bone_specs.len() {
        return Err(CnaError::InvalidInput(
            "model root bone index is out of range",
        ));
    }
    for (index, spec) in bone_specs.iter().enumerate() {
        if spec.parent.is_some_and(|parent| parent >= index) {
            return Err(CnaError::InvalidInput(
                "model bone parents must precede their children",
            ));
        }
    }
    Ok(())
}

fn enumerator_move_next(
    lifetime: &ModelLifetime,
    length: usize,
    position: &mut Option<usize>,
    disposed: bool,
) -> Result<bool> {
    lifetime.ensure()?;
    if disposed {
        return Err(CnaError::InvalidInput("model enumerator is disposed"));
    }
    let next = position.map_or(0, |value| value.saturating_add(1));
    if next < length {
        *position = Some(next);
        Ok(true)
    } else {
        *position = Some(length);
        Ok(false)
    }
}

fn enumerator_current<'a, T>(
    lifetime: &ModelLifetime,
    items: &'a [Arc<T>],
    position: Option<usize>,
    disposed: bool,
) -> Result<&'a T> {
    lifetime.ensure()?;
    if disposed {
        return Err(CnaError::InvalidInput("model enumerator is disposed"));
    }
    let index = position.ok_or(CnaError::InvalidInput("model enumerator is not positioned"))?;
    items
        .get(index)
        .map(AsRef::as_ref)
        .ok_or(CnaError::InvalidInput("model enumerator is past the end"))
}
