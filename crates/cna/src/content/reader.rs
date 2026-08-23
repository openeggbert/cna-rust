#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::type_complexity
)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::error::{CnaError, Result};
use crate::graphics::{Effect, GraphicsResource, SpriteFont, SurfaceFormat, Texture2D};
use crate::value::{Color, Matrix, Quaternion, Rectangle, Vector2, Vector3, Vector4};

use super::manager::{
    content_error, content_error_with_inner, ContentDisposable, ContentDisposableRecorder,
};
use super::{ContentLoadable, ContentManager};

type ArcAny = Arc<dyn Any + Send + Sync>;
type ReaderCallback =
    dyn Fn(&ContentReader, Option<ArcAny>) -> Result<ArcAny> + Send + Sync + 'static;
type InitializeCallback = dyn Fn(&ContentTypeReaderManager) -> Result<()> + Send + Sync + 'static;
type SharedFixup = Box<dyn FnOnce(ArcAny) -> Result<()> + Send>;

struct ReaderDescriptor {
    target_type: TypeId,
    type_version: i32,
    can_deserialize_into_existing: bool,
    value_type: bool,
    initialize: Arc<InitializeCallback>,
    read: Arc<ReaderCallback>,
    disposable: fn(&ArcAny) -> Option<Arc<dyn ContentDisposable>>,
}

/// Runtime descriptor for one XNB type reader.
#[derive(Clone)]
pub struct ContentTypeReader {
    descriptor: Arc<ReaderDescriptor>,
}

#[allow(non_snake_case)]
impl ContentTypeReader {
    #[must_use]
    pub fn new(targetType: TypeId) -> Self {
        registered_reader_for_target(targetType).unwrap_or_else(|| Self {
            descriptor: Arc::new(ReaderDescriptor {
                target_type: targetType,
                type_version: 0,
                can_deserialize_into_existing: false,
                value_type: is_builtin_value_type(targetType),
                initialize: Arc::new(|_| Ok(())),
                read: Arc::new(|_, _| {
                    Err(CnaError::InvalidInput(
                        "abstract ContentTypeReader has no registered typed reader",
                    ))
                }),
                disposable: |_| None,
            }),
        })
    }

    #[must_use]
    pub fn TargetType(&self) -> TypeId {
        self.descriptor.target_type
    }

    #[must_use]
    pub fn TypeVersion(&self) -> i32 {
        self.descriptor.type_version
    }

    #[must_use]
    pub fn CanDeserializeIntoExistingObject(&self) -> bool {
        self.descriptor.can_deserialize_into_existing
    }

    pub fn Initialize(&self, manager: &ContentTypeReaderManager) -> Result<()> {
        (self.descriptor.initialize)(manager)
    }

    pub fn Read(
        &self,
        input: &ContentReader,
        existingInstance: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Arc<dyn Any + Send + Sync>> {
        (self.descriptor.read)(input, existingInstance)
    }

    fn is_value_type(&self) -> bool {
        self.descriptor.value_type
    }

    fn disposable(&self, value: &ArcAny) -> Option<Arc<dyn ContentDisposable>> {
        (self.descriptor.disposable)(value)
    }
}

/// Rust base contract for the generic XNA reader relationship.
pub trait ContentTypeReaderBase {}

/// Strongly typed view of a content reader descriptor.
pub struct ContentTypeReaderOfT<T: ContentLoadable> {
    reader: ContentTypeReader,
    marker: PhantomData<fn() -> T>,
}

#[allow(non_snake_case)]
impl<T: ContentLoadable> ContentTypeReaderOfT<T> {
    #[must_use]
    pub fn new() -> Self {
        let target = TypeId::of::<T>();
        let reader = registered_reader_for_target(target)
            .or_else(|| builtin_reader_for_target(target))
            .unwrap_or_else(|| ContentTypeReader::new(target));
        Self {
            reader,
            marker: PhantomData,
        }
    }

    pub fn Read(
        &self,
        input: &ContentReader,
        existingInstance: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Arc<dyn Any + Send + Sync>> {
        self.reader.Read(input, existingInstance)
    }

    pub fn ReadWithInputAndExistingInstance(
        &self,
        input: &ContentReader,
        existingInstance: Option<Arc<T>>,
    ) -> Result<Arc<T>> {
        let existing = existingInstance.map(|value| value as ArcAny);
        self.reader
            .Read(input, existing)?
            .downcast::<T>()
            .map_err(|_| content_error("typed content reader returned the wrong Rust type"))
    }
}

impl<T: ContentLoadable> Default for ContentTypeReaderOfT<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ContentLoadable> ContentTypeReaderBase for ContentTypeReaderOfT<T> {}

struct ReaderManagerState {
    readers_by_target: HashMap<TypeId, Arc<ContentTypeReader>>,
}

/// Per-asset reader table used for dependency lookup during initialization.
#[derive(Clone)]
pub struct ContentTypeReaderManager {
    state: Arc<ReaderManagerState>,
}

#[allow(non_snake_case)]
impl ContentTypeReaderManager {
    #[must_use]
    pub fn GetTypeReader(&self, targetType: TypeId) -> Option<Arc<ContentTypeReader>> {
        self.state.readers_by_target.get(&targetType).cloned()
    }

    fn from_readers(readers: &[Arc<ContentTypeReader>]) -> Self {
        let mut readers_by_target = HashMap::new();
        for reader in readers {
            readers_by_target
                .entry(reader.TargetType())
                .or_insert_with(|| Arc::clone(reader));
        }
        Self {
            state: Arc::new(ReaderManagerState { readers_by_target }),
        }
    }
}

struct BinaryCursor {
    bytes: Vec<u8>,
    position: usize,
}

impl BinaryCursor {
    fn read_exact(&mut self, count: usize) -> Option<&[u8]> {
        let end = self.position.checked_add(count)?;
        if end > self.bytes.len() {
            return None;
        }
        let start = self.position;
        self.position = end;
        Some(&self.bytes[start..end])
    }
}

/// Managed XNA XNB object-graph reader.
pub struct ContentReader {
    content_manager: ContentManager,
    asset_name: String,
    cursor: Mutex<BinaryCursor>,
    type_readers: Mutex<Vec<Arc<ContentTypeReader>>>,
    type_reader_versions: Mutex<Vec<i32>>,
    shared_resource_fixups: Mutex<Option<Vec<Vec<SharedFixup>>>>,
    record_disposable_object: Option<Arc<dyn ContentDisposableRecorder>>,
    disposed: std::sync::atomic::AtomicBool,
}

#[allow(non_snake_case)]
impl ContentReader {
    pub(crate) fn create(
        content_manager: ContentManager,
        bytes: Vec<u8>,
        asset_name: &str,
        record_disposable_object: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Self> {
        if bytes.len() < 10 {
            return Err(xnb_error(asset_name, "truncated XNB header"));
        }
        if &bytes[..3] != b"XNB" {
            return Err(xnb_error(asset_name, "invalid XNB magic"));
        }
        if bytes[3] != b'w' {
            return Err(xnb_error(
                asset_name,
                &format!("unsupported XNB platform '{}'", char::from(bytes[3])),
            ));
        }
        if bytes[4] != 5 {
            return Err(xnb_error(
                asset_name,
                &format!("unsupported XNB version {}", bytes[4]),
            ));
        }
        let flags = bytes[5];
        if flags & 0x80 != 0 {
            return Err(xnb_error(
                asset_name,
                "LZX-compressed XNB is not yet supported",
            ));
        }
        if flags & 0x40 != 0 {
            return Err(xnb_error(
                asset_name,
                "LZ4 compression is not part of the selected XNA 4.0 format",
            ));
        }
        if flags & 0x3f != 0 {
            return Err(xnb_error(asset_name, "XNB header contains unknown flags"));
        }
        let declared_size = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        if usize::try_from(declared_size).ok() != Some(bytes.len()) {
            return Err(xnb_error(
                asset_name,
                &format!(
                    "declared XNB size {declared_size} does not match stream size {}",
                    bytes.len()
                ),
            ));
        }
        Ok(Self {
            content_manager,
            asset_name: asset_name.to_owned(),
            cursor: Mutex::new(BinaryCursor {
                bytes,
                position: 10,
            }),
            type_readers: Mutex::new(Vec::new()),
            type_reader_versions: Mutex::new(Vec::new()),
            shared_resource_fixups: Mutex::new(None),
            record_disposable_object,
            disposed: std::sync::atomic::AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn ContentManager(&self) -> &ContentManager {
        &self.content_manager
    }

    #[must_use]
    pub fn AssetName(&self) -> String {
        self.asset_name.clone()
    }

    pub fn ReadObject<T: ContentLoadable>(&self) -> Result<Option<Arc<T>>> {
        self.read_object_erased(None)?
            .map(|value| {
                value.downcast::<T>().map_err(|_| {
                    content_error(format!(
                        "content reader result is not '{}'",
                        std::any::type_name::<T>()
                    ))
                })
            })
            .transpose()
    }

    pub fn ReadObjectWithExistingInstance<T: ContentLoadable>(
        &self,
        existingInstance: Option<Arc<T>>,
    ) -> Result<Arc<T>> {
        let existing = existingInstance.map(|value| value as ArcAny);
        self.read_object_erased(existing)?
            .ok_or_else(|| content_error("XNB object tag was null for an existing instance"))?
            .downcast::<T>()
            .map_err(|_| content_error("content reader result has the wrong Rust type"))
    }

    pub fn ReadObjectWithTypeReader<T: ContentLoadable>(
        &self,
        typeReader: &ContentTypeReader,
    ) -> Result<Option<Arc<T>>> {
        let value = if typeReader.is_value_type() {
            Some(self.read_and_record(typeReader, None)?)
        } else {
            self.read_object_erased(None)?
        };
        value
            .map(|value| {
                value
                    .downcast::<T>()
                    .map_err(|_| content_error("content reader result has the wrong Rust type"))
            })
            .transpose()
    }

    pub fn ReadObjectWithTypeReaderAndExistingInstance<T: ContentLoadable>(
        &self,
        typeReader: &ContentTypeReader,
        existingInstance: Option<Arc<T>>,
    ) -> Result<Arc<T>> {
        let existing = existingInstance.map(|value| value as ArcAny);
        let value = if typeReader.is_value_type() {
            self.read_and_record(typeReader, existing)?
        } else {
            self.read_object_erased(existing)?
                .ok_or_else(|| content_error("XNB object tag was null for an existing instance"))?
        };
        value
            .downcast::<T>()
            .map_err(|_| content_error("content reader result has the wrong Rust type"))
    }

    pub fn ReadRawObject<T: ContentLoadable>(&self) -> Result<Arc<T>> {
        let reader = self.find_type_reader(TypeId::of::<T>())?;
        self.read_and_record(&reader, None)?
            .downcast::<T>()
            .map_err(|_| content_error("raw content reader result has the wrong Rust type"))
    }

    pub fn ReadRawObjectWithExistingInstance<T: ContentLoadable>(
        &self,
        existingInstance: Option<Arc<T>>,
    ) -> Result<Arc<T>> {
        let reader = self.find_type_reader(TypeId::of::<T>())?;
        let existing = existingInstance.map(|value| value as ArcAny);
        self.read_and_record(&reader, existing)?
            .downcast::<T>()
            .map_err(|_| content_error("raw content reader result has the wrong Rust type"))
    }

    pub fn ReadRawObjectWithTypeReader<T: ContentLoadable>(
        &self,
        typeReader: &ContentTypeReader,
    ) -> Result<Arc<T>> {
        self.read_and_record(typeReader, None)?
            .downcast::<T>()
            .map_err(|_| content_error("raw content reader result has the wrong Rust type"))
    }

    pub fn ReadRawObjectWithTypeReaderAndExistingInstance<T: ContentLoadable>(
        &self,
        typeReader: &ContentTypeReader,
        existingInstance: Option<Arc<T>>,
    ) -> Result<Arc<T>> {
        let existing = existingInstance.map(|value| value as ArcAny);
        self.read_and_record(typeReader, existing)?
            .downcast::<T>()
            .map_err(|_| content_error("raw content reader result has the wrong Rust type"))
    }

    pub fn ReadSharedResource<T: ContentLoadable>(
        &self,
        fixup: Box<dyn FnOnce(Arc<T>) + Send>,
    ) -> Result<()> {
        let index = self.read_7bit_encoded_i32()?;
        if index == 0 {
            return Ok(());
        }
        let zero_based = usize::try_from(index - 1)
            .map_err(|_| xnb_error(&self.asset_name, "shared resource index is negative"))?;
        let mut fixups = self
            .shared_resource_fixups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slots = fixups.as_mut().ok_or_else(|| {
            xnb_error(&self.asset_name, "shared resource table is not initialized")
        })?;
        let slot = slots.get_mut(zero_based).ok_or_else(|| {
            xnb_error(
                &self.asset_name,
                &format!("invalid shared resource index {index}"),
            )
        })?;
        slot.push(Box::new(move |value| {
            let typed = value
                .downcast::<T>()
                .map_err(|_| content_error("shared resource has the wrong Rust type"))?;
            fixup(typed);
            Ok(())
        }));
        Ok(())
    }

    pub fn ReadExternalReference<T: ContentLoadable>(&self) -> Result<Option<Arc<T>>> {
        let reference = self.read_string()?;
        if reference.is_empty() {
            return Ok(None);
        }
        let parent = Path::new(&self.asset_name)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let resolved = normalize_external_reference(&parent.join(reference))?;
        self.content_manager.Load::<T>(&resolved).map(Some)
    }

    pub fn ReadVector2(&self) -> Result<Vector2> {
        Ok(Vector2::from_x_and_y(self.read_f32()?, self.read_f32()?))
    }

    pub fn ReadVector3(&self) -> Result<Vector3> {
        Ok(Vector3::from_x_and_y_and_z(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn ReadVector4(&self) -> Result<Vector4> {
        Ok(Vector4::from_x_and_y_and_z_and_w(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn ReadMatrix(&self) -> Result<Matrix> {
        Ok(Matrix::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn ReadQuaternion(&self) -> Result<Quaternion> {
        Ok(Quaternion::from_x_and_y_and_z_and_w(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn ReadColor(&self) -> Result<Color> {
        Ok(
            Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                i32::from(self.read_u8()?),
                i32::from(self.read_u8()?),
                i32::from(self.read_u8()?),
                i32::from(self.read_u8()?),
            ),
        )
    }

    pub fn ReadSingle(&self) -> Result<f32> {
        self.read_f32()
    }

    pub fn ReadDouble(&self) -> Result<f64> {
        self.read_f64()
    }

    pub(crate) fn read_asset<T: ContentLoadable>(&self) -> Result<Arc<T>> {
        let readers = self.load_asset_readers()?;
        let manager = ContentTypeReaderManager::from_readers(&readers);
        for reader in &readers {
            reader.Initialize(&manager).map_err(|error| {
                content_error(format!(
                    "failed to initialize content type reader for '{}': {error}",
                    self.asset_name
                ))
            })?;
        }
        let shared_count = self.read_7bit_encoded_i32()?;
        if !(0..=1_000_000).contains(&shared_count) {
            return Err(xnb_error(
                &self.asset_name,
                &format!("invalid shared resource count {shared_count}"),
            ));
        }
        let shared_count = usize::try_from(shared_count)
            .map_err(|_| xnb_error(&self.asset_name, "shared resource count is negative"))?;
        *self
            .shared_resource_fixups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some((0..shared_count).map(|_| Vec::new()).collect());

        let root = self
            .read_object_erased(None)?
            .ok_or_else(|| xnb_error(&self.asset_name, "root object is null"))?;
        let mut shared_values = Vec::with_capacity(shared_count);
        for _ in 0..shared_count {
            shared_values.push(self.read_object_erased(None)?);
        }
        let fixups = self
            .shared_resource_fixups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .unwrap_or_default();
        for (position, (value, callbacks)) in shared_values.into_iter().zip(fixups).enumerate() {
            if callbacks.is_empty() {
                continue;
            }
            let value = value.ok_or_else(|| {
                xnb_error(
                    &self.asset_name,
                    &format!("shared resource {} is null", position + 1),
                )
            })?;
            for callback in callbacks {
                callback(Arc::clone(&value))?;
            }
        }
        root.downcast::<T>().map_err(|value| {
            content_error(format!(
                "content asset '{}' produced '{}', not '{}'",
                self.asset_name,
                type_name_for_any(&value),
                std::any::type_name::<T>()
            ))
        })
    }

    fn load_asset_readers(&self) -> Result<Vec<Arc<ContentTypeReader>>> {
        let count = self.read_7bit_encoded_i32()?;
        if !(0..=4096).contains(&count) {
            return Err(xnb_error(
                &self.asset_name,
                &format!("invalid type reader count {count}"),
            ));
        }
        let count = usize::try_from(count)
            .map_err(|_| xnb_error(&self.asset_name, "type reader count is negative"))?;
        let mut readers = Vec::with_capacity(count);
        let mut versions = Vec::with_capacity(count);
        for _ in 0..count {
            let serialized_name = self.read_string()?;
            if serialized_name.trim().is_empty() {
                return Err(xnb_error(&self.asset_name, "content reader name is empty"));
            }
            let reader = activate_reader(&serialized_name).ok_or_else(|| {
                content_error(format!(
                    "unknown content type reader '{serialized_name}' while loading '{}'",
                    self.asset_name
                ))
            })?;
            let version = self.read_i32()?;
            if version != reader.TypeVersion() {
                return Err(xnb_error(
                    &self.asset_name,
                    &format!(
                        "reader version mismatch for '{serialized_name}': asset {version}, runtime {}",
                        reader.TypeVersion()
                    ),
                ));
            }
            readers.push(Arc::new(reader));
            versions.push(version);
        }
        *self
            .type_readers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = readers.clone();
        *self
            .type_reader_versions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = versions;
        Ok(readers)
    }

    fn read_object_erased(&self, existing: Option<ArcAny>) -> Result<Option<ArcAny>> {
        let index = self.read_7bit_encoded_i32()?;
        if index == 0 {
            return Ok(None);
        }
        let position = usize::try_from(index - 1)
            .map_err(|_| xnb_error(&self.asset_name, "type reader index is negative"))?;
        let reader = self
            .type_readers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(position)
            .cloned()
            .ok_or_else(|| {
                xnb_error(
                    &self.asset_name,
                    &format!("invalid type reader index {index}"),
                )
            })?;
        self.read_and_record(&reader, existing).map(Some)
    }

    fn read_and_record(
        &self,
        reader: &ContentTypeReader,
        existing: Option<ArcAny>,
    ) -> Result<ArcAny> {
        let retained_existing = existing.as_ref().map(Arc::clone);
        let value = reader.Read(self, existing).map_err(|error| {
            content_error(format!(
                "content type reader failed while loading '{}': {error}",
                self.asset_name
            ))
        })?;
        if value.as_ref().type_id() != reader.TargetType() {
            return Err(content_error(format!(
                "content reader declared target {:?} but returned '{}'",
                reader.TargetType(),
                type_name_for_any(&value)
            )));
        }
        if let Some(existing) = retained_existing {
            if !Arc::ptr_eq(&existing, &value) {
                return Err(CnaError::InvalidInput(
                    "content reader replaced an existing instance",
                ));
            }
        } else if let Some(disposable) = reader.disposable(&value) {
            self.record_disposable(disposable)?;
        }
        Ok(value)
    }

    fn find_type_reader(&self, target_type: TypeId) -> Result<Arc<ContentTypeReader>> {
        self.type_readers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|reader| reader.TargetType() == target_type)
            .cloned()
            .ok_or_else(|| {
                content_error(format!(
                    "XNB reader table has no reader for '{target_type:?}'"
                ))
            })
    }

    fn record_disposable(&self, value: Arc<dyn ContentDisposable>) -> Result<()> {
        if let Some(recorder) = &self.record_disposable_object {
            recorder.Record(value)
        } else {
            self.content_manager.record_disposable(value)
        }
    }

    fn reader_version(&self, reader: &ContentTypeReader) -> Result<i32> {
        let readers = self
            .type_readers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let versions = self
            .type_reader_versions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        readers
            .iter()
            .position(|value| Arc::ptr_eq(&value.descriptor, &reader.descriptor))
            .and_then(|position| versions.get(position).copied())
            .ok_or(CnaError::InvalidInput(
                "content type reader is not in this asset's reader table",
            ))
    }

    fn read_bytes(&self, count: usize) -> Result<Vec<u8>> {
        if self.disposed.load(Ordering::Acquire) {
            return Err(CnaError::InvalidInput("content reader is disposed"));
        }
        self.cursor
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .read_exact(count)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| xnb_error(&self.asset_name, "unexpected end of content stream"))
    }

    fn read_u8(&self) -> Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_i8(&self) -> Result<i8> {
        Ok(i8::from_le_bytes([self.read_u8()?]))
    }

    fn read_u16(&self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_i16(&self) -> Result<i16> {
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i32(&self) -> Result<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_i64(&self) -> Result<i64> {
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_f32(&self) -> Result<f32> {
        self.read_u32().map(f32::from_bits)
    }

    fn read_f64(&self) -> Result<f64> {
        self.read_u64().map(f64::from_bits)
    }

    fn read_7bit_encoded_i32(&self) -> Result<i32> {
        let mut result = 0_u32;
        for shift in (0..35).step_by(7) {
            let value = self.read_u8()?;
            if shift == 28 && value & 0xf0 != 0 {
                return Err(xnb_error(&self.asset_name, "invalid 7-bit encoded integer"));
            }
            result |= u32::from(value & 0x7f) << shift;
            if value & 0x80 == 0 {
                return i32::try_from(result)
                    .map_err(|_| xnb_error(&self.asset_name, "negative 7-bit encoded integer"));
            }
        }
        Err(xnb_error(&self.asset_name, "invalid 7-bit encoded integer"))
    }

    fn read_string(&self) -> Result<String> {
        let count = usize::try_from(self.read_7bit_encoded_i32()?)
            .map_err(|_| xnb_error(&self.asset_name, "string length is negative"))?;
        let bytes = self.read_bytes(count)?;
        String::from_utf8(bytes)
            .map_err(|_| xnb_error(&self.asset_name, "content string is not valid UTF-8"))
    }
}

/// Rust projection of the disposable `BinaryReader` base contract inherited
/// by XNA's content reader.
pub trait ContentReaderBase {
    fn Dispose(&self) -> Result<()>;
}

impl ContentReaderBase for ContentReader {
    fn Dispose(&self) -> Result<()> {
        self.disposed.store(true, Ordering::Release);
        Ok(())
    }
}

impl Drop for ContentReader {
    fn drop(&mut self) {
        self.disposed.store(true, Ordering::Release);
    }
}

/// `BinaryReader` behavior and managed-reader helpers inherited by XNA's
/// `ContentReader`, exposed as a Rust extension trait.
pub trait ContentReaderExt {
    fn ReadBoolean(&self) -> Result<bool>;
    fn ReadByte(&self) -> Result<u8>;
    fn ReadSByte(&self) -> Result<i8>;
    fn ReadInt16(&self) -> Result<i16>;
    fn ReadUInt16(&self) -> Result<u16>;
    fn ReadInt32(&self) -> Result<i32>;
    fn ReadUInt32(&self) -> Result<u32>;
    fn ReadInt64(&self) -> Result<i64>;
    fn ReadUInt64(&self) -> Result<u64>;
    fn ReadChar(&self) -> Result<char>;
    fn ReadString(&self) -> Result<String>;
    fn RecordDisposable<T: ContentDisposable + 'static>(&self, value: Arc<T>) -> Result<()>;
    fn TypeReaderVersion(&self, reader: &ContentTypeReader) -> Result<i32>;
}

#[allow(non_snake_case)]
impl ContentReaderExt for ContentReader {
    fn ReadBoolean(&self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    fn ReadByte(&self) -> Result<u8> {
        self.read_u8()
    }

    fn ReadSByte(&self) -> Result<i8> {
        self.read_i8()
    }

    fn ReadInt16(&self) -> Result<i16> {
        self.read_i16()
    }

    fn ReadUInt16(&self) -> Result<u16> {
        self.read_u16()
    }

    fn ReadInt32(&self) -> Result<i32> {
        self.read_i32()
    }

    fn ReadUInt32(&self) -> Result<u32> {
        self.read_u32()
    }

    fn ReadInt64(&self) -> Result<i64> {
        self.read_i64()
    }

    fn ReadUInt64(&self) -> Result<u64> {
        self.read_u64()
    }

    fn ReadChar(&self) -> Result<char> {
        char::from_u32(u32::from(self.read_u16()?))
            .ok_or_else(|| xnb_error(&self.asset_name, "content character is invalid"))
    }

    fn ReadString(&self) -> Result<String> {
        self.read_string()
    }

    fn RecordDisposable<T: ContentDisposable + 'static>(&self, value: Arc<T>) -> Result<()> {
        self.record_disposable(value)
    }

    fn TypeReaderVersion(&self, reader: &ContentTypeReader) -> Result<i32> {
        self.reader_version(reader)
    }
}

#[derive(Clone)]
struct RegistryEntry {
    token: u64,
    reader: ContentTypeReader,
}

fn registry() -> &'static Mutex<HashMap<String, RegistryEntry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, RegistryEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

static NEXT_REGISTRATION: AtomicU64 = AtomicU64::new(0);

/// RAII registration for a custom XNB content reader.
pub struct ContentTypeReaderRegistration {
    name: String,
    token: u64,
}

impl Drop for ContentTypeReaderRegistration {
    fn drop(&mut self) {
        let mut entries = registry()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if entries
            .get(&self.name)
            .is_some_and(|entry| entry.token == self.token)
        {
            entries.remove(&self.name);
        }
    }
}

/// Explicit Rust activator for user-defined XNB reader names.
pub struct ContentTypeReaderRegistry;

#[allow(non_snake_case)]
impl ContentTypeReaderRegistry {
    pub fn Register<T, F>(
        serializedReaderName: &str,
        typeVersion: i32,
        canDeserializeIntoExistingObject: bool,
        read: F,
    ) -> Result<ContentTypeReaderRegistration>
    where
        T: ContentLoadable,
        F: Fn(&ContentReader, Option<Arc<T>>) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        Self::RegisterWithInitialize(
            serializedReaderName,
            typeVersion,
            canDeserializeIntoExistingObject,
            |_| Ok(()),
            read,
        )
    }

    pub fn RegisterWithInitialize<T, I, F>(
        serializedReaderName: &str,
        typeVersion: i32,
        canDeserializeIntoExistingObject: bool,
        initialize: I,
        read: F,
    ) -> Result<ContentTypeReaderRegistration>
    where
        T: ContentLoadable,
        I: Fn(&ContentTypeReaderManager) -> Result<()> + Send + Sync + 'static,
        F: Fn(&ContentReader, Option<Arc<T>>) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        let name = serializedReaderName.trim();
        if name.is_empty() {
            return Err(CnaError::InvalidInput(
                "serialized content reader name must not be empty",
            ));
        }
        let read = Arc::new(read);
        let erased_read = Arc::new(
            move |input: &ContentReader, existing: Option<ArcAny>| -> Result<ArcAny> {
                let typed_existing = existing
                    .map(|value| {
                        value.downcast::<T>().map_err(|_| {
                            content_error("existing content instance has the wrong Rust type")
                        })
                    })
                    .transpose()?;
                let value = read(input, typed_existing)?;
                Ok(value as ArcAny)
            },
        );
        let reader = ContentTypeReader {
            descriptor: Arc::new(ReaderDescriptor {
                target_type: TypeId::of::<T>(),
                type_version: typeVersion,
                can_deserialize_into_existing: canDeserializeIntoExistingObject,
                value_type: is_builtin_value_type(TypeId::of::<T>()),
                initialize: Arc::new(initialize),
                read: erased_read,
                disposable: disposable_for::<T>,
            }),
        };
        let token = NEXT_REGISTRATION.fetch_add(1, Ordering::Relaxed) + 1;
        let mut entries = registry()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if entries.contains_key(name) {
            return Err(CnaError::InvalidInput(
                "a content type reader is already registered with this name",
            ));
        }
        entries.insert(name.to_owned(), RegistryEntry { token, reader });
        Ok(ContentTypeReaderRegistration {
            name: name.to_owned(),
            token,
        })
    }
}

fn disposable_for<T: ContentLoadable>(value: &ArcAny) -> Option<Arc<dyn ContentDisposable>> {
    let typed = Arc::clone(value).downcast::<T>().ok()?;
    T::ContentDisposable(&typed)
}

fn activate_reader(serialized_name: &str) -> Option<ContentTypeReader> {
    let stripped = strip_assembly_qualification(serialized_name);
    if let Some(reader) = builtin_reader(&stripped) {
        return Some(reader);
    }
    let entries = registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    entries
        .get(serialized_name.trim())
        .or_else(|| entries.get(&stripped))
        .map(|entry| entry.reader.clone())
}

fn registered_reader_for_target(target: TypeId) -> Option<ContentTypeReader> {
    let entries = registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut names = entries.keys().collect::<Vec<_>>();
    names.sort();
    names.into_iter().find_map(|name| {
        let reader = &entries[name].reader;
        (reader.TargetType() == target).then(|| reader.clone())
    })
}

fn strip_assembly_qualification(value: &str) -> String {
    let mut depth = 0_i32;
    for (index, character) in value.char_indices() {
        match character {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => return value[..index].trim().to_owned(),
            _ => {}
        }
    }
    value.trim().to_owned()
}

fn typed_reader<T, F>(read: F) -> ContentTypeReader
where
    T: ContentLoadable,
    F: Fn(&ContentReader) -> Result<T> + Send + Sync + 'static,
{
    ContentTypeReader {
        descriptor: Arc::new(ReaderDescriptor {
            target_type: TypeId::of::<T>(),
            type_version: 0,
            can_deserialize_into_existing: false,
            value_type: true,
            initialize: Arc::new(|_| Ok(())),
            read: Arc::new(move |input, _| Ok(Arc::new(read(input)?) as ArcAny)),
            disposable: disposable_for::<T>,
        }),
    }
}

fn class_reader<T, F>(read: F) -> ContentTypeReader
where
    T: ContentLoadable,
    F: Fn(&ContentReader) -> Result<T> + Send + Sync + 'static,
{
    ContentTypeReader {
        descriptor: Arc::new(ReaderDescriptor {
            target_type: TypeId::of::<T>(),
            type_version: 0,
            can_deserialize_into_existing: false,
            value_type: false,
            initialize: Arc::new(|_| Ok(())),
            read: Arc::new(move |input, _| Ok(Arc::new(read(input)?) as ArcAny)),
            disposable: disposable_for::<T>,
        }),
    }
}

fn list_reader<T>() -> ContentTypeReader
where
    T: ContentLoadable + Clone,
{
    let element_reader = Arc::new(Mutex::new(None::<Arc<ContentTypeReader>>));
    let initialize_reader = Arc::clone(&element_reader);
    let read_reader = Arc::clone(&element_reader);
    ContentTypeReader {
        descriptor: Arc::new(ReaderDescriptor {
            target_type: TypeId::of::<Vec<T>>(),
            type_version: 0,
            can_deserialize_into_existing: false,
            value_type: false,
            initialize: Arc::new(move |manager| {
                let reader = manager.GetTypeReader(TypeId::of::<T>()).ok_or_else(|| {
                    content_error(format!(
                        "XNB list reader has no element reader for '{}'",
                        std::any::type_name::<T>()
                    ))
                })?;
                *initialize_reader
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reader);
                Ok(())
            }),
            read: Arc::new(move |input, _| {
                let count = input.read_i32()?;
                if !(0..=1_000_000).contains(&count) {
                    return Err(xnb_error(
                        &input.asset_name,
                        &format!("invalid list element count {count}"),
                    ));
                }
                let count = usize::try_from(count)
                    .map_err(|_| xnb_error(&input.asset_name, "list element count is negative"))?;
                let reader = read_reader
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
                    .ok_or_else(|| {
                        xnb_error(&input.asset_name, "list reader was not initialized")
                    })?;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    let value = input
                        .ReadObjectWithTypeReader::<T>(&reader)?
                        .ok_or_else(|| xnb_error(&input.asset_name, "list element is null"))?;
                    values.push((*value).clone());
                }
                Ok(Arc::new(values) as ArcAny)
            }),
            disposable: disposable_for::<Vec<T>>,
        }),
    }
}

fn texture2d_reader() -> ContentTypeReader {
    class_reader::<Texture2D, _>(|input| {
        let format_value = input.read_i32()?;
        let format = u32::try_from(format_value)
            .ok()
            .and_then(SurfaceFormat::from_native)
            .ok_or_else(|| {
                xnb_error(
                    &input.asset_name,
                    &format!("invalid Texture2D SurfaceFormat {format_value}"),
                )
            })?;
        let width = input.read_i32()?;
        let height = input.read_i32()?;
        let mip_count = input.read_i32()?;
        if width <= 0 || height <= 0 {
            return Err(xnb_error(
                &input.asset_name,
                "Texture2D dimensions must be positive",
            ));
        }
        let maximum_dimension = u32::try_from(width.max(height))
            .map_err(|_| xnb_error(&input.asset_name, "Texture2D dimensions are invalid"))?;
        let complete_mip_count = i32::try_from(u32::BITS - maximum_dimension.leading_zeros())
            .map_err(|_| xnb_error(&input.asset_name, "Texture2D mip count overflows"))?;
        if mip_count != 1 && mip_count != complete_mip_count {
            return Err(xnb_error(
                &input.asset_name,
                &format!("invalid Texture2D mip count {mip_count} for {width}x{height}"),
            ));
        }
        if format != SurfaceFormat::Color {
            return Err(xnb_error(
                &input.asset_name,
                "initial Texture2D XNB support requires SurfaceFormat.Color",
            ));
        }
        let texture = Texture2D::from_graphics_device_and_width_and_height_and_mip_map_and_format(
            &input.content_manager.graphics_device()?,
            width,
            height,
            mip_count > 1,
            format,
        )?;
        for level in 0..mip_count {
            let payload_length = input.read_i32()?;
            let shift = u32::try_from(level)
                .map_err(|_| xnb_error(&input.asset_name, "Texture2D mip level is negative"))?;
            let level_width = width.checked_shr(shift).unwrap_or(0).max(1);
            let level_height = height.checked_shr(shift).unwrap_or(0).max(1);
            let expected = i64::from(level_width)
                .checked_mul(i64::from(level_height))
                .and_then(|pixels| pixels.checked_mul(4))
                .ok_or_else(|| xnb_error(&input.asset_name, "Texture2D mip size overflows"))?;
            if i64::from(payload_length) != expected {
                return Err(xnb_error(
                    &input.asset_name,
                    &format!(
                        "Texture2D mip {level} payload length {payload_length} does not match {expected}"
                    ),
                ));
            }
            let payload = input.read_bytes(usize::try_from(payload_length).map_err(|_| {
                xnb_error(&input.asset_name, "Texture2D payload length is negative")
            })?)?;
            let pixels = payload
                .chunks_exact(4)
                .map(|rgba| {
                    Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                        i32::from(rgba[0]),
                        i32::from(rgba[1]),
                        i32::from(rgba[2]),
                        i32::from(rgba[3]),
                    )
                })
                .collect::<Vec<_>>();
            let pixel_count = i32::try_from(pixels.len())
                .map_err(|_| xnb_error(&input.asset_name, "Texture2D pixel count overflows"))?;
            texture.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
                level,
                None,
                &pixels,
                0,
                pixel_count,
            )?;
        }
        Ok(texture)
    })
}

fn sprite_font_reader() -> ContentTypeReader {
    class_reader::<SpriteFont, _>(|input| {
        let texture = input
            .ReadObject::<Texture2D>()?
            .ok_or_else(|| xnb_error(&input.asset_name, "SpriteFont atlas is null"))?;
        let glyphs = input
            .ReadObject::<Vec<Rectangle>>()?
            .ok_or_else(|| xnb_error(&input.asset_name, "SpriteFont glyph list is null"))?;
        let cropping = input
            .ReadObject::<Vec<Rectangle>>()?
            .ok_or_else(|| xnb_error(&input.asset_name, "SpriteFont cropping list is null"))?;
        let characters = input
            .ReadObject::<Vec<char>>()?
            .ok_or_else(|| xnb_error(&input.asset_name, "SpriteFont character list is null"))?;
        let line_spacing = input.read_i32()?;
        let spacing = input.read_f32()?;
        let kerning = input
            .ReadObject::<Vec<Vector3>>()?
            .ok_or_else(|| xnb_error(&input.asset_name, "SpriteFont kerning list is null"))?;
        let default_character = if input.read_u8()? == 0 {
            None
        } else {
            Some(ContentReaderExt::ReadChar(input)?)
        };
        SpriteFont::from_parts(
            texture,
            (*glyphs).clone(),
            (*cropping).clone(),
            (*characters).clone(),
            line_spacing,
            spacing,
            (*kerning).clone(),
            default_character,
        )
    })
}

fn effect_reader() -> ContentTypeReader {
    class_reader::<Effect, _>(|input| {
        let length = input.read_i32()?;
        if !(1..=64 * 1024 * 1024).contains(&length) {
            return Err(xnb_error(
                &input.asset_name,
                &format!("invalid Effect bytecode length {length}"),
            ));
        }
        let payload = input
            .read_bytes(usize::try_from(length).map_err(|_| {
                xnb_error(&input.asset_name, "Effect bytecode length is negative")
            })?)?;
        let mut effect = Effect::from_graphics_device_and_effect_code(
            &input.content_manager.graphics_device()?,
            &payload,
        )
        .map_err(|error| {
            content_error_with_inner(
                format!(
                    "could not construct Effect content asset '{}'",
                    input.asset_name
                ),
                error,
            )
        })?;
        effect.SetName(&input.asset_name);
        Ok(effect)
    })
}

fn builtin_reader(name: &str) -> Option<ContentTypeReader> {
    if name.starts_with("Microsoft.Xna.Framework.Content.ListReader`1") {
        if name.contains("Microsoft.Xna.Framework.Rectangle") {
            return Some(list_reader::<Rectangle>());
        }
        if name.contains("Microsoft.Xna.Framework.Vector3") {
            return Some(list_reader::<Vector3>());
        }
        if name.contains("System.Char") {
            return Some(list_reader::<char>());
        }
        return None;
    }
    Some(match name {
        "Microsoft.Xna.Framework.Content.BooleanReader" => {
            typed_reader::<bool, _>(ContentReaderExt::ReadBoolean)
        }
        "Microsoft.Xna.Framework.Content.ByteReader" => {
            typed_reader::<u8, _>(ContentReaderExt::ReadByte)
        }
        "Microsoft.Xna.Framework.Content.CharReader" => {
            typed_reader::<char, _>(ContentReaderExt::ReadChar)
        }
        "Microsoft.Xna.Framework.Content.DoubleReader" => {
            typed_reader::<f64, _>(ContentReader::ReadDouble)
        }
        "Microsoft.Xna.Framework.Content.Int16Reader" => {
            typed_reader::<i16, _>(ContentReaderExt::ReadInt16)
        }
        "Microsoft.Xna.Framework.Content.Int32Reader" => {
            typed_reader::<i32, _>(ContentReaderExt::ReadInt32)
        }
        "Microsoft.Xna.Framework.Content.Int64Reader" => {
            typed_reader::<i64, _>(ContentReaderExt::ReadInt64)
        }
        "Microsoft.Xna.Framework.Content.SByteReader" => {
            typed_reader::<i8, _>(ContentReaderExt::ReadSByte)
        }
        "Microsoft.Xna.Framework.Content.SingleReader" => {
            typed_reader::<f32, _>(ContentReader::ReadSingle)
        }
        "Microsoft.Xna.Framework.Content.StringReader" => {
            typed_reader::<String, _>(ContentReaderExt::ReadString)
        }
        "Microsoft.Xna.Framework.Content.UInt16Reader" => {
            typed_reader::<u16, _>(ContentReaderExt::ReadUInt16)
        }
        "Microsoft.Xna.Framework.Content.UInt32Reader" => {
            typed_reader::<u32, _>(ContentReaderExt::ReadUInt32)
        }
        "Microsoft.Xna.Framework.Content.UInt64Reader" => {
            typed_reader::<u64, _>(ContentReaderExt::ReadUInt64)
        }
        "Microsoft.Xna.Framework.Content.Vector2Reader" => {
            typed_reader::<Vector2, _>(ContentReader::ReadVector2)
        }
        "Microsoft.Xna.Framework.Content.Vector3Reader" => {
            typed_reader::<Vector3, _>(ContentReader::ReadVector3)
        }
        "Microsoft.Xna.Framework.Content.Vector4Reader" => {
            typed_reader::<Vector4, _>(ContentReader::ReadVector4)
        }
        "Microsoft.Xna.Framework.Content.MatrixReader" => {
            typed_reader::<Matrix, _>(ContentReader::ReadMatrix)
        }
        "Microsoft.Xna.Framework.Content.QuaternionReader" => {
            typed_reader::<Quaternion, _>(ContentReader::ReadQuaternion)
        }
        "Microsoft.Xna.Framework.Content.ColorReader" => {
            typed_reader::<Color, _>(ContentReader::ReadColor)
        }
        "Microsoft.Xna.Framework.Content.RectangleReader" => {
            typed_reader::<Rectangle, _>(|input| {
                Ok(Rectangle::new(
                    input.ReadInt32()?,
                    input.ReadInt32()?,
                    input.ReadInt32()?,
                    input.ReadInt32()?,
                ))
            })
        }
        "Microsoft.Xna.Framework.Content.Texture2DReader" => texture2d_reader(),
        "Microsoft.Xna.Framework.Content.SpriteFontReader" => sprite_font_reader(),
        "Microsoft.Xna.Framework.Content.EffectReader" => effect_reader(),
        _ => return None,
    })
}

fn builtin_reader_for_target(target: TypeId) -> Option<ContentTypeReader> {
    const NAMES: &[&str] = &[
        "Microsoft.Xna.Framework.Content.BooleanReader",
        "Microsoft.Xna.Framework.Content.ByteReader",
        "Microsoft.Xna.Framework.Content.CharReader",
        "Microsoft.Xna.Framework.Content.DoubleReader",
        "Microsoft.Xna.Framework.Content.Int16Reader",
        "Microsoft.Xna.Framework.Content.Int32Reader",
        "Microsoft.Xna.Framework.Content.Int64Reader",
        "Microsoft.Xna.Framework.Content.SByteReader",
        "Microsoft.Xna.Framework.Content.SingleReader",
        "Microsoft.Xna.Framework.Content.StringReader",
        "Microsoft.Xna.Framework.Content.UInt16Reader",
        "Microsoft.Xna.Framework.Content.UInt32Reader",
        "Microsoft.Xna.Framework.Content.UInt64Reader",
        "Microsoft.Xna.Framework.Content.Vector2Reader",
        "Microsoft.Xna.Framework.Content.Vector3Reader",
        "Microsoft.Xna.Framework.Content.Vector4Reader",
        "Microsoft.Xna.Framework.Content.MatrixReader",
        "Microsoft.Xna.Framework.Content.QuaternionReader",
        "Microsoft.Xna.Framework.Content.ColorReader",
        "Microsoft.Xna.Framework.Content.RectangleReader",
        "Microsoft.Xna.Framework.Content.Texture2DReader",
        "Microsoft.Xna.Framework.Content.SpriteFontReader",
        "Microsoft.Xna.Framework.Content.EffectReader",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Rectangle]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Vector3]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[System.Char]]",
    ];
    NAMES
        .iter()
        .filter_map(|name| builtin_reader(name))
        .find(|reader| reader.TargetType() == target)
}

fn is_builtin_value_type(target: TypeId) -> bool {
    target == TypeId::of::<bool>()
        || target == TypeId::of::<u8>()
        || target == TypeId::of::<i8>()
        || target == TypeId::of::<u16>()
        || target == TypeId::of::<i16>()
        || target == TypeId::of::<u32>()
        || target == TypeId::of::<i32>()
        || target == TypeId::of::<u64>()
        || target == TypeId::of::<i64>()
        || target == TypeId::of::<f32>()
        || target == TypeId::of::<f64>()
        || target == TypeId::of::<char>()
        || target == TypeId::of::<Vector2>()
        || target == TypeId::of::<Vector3>()
        || target == TypeId::of::<Vector4>()
        || target == TypeId::of::<Quaternion>()
        || target == TypeId::of::<Matrix>()
        || target == TypeId::of::<Color>()
        || target == TypeId::of::<Rectangle>()
}

fn normalize_external_reference(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(CnaError::InvalidInput(
                        "external content reference escapes the root directory",
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(CnaError::InvalidInput(
                    "external content reference must be relative",
                ));
            }
        }
    }
    if parts.is_empty() {
        Err(CnaError::InvalidInput(
            "external content reference is empty",
        ))
    } else {
        Ok(parts.join("/"))
    }
}

fn xnb_error(asset_name: &str, detail: &str) -> CnaError {
    content_error(format!(
        "content asset '{asset_name}' is not a valid XNB: {detail}"
    ))
}

fn type_name_for_any(_value: &ArcAny) -> &'static str {
    // Rust's `Any` exposes TypeId but not a reverse type-name lookup. The
    // expected type name and the reader identity are included by callers.
    "an unexpected registered Rust type"
}
