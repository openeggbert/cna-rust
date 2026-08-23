#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{CnaError, Result};
use crate::game::ServiceProvider;
use crate::graphics::GraphicsDevice;

use super::reader::ContentReader;
use super::ContentLoadException;

type ArcAny = Arc<dyn Any + Send + Sync>;

/// A content-owned object that can be released by `ContentManager.Unload`.
pub trait ContentDisposable: Send + Sync {
    fn DisposeContent(&self) -> Result<()>;
}

/// Type contract used by the typed XNB cache and reader registry.
pub trait ContentLoadable: Any + Send + Sync + 'static {
    /// Returns the disposable identity for this loaded value when it owns a
    /// resource. Value-only content uses the default non-disposable behavior.
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        let _ = value;
        None
    }
}

macro_rules! value_content_loadable {
    ($($type:ty),+ $(,)?) => {$(
        impl ContentLoadable for $type {}
    )+};
}

value_content_loadable!(
    bool,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    f32,
    f64,
    char,
    String,
    crate::value::Vector2,
    crate::value::Vector3,
    crate::value::Vector4,
    crate::value::Quaternion,
    crate::value::Matrix,
    crate::value::Color,
    crate::value::Rectangle,
);

impl<T: ContentLoadable> ContentLoadable for Vec<T> {}

/// Strongly typed replacement for XNA's disposable-record callback.
pub trait ContentDisposableRecorder: Send + Sync {
    fn Record(&self, value: Arc<dyn ContentDisposable>) -> Result<()>;
}

impl<F> ContentDisposableRecorder for F
where
    F: Fn(Arc<dyn ContentDisposable>) -> Result<()> + Send + Sync,
{
    fn Record(&self, value: Arc<dyn ContentDisposable>) -> Result<()> {
        self(value)
    }
}

/// Resource byte provider used by `ResourceContentManager` without exposing a
/// CLR `ResourceManager` facade unrelated to the XNA namespace.
pub trait ContentResourceProvider: Send + Sync {
    fn GetObject(&self, assetName: &str) -> Result<Option<Vec<u8>>>;
}

/// Rust composition contract for the XNA `ContentManager` base relationship.
pub trait ContentManagerBase {
    fn ServiceProvider(&self) -> Arc<dyn ServiceProvider>;
    fn RootDirectory(&self) -> String;
    fn SetRootDirectory(&self, value: &str) -> Result<()>;
    fn Load<T: ContentLoadable>(&self, assetName: &str) -> Result<Arc<T>>;
    fn ReadAsset<T: ContentLoadable>(
        &self,
        assetName: &str,
        recordDisposableObject: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Arc<T>>;
    fn OpenStream(&self, assetName: &str) -> Result<Box<dyn Read + Send>>;
    fn Dispose(&self) -> Result<()>;
    fn Unload(&self) -> Result<()>;
}

#[derive(Clone)]
enum ContentSource {
    FileSystem,
    Resource(Arc<dyn ContentResourceProvider>),
}

struct CacheEntry {
    target_type: TypeId,
    value: ArcAny,
}

struct DisposableEntry {
    identity: usize,
    value: Arc<dyn ContentDisposable>,
}

pub(crate) struct ContentManagerInner {
    service_provider: Arc<dyn ServiceProvider>,
    root_directory: Mutex<String>,
    source: ContentSource,
    assets: Mutex<HashMap<String, CacheEntry>>,
    disposables: Mutex<Vec<DisposableEntry>>,
    disposable_identities: Mutex<HashSet<usize>>,
    operation: Mutex<()>,
    graphics_device: Mutex<Option<GraphicsDevice>>,
    disposed: AtomicBool,
}

/// Typed XNA content cache and managed XNB loader.
#[derive(Clone)]
pub struct ContentManager {
    pub(crate) inner: Arc<ContentManagerInner>,
}

#[allow(non_snake_case)]
impl ContentManager {
    #[must_use]
    pub fn new(serviceProvider: Arc<dyn ServiceProvider>) -> Self {
        Self::with_source(serviceProvider, String::new(), ContentSource::FileSystem)
    }

    #[must_use]
    pub fn from_service_provider_and_root_directory(
        serviceProvider: Arc<dyn ServiceProvider>,
        rootDirectory: &str,
    ) -> Self {
        Self::with_source(
            serviceProvider,
            rootDirectory.to_owned(),
            ContentSource::FileSystem,
        )
    }

    fn with_source(
        service_provider: Arc<dyn ServiceProvider>,
        root_directory: String,
        source: ContentSource,
    ) -> Self {
        Self {
            inner: Arc::new(ContentManagerInner {
                service_provider,
                root_directory: Mutex::new(root_directory),
                source,
                assets: Mutex::new(HashMap::new()),
                disposables: Mutex::new(Vec::new()),
                disposable_identities: Mutex::new(HashSet::new()),
                operation: Mutex::new(()),
                graphics_device: Mutex::new(None),
                disposed: AtomicBool::new(false),
            }),
        }
    }

    #[must_use]
    pub fn ServiceProvider(&self) -> Arc<dyn ServiceProvider> {
        Arc::clone(&self.inner.service_provider)
    }

    #[must_use]
    pub fn RootDirectory(&self) -> String {
        self.inner
            .root_directory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn SetRootDirectory(&self, value: &str) -> Result<()> {
        self.ensure_open()?;
        let _operation = self
            .inner
            .operation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self
            .inner
            .assets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
        {
            return Err(CnaError::InvalidInput(
                "the content root directory cannot change after an asset is loaded",
            ));
        }
        *self
            .inner
            .root_directory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value.to_owned();
        Ok(())
    }

    pub fn Load<T: ContentLoadable>(&self, assetName: &str) -> Result<Arc<T>> {
        self.ensure_open()?;
        let normalized = normalize_asset_name(assetName)?;
        let key = normalized.to_lowercase();
        if let Some(cached) = self
            .inner
            .assets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
        {
            if cached.target_type != TypeId::of::<T>() {
                return Err(content_error(format!(
                    "content asset '{assetName}' was cached with a different Rust type"
                )));
            }
            return Arc::clone(&cached.value)
                .downcast::<T>()
                .map_err(|_| content_error("cached content type identity is inconsistent"));
        }

        let loaded = self.read_asset_locked::<T>(&normalized, None)?;
        let erased: ArcAny = Arc::clone(&loaded) as ArcAny;
        self.inner
            .assets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                key,
                CacheEntry {
                    target_type: TypeId::of::<T>(),
                    value: erased,
                },
            );
        Ok(loaded)
    }

    pub fn ReadAsset<T: ContentLoadable>(
        &self,
        assetName: &str,
        recordDisposableObject: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Arc<T>> {
        self.ensure_open()?;
        let normalized = normalize_asset_name(assetName)?;
        self.read_asset_locked::<T>(&normalized, recordDisposableObject)
    }

    fn read_asset_locked<T: ContentLoadable>(
        &self,
        normalized_asset_name: &str,
        recorder: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Arc<T>> {
        let records_with_manager = recorder.is_none();
        let disposable_start = self
            .inner
            .disposables
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let loaded = (|| {
            let mut stream = self.open_stream_normalized(normalized_asset_name)?;
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes).map_err(|error| {
                content_error_with_inner(
                    format!("could not read content asset '{normalized_asset_name}'"),
                    error,
                )
            })?;
            let reader =
                ContentReader::create(self.clone(), bytes, normalized_asset_name, recorder)?;
            reader.read_asset::<T>()
        })();
        if loaded.is_err() && records_with_manager {
            self.cleanup_partial_load(disposable_start);
        }
        loaded
    }

    pub fn OpenStream(&self, assetName: &str) -> Result<Box<dyn Read + Send>> {
        self.ensure_open()?;
        let normalized = normalize_asset_name(assetName)?;
        self.open_stream_normalized(&normalized)
    }

    fn open_stream_normalized(&self, asset_name: &str) -> Result<Box<dyn Read + Send>> {
        match &self.inner.source {
            ContentSource::FileSystem => {
                let root = self.RootDirectory();
                let path = Path::new(&root).join(format!("{asset_name}.xnb"));
                File::open(path)
                    .map(|file| Box::new(file) as Box<dyn Read + Send>)
                    .map_err(|error| {
                        content_error_with_inner(
                            format!("could not open content asset '{asset_name}'"),
                            error,
                        )
                    })
            }
            ContentSource::Resource(provider) => provider
                .GetObject(asset_name)?
                .map(|bytes| Box::new(Cursor::new(bytes)) as Box<dyn Read + Send>)
                .ok_or_else(|| content_error(format!("resource '{asset_name}' was not found"))),
        }
    }

    pub fn Unload(&self) -> Result<()> {
        self.ensure_open()?;
        let _operation = self
            .inner
            .operation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.unload_locked()
    }

    fn unload_locked(&self) -> Result<()> {
        self.inner
            .assets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        let disposables = std::mem::take(
            &mut *self
                .inner
                .disposables
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        self.inner
            .disposable_identities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        let mut first_error = None;
        for entry in disposables.into_iter().rev() {
            if let Err(error) = entry.value.DisposeContent() {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn Dispose(&self) -> Result<()> {
        self.DisposeWithDisposing(true)
    }

    pub fn DisposeWithDisposing(&self, disposing: bool) -> Result<()> {
        if self.inner.disposed.load(Ordering::Acquire) {
            return Ok(());
        }
        let _operation = self
            .inner
            .operation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.inner.disposed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        if disposing {
            self.unload_locked()
        } else {
            self.inner
                .assets
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
            self.inner
                .disposables
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
            self.inner
                .disposable_identities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
            Ok(())
        }
    }

    pub(crate) fn record_disposable(&self, value: Arc<dyn ContentDisposable>) -> Result<()> {
        self.ensure_open()?;
        let identity = Arc::as_ptr(&value).cast::<()>() as usize;
        let mut identities = self
            .inner
            .disposable_identities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if identities.insert(identity) {
            self.inner
                .disposables
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(DisposableEntry { identity, value });
        }
        Ok(())
    }

    fn cleanup_partial_load(&self, start: usize) {
        let removed = {
            let mut disposables = self
                .inner
                .disposables
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if start >= disposables.len() {
                Vec::new()
            } else {
                disposables.drain(start..).collect::<Vec<_>>()
            }
        };
        let mut identities = self
            .inner
            .disposable_identities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for entry in &removed {
            identities.remove(&entry.identity);
        }
        drop(identities);
        for entry in removed.into_iter().rev() {
            let _ = entry.value.DisposeContent();
        }
    }

    fn ensure_open(&self) -> Result<()> {
        if self.inner.disposed.load(Ordering::Acquire) {
            Err(CnaError::InvalidInput("content manager is disposed"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn cleanup_for_game_shutdown(&self) -> Result<()> {
        if self.inner.disposed.load(Ordering::Acquire) {
            Ok(())
        } else {
            self.Unload()
        }
    }

    pub(crate) fn bind_graphics_device(&self, device: &GraphicsDevice) -> Result<()> {
        let mut current = self
            .inner
            .graphics_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(current) = current.as_ref() {
            if !current.is_same_device(device) {
                return Err(CnaError::InvalidInput(
                    "a ContentManager cannot load graphics resources for multiple graphics devices",
                ));
            }
        } else {
            *current = Some(device.clone());
        }
        Ok(())
    }

    pub(crate) fn graphics_device(&self) -> Result<GraphicsDevice> {
        self.ensure_open()?;
        self.inner
            .graphics_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::InvalidInput(
                "graphics content requires a ContentManager attached to a running Game",
            ))
    }
}

impl ContentManagerBase for ContentManager {
    fn ServiceProvider(&self) -> Arc<dyn ServiceProvider> {
        Self::ServiceProvider(self)
    }

    fn RootDirectory(&self) -> String {
        Self::RootDirectory(self)
    }

    fn SetRootDirectory(&self, value: &str) -> Result<()> {
        Self::SetRootDirectory(self, value)
    }

    fn Load<T: ContentLoadable>(&self, assetName: &str) -> Result<Arc<T>> {
        Self::Load(self, assetName)
    }

    fn ReadAsset<T: ContentLoadable>(
        &self,
        assetName: &str,
        recordDisposableObject: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Arc<T>> {
        Self::ReadAsset(self, assetName, recordDisposableObject)
    }

    fn OpenStream(&self, assetName: &str) -> Result<Box<dyn Read + Send>> {
        Self::OpenStream(self, assetName)
    }

    fn Dispose(&self) -> Result<()> {
        Self::Dispose(self)
    }

    fn Unload(&self) -> Result<()> {
        Self::Unload(self)
    }
}

impl Drop for ContentManager {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            let _ = self.Dispose();
        }
    }
}

/// Content manager whose XNB streams come from a resource provider.
pub struct ResourceContentManager {
    manager: ContentManager,
}

#[allow(non_snake_case)]
impl ResourceContentManager {
    #[must_use]
    pub fn new(
        serviceProvider: Arc<dyn ServiceProvider>,
        resourceManager: Arc<dyn ContentResourceProvider>,
    ) -> Self {
        Self {
            manager: ContentManager::with_source(
                serviceProvider,
                String::new(),
                ContentSource::Resource(resourceManager),
            ),
        }
    }

    pub fn OpenStream(&self, assetName: &str) -> Result<Box<dyn Read + Send>> {
        self.manager.OpenStream(assetName)
    }
}

impl ContentManagerBase for ResourceContentManager {
    fn ServiceProvider(&self) -> Arc<dyn ServiceProvider> {
        self.manager.ServiceProvider()
    }

    fn RootDirectory(&self) -> String {
        self.manager.RootDirectory()
    }

    fn SetRootDirectory(&self, value: &str) -> Result<()> {
        self.manager.SetRootDirectory(value)
    }

    fn Load<T: ContentLoadable>(&self, assetName: &str) -> Result<Arc<T>> {
        self.manager.Load(assetName)
    }

    fn ReadAsset<T: ContentLoadable>(
        &self,
        assetName: &str,
        recordDisposableObject: Option<Arc<dyn ContentDisposableRecorder>>,
    ) -> Result<Arc<T>> {
        self.manager.ReadAsset(assetName, recordDisposableObject)
    }

    fn OpenStream(&self, assetName: &str) -> Result<Box<dyn Read + Send>> {
        Self::OpenStream(self, assetName)
    }

    fn Dispose(&self) -> Result<()> {
        self.manager.Dispose()
    }

    fn Unload(&self) -> Result<()> {
        self.manager.Unload()
    }
}

impl Drop for ResourceContentManager {
    fn drop(&mut self) {
        let _ = self.manager.Dispose();
    }
}

fn normalize_asset_name(value: &str) -> Result<String> {
    if value.is_empty() {
        return Err(CnaError::InvalidInput(
            "content asset name must not be empty",
        ));
    }
    let portable = value.replace('\\', "/");
    let mut parts = Vec::new();
    for component in PathBuf::from(&portable).components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(CnaError::InvalidInput(
                        "content asset path escapes the root directory",
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(CnaError::InvalidInput(
                    "content asset path must be relative",
                ));
            }
        }
    }
    if parts.is_empty() {
        Err(CnaError::InvalidInput(
            "content asset name must not be empty",
        ))
    } else {
        Ok(parts.join("/"))
    }
}

pub(crate) fn content_error(message: impl Into<String>) -> CnaError {
    CnaError::Content(ContentLoadException::from_message(&message.into()))
}

pub(crate) fn content_error_with_inner(
    message: impl Into<String>,
    inner: impl std::fmt::Display,
) -> CnaError {
    CnaError::Content(ContentLoadException::with_inner_message(
        message.into(),
        inner.to_string(),
    ))
}
