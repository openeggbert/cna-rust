#![allow(non_snake_case, clippy::struct_excessive_bools)]

/// Mutable XNA content-serialization metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSerializerAttribute {
    element_name: String,
    flatten_content: bool,
    optional: bool,
    allow_null: bool,
    shared_resource: bool,
    collection_item_name: Option<String>,
}

#[allow(non_snake_case)]
impl ContentSerializerAttribute {
    #[must_use]
    pub fn new() -> Self {
        Self {
            element_name: String::new(),
            flatten_content: false,
            optional: false,
            allow_null: true,
            shared_resource: false,
            collection_item_name: None,
        }
    }

    #[must_use]
    pub fn ElementName(&self) -> String {
        self.element_name.clone()
    }

    pub fn SetElementName(&mut self, value: &str) {
        self.element_name = value.to_owned();
    }

    #[must_use]
    pub const fn FlattenContent(&self) -> bool {
        self.flatten_content
    }

    pub fn SetFlattenContent(&mut self, value: bool) {
        self.flatten_content = value;
    }

    #[must_use]
    pub const fn Optional(&self) -> bool {
        self.optional
    }

    pub fn SetOptional(&mut self, value: bool) {
        self.optional = value;
    }

    #[must_use]
    pub const fn AllowNull(&self) -> bool {
        self.allow_null
    }

    pub fn SetAllowNull(&mut self, value: bool) {
        self.allow_null = value;
    }

    #[must_use]
    pub const fn SharedResource(&self) -> bool {
        self.shared_resource
    }

    pub fn SetSharedResource(&mut self, value: bool) {
        self.shared_resource = value;
    }

    #[must_use]
    pub fn CollectionItemName(&self) -> String {
        self.collection_item_name
            .clone()
            .unwrap_or_else(|| "Item".to_owned())
    }

    /// # Panics
    ///
    /// Panics when `value` is empty, matching XNA's rejected empty item name.
    pub fn SetCollectionItemName(&mut self, value: &str) {
        assert!(!value.is_empty(), "CollectionItemName must not be empty");
        self.collection_item_name = Some(value.to_owned());
    }

    #[must_use]
    pub const fn HasCollectionItemName(&self) -> bool {
        self.collection_item_name.is_some()
    }

    #[must_use]
    pub fn Clone(&self) -> Self {
        <Self as std::clone::Clone>::clone(self)
    }
}

impl Default for ContentSerializerAttribute {
    fn default() -> Self {
        Self::new()
    }
}

/// Specifies the serialized item element name for a collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSerializerCollectionItemNameAttribute {
    collection_item_name: String,
}

#[allow(non_snake_case)]
impl ContentSerializerCollectionItemNameAttribute {
    #[must_use]
    pub fn new(collectionItemName: &str) -> Self {
        Self {
            collection_item_name: collectionItemName.to_owned(),
        }
    }

    #[must_use]
    pub fn CollectionItemName(&self) -> String {
        self.collection_item_name.clone()
    }
}

/// Marks a content member as excluded from serialization.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContentSerializerIgnoreAttribute;

impl ContentSerializerIgnoreAttribute {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Stores the runtime type name emitted for an intermediate content value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSerializerRuntimeTypeAttribute {
    runtime_type: String,
}

#[allow(non_snake_case)]
impl ContentSerializerRuntimeTypeAttribute {
    #[must_use]
    pub fn new(runtimeType: &str) -> Self {
        Self {
            runtime_type: runtimeType.to_owned(),
        }
    }

    #[must_use]
    pub fn RuntimeType(&self) -> String {
        self.runtime_type.clone()
    }
}

/// Stores the content serializer version assigned to a type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentSerializerTypeVersionAttribute {
    type_version: i32,
}

#[allow(non_snake_case)]
impl ContentSerializerTypeVersionAttribute {
    #[must_use]
    pub const fn new(typeVersion: i32) -> Self {
        Self {
            type_version: typeVersion,
        }
    }

    #[must_use]
    pub const fn TypeVersion(&self) -> i32 {
        self.type_version
    }
}
