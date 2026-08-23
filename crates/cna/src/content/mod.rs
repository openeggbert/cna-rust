//! Managed XNA content cache and XNB reader infrastructure.

#![allow(clippy::module_name_repetitions)]

mod error;
mod lzx;
mod manager;
mod reader;
mod serializer;

pub use error::{ContentLoadException, SerializationInfo, StreamingContext};
pub use manager::{ContentManager, ResourceContentManager};
pub use reader::{
    ContentReader, ContentTypeReader, ContentTypeReaderManager, ContentTypeReaderOfT,
};
pub use serializer::{
    ContentSerializerAttribute, ContentSerializerCollectionItemNameAttribute,
    ContentSerializerIgnoreAttribute, ContentSerializerRuntimeTypeAttribute,
    ContentSerializerTypeVersionAttribute,
};

pub use manager::{
    ContentDisposable, ContentDisposableRecorder, ContentLoadable, ContentManagerBase,
    ContentResourceProvider,
};
pub use reader::{
    ContentReaderBase, ContentReaderExt, ContentTypeReaderBase, ContentTypeReaderRegistration,
    ContentTypeReaderRegistry,
};
