//! The `Tag` a content processor wrote, as a typed dictionary.
//!
//! XNA's `Model.Tag` is `object`, and the shape XNA's own
//! `TrianglePickingSample` puts there is a `Dictionary<string, object>`: the
//! processor tags each model with its world-space triangle vertices and a
//! bounding sphere, and the game reads them back to pick against real triangles
//! rather than against a bounding volume. That sample is why the route exists,
//! and this is how a Rust game reads it.
//!
//! # Why this one is safe to project and the reflective reader is not
//!
//! Every entry is *tagged*: a caller asks what an entry holds and CNA answers,
//! and the Rust side then picks the destination type from that answer. There is
//! no offset and no size for a caller to get wrong, so nothing here needs to be
//! `unsafe` for a caller. `content_readers.h`'s reflective builder is the
//! opposite shape -- it writes decoded values at caller-supplied byte offsets
//! into a caller-supplied pointer -- which is why that one is not bound.

#![allow(clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::content::{string_view, NativeContentManager};
use crate::native::Native;
use crate::value::{
    BoundingBox, BoundingSphere, Color, Matrix, Quaternion, Vector2, Vector3, Vector4,
};

/// One value a content processor stored under a key.
///
/// `Unknown` and `ForeignObject` are the two that carry no value: the first is
/// a .NET type this ABI has no shape for, and the second is an object a
/// caller's own reflective reader produced, which CNA never dereferences.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ObjectValue {
    /// A type this ABI does not express. [`ObjectDictionary::runtime_type_name`]
    /// says what it was.
    Unknown,
    Boolean(bool),
    Int32(i32),
    Single(f32),
    Double(f64),
    Text(String),
    Vector2(Vector2),
    Vector3(Vector3),
    Vector4(Vector4),
    Matrix(Matrix),
    Quaternion(Quaternion),
    Color(Color),
    BoundingSphere(BoundingSphere),
    BoundingBox(BoundingBox),
    /// An object a caller's own reflective reader made, as its opaque pointer.
    ForeignObject(*mut core::ffi::c_void),
}

/// An array of one of the value kinds.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ObjectArray {
    Boolean(Vec<bool>),
    Int32(Vec<i32>),
    Single(Vec<f32>),
    Double(Vec<f64>),
    Vector2(Vec<Vector2>),
    Vector3(Vec<Vector3>),
    Vector4(Vec<Vector4>),
    Matrix(Vec<Matrix>),
    Quaternion(Vec<Quaternion>),
    Color(Vec<Color>),
    BoundingSphere(Vec<BoundingSphere>),
    BoundingBox(Vec<BoundingBox>),
}

/// What an entry holds, without reading it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ObjectEntry {
    /// Which value, or the element type when [`Self::is_array`] is true.
    pub kind: ObjectValueKind,
    pub is_array: bool,
    /// The array length, or 1 for a scalar.
    pub element_count: u64,
}

/// Which value one entry holds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ObjectValueKind {
    Unknown,
    Boolean,
    Int32,
    Single,
    Double,
    Text,
    Vector2,
    Vector3,
    Vector4,
    Matrix,
    Quaternion,
    Color,
    BoundingSphere,
    BoundingBox,
    ForeignObject,
}

impl ObjectValueKind {
    const fn from_native(value: sys::CNA_ObjectDictionaryValueKind) -> Option<Self> {
        Some(match value {
            sys::CNA_OBJECT_DICTIONARY_VALUE_UNKNOWN => Self::Unknown,
            sys::CNA_OBJECT_DICTIONARY_VALUE_BOOLEAN => Self::Boolean,
            sys::CNA_OBJECT_DICTIONARY_VALUE_INT32 => Self::Int32,
            sys::CNA_OBJECT_DICTIONARY_VALUE_SINGLE => Self::Single,
            sys::CNA_OBJECT_DICTIONARY_VALUE_DOUBLE => Self::Double,
            sys::CNA_OBJECT_DICTIONARY_VALUE_STRING => Self::Text,
            sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR2 => Self::Vector2,
            sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR3 => Self::Vector3,
            sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR4 => Self::Vector4,
            sys::CNA_OBJECT_DICTIONARY_VALUE_MATRIX => Self::Matrix,
            sys::CNA_OBJECT_DICTIONARY_VALUE_QUATERNION => Self::Quaternion,
            sys::CNA_OBJECT_DICTIONARY_VALUE_COLOR => Self::Color,
            sys::CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_SPHERE => Self::BoundingSphere,
            sys::CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_BOX => Self::BoundingBox,
            sys::CNA_OBJECT_DICTIONARY_VALUE_FOREIGN_OBJECT => Self::ForeignObject,
            _ => return None,
        })
    }

    const fn to_native(self) -> sys::CNA_ObjectDictionaryValueKind {
        match self {
            Self::Unknown => sys::CNA_OBJECT_DICTIONARY_VALUE_UNKNOWN,
            Self::Boolean => sys::CNA_OBJECT_DICTIONARY_VALUE_BOOLEAN,
            Self::Int32 => sys::CNA_OBJECT_DICTIONARY_VALUE_INT32,
            Self::Single => sys::CNA_OBJECT_DICTIONARY_VALUE_SINGLE,
            Self::Double => sys::CNA_OBJECT_DICTIONARY_VALUE_DOUBLE,
            Self::Text => sys::CNA_OBJECT_DICTIONARY_VALUE_STRING,
            Self::Vector2 => sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR2,
            Self::Vector3 => sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR3,
            Self::Vector4 => sys::CNA_OBJECT_DICTIONARY_VALUE_VECTOR4,
            Self::Matrix => sys::CNA_OBJECT_DICTIONARY_VALUE_MATRIX,
            Self::Quaternion => sys::CNA_OBJECT_DICTIONARY_VALUE_QUATERNION,
            Self::Color => sys::CNA_OBJECT_DICTIONARY_VALUE_COLOR,
            Self::BoundingSphere => sys::CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_SPHERE,
            Self::BoundingBox => sys::CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_BOX,
            Self::ForeignObject => sys::CNA_OBJECT_DICTIONARY_VALUE_FOREIGN_OBJECT,
        }
    }
}

/// A `Dictionary<string, object>` a content processor wrote.
///
/// Owned, and it outlives the asset it came from: upstream states the handle
/// "keeps the loaded model's data alive on its own, so destroying the model
/// first is safe and does not invalidate it".
#[derive(Debug)]
pub struct ObjectDictionary {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_ObjectDictionaryHandle>,
}

impl ObjectDictionary {
    pub(crate) fn from_owned_handle(
        native: &Arc<Native>,
        handle: sys::CNA_ObjectDictionaryHandle,
    ) -> Self {
        Self {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
        }
    }

    /// Loads a dictionary asset by name.
    pub fn load(content_manager: &NativeContentManager, asset_name: &str) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is borrowed for the call, the name is
        // borrowed and copied, and the output is a live local.
        native.check(unsafe {
            (native.runtime.content_manager_load_object_dictionary_ext)(
                content_manager.handle(),
                string_view(asset_name),
                &mut handle,
            )
        })?;
        Ok(Self::from_owned_handle(&native, handle))
    }

    fn get(&self) -> Result<sys::CNA_ObjectDictionaryHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the dictionary has been released"));
        }
        Ok(handle)
    }

    /// How many entries the dictionary holds.
    pub fn len(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Whether the dictionary holds no entries.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Every key, in the dictionary's own order.
    pub fn keys(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let count = self.len()?;
        (0..count)
            .map(|index| {
                crate::native::runtime::read_string(
                    |value| self.native.check(value),
                    // SAFETY: owned handle, live outputs; the size-then-copy pair.
                    |bytes| unsafe {
                        (api.object_dictionary_ext_get_key_size_at)(handle, index, bytes)
                    },
                    |destination, capacity, written| unsafe {
                        (api.object_dictionary_ext_copy_key_at)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    },
                )
            })
            .collect()
    }

    /// Whether the dictionary holds that key.
    pub fn contains_key(&self, key: &str) -> Result<bool> {
        let handle = self.get()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned, the key is borrowed for the call, and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_contains_key)(
                handle,
                string_view(key),
                &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// What one entry holds, without reading it.
    pub fn entry(&self, key: &str) -> Result<ObjectEntry> {
        let handle = self.get()?;
        let mut value = sys::CNA_ObjectDictionaryEntry {
            struct_size: core::mem::size_of::<sys::CNA_ObjectDictionaryEntry>() as u32,
            struct_version: 1,
            ..sys::CNA_ObjectDictionaryEntry::default()
        };
        // SAFETY: the handle is owned, the key is borrowed, and the output is a
        // live local whose size header is set as the route requires.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_get_entry)(
                handle,
                string_view(key),
                &mut value,
            )
        })?;
        Ok(ObjectEntry {
            kind: ObjectValueKind::from_native(value.kind).ok_or(CnaError::InvalidInput(
                "CNA reported a dictionary value kind this build does not know",
            ))?,
            is_array: value.is_array != sys::CNA_FALSE,
            element_count: value.element_count,
        })
    }

    /// The dictionary's own managed runtime type name.
    ///
    /// The stable managed identity the compiled asset retained -- something
    /// like `System.Collections.Generic.Dictionary\`2[System.String,System.Object]`
    /// -- and a property of the whole dictionary rather than of an entry, which
    /// is what its route takes and what its name would otherwise suggest.
    pub fn runtime_type_name(&self) -> Result<String> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; the size-then-copy pair.
            |bytes| unsafe {
                (api.object_dictionary_ext_get_runtime_type_name_size)(handle, bytes)
            },
            |destination, capacity, written| unsafe {
                (api.object_dictionary_ext_copy_runtime_type_name)(
                    handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// Releases the dictionary early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.object_dictionary_ext_destroy)(handle) })
    }
}

impl Drop for ObjectDictionary {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Reading values.
///
/// Each of these asks the entry's kind first and then reads with the type CNA
/// named. A caller never supplies a size or a kind, which is what keeps the
/// whole surface safe: `cna_object_dictionary_ext_copy_value` writes `kind`'s
/// value into `capacity` bytes, and the only way to get that wrong is to pass a
/// kind the entry does not hold.
impl ObjectDictionary {
    /// The value under a key.
    ///
    /// `Unknown` and `ForeignObject` carry no value of their own: the first has
    /// no shape in this ABI -- [`Self::runtime_type_name`] says what the
    /// dictionary is -- and the second is read with
    /// [`Self::foreign_object`].
    pub fn value(&self, key: &str) -> Result<ObjectValue> {
        let entry = self.entry(key)?;
        if entry.is_array {
            return Err(CnaError::InvalidInput(
                "that entry holds an array; read it with `array`",
            ));
        }
        Ok(match entry.kind {
            ObjectValueKind::Unknown => ObjectValue::Unknown,
            ObjectValueKind::Boolean => {
                ObjectValue::Boolean(self.scalar::<sys::CNA_Bool>(key, entry.kind)? != 0)
            }
            ObjectValueKind::Int32 => ObjectValue::Int32(self.scalar(key, entry.kind)?),
            ObjectValueKind::Single => ObjectValue::Single(self.scalar(key, entry.kind)?),
            ObjectValueKind::Double => ObjectValue::Double(self.scalar(key, entry.kind)?),
            ObjectValueKind::Text => ObjectValue::Text(self.text(key)?),
            ObjectValueKind::Vector2 => {
                ObjectValue::Vector2(vector2(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::Vector3 => {
                ObjectValue::Vector3(vector3(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::Vector4 => {
                ObjectValue::Vector4(vector4(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::Matrix => ObjectValue::Matrix(matrix(self.scalar(key, entry.kind)?)),
            ObjectValueKind::Quaternion => {
                ObjectValue::Quaternion(quaternion(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::Color => ObjectValue::Color(color(self.scalar(key, entry.kind)?)),
            ObjectValueKind::BoundingSphere => {
                ObjectValue::BoundingSphere(bounding_sphere(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::BoundingBox => {
                ObjectValue::BoundingBox(bounding_box(self.scalar(key, entry.kind)?))
            }
            ObjectValueKind::ForeignObject => ObjectValue::ForeignObject(self.foreign_object(key)?),
        })
    }

    /// The array under a key.
    ///
    /// The shape XNA's `TrianglePickingSample` stores: a `Vector3[]` of
    /// world-space triangle vertices.
    pub fn array(&self, key: &str) -> Result<ObjectArray> {
        let entry = self.entry(key)?;
        if !entry.is_array {
            return Err(CnaError::InvalidInput(
                "that entry holds one value; read it with `value`",
            ));
        }
        let count = entry.element_count;
        Ok(match entry.kind {
            ObjectValueKind::Boolean => ObjectArray::Boolean(
                self.elements::<sys::CNA_Bool>(key, entry.kind, count)?
                    .into_iter()
                    .map(|value| value != 0)
                    .collect(),
            ),
            ObjectValueKind::Int32 => {
                ObjectArray::Int32(self.elements(key, entry.kind, count)?)
            }
            ObjectValueKind::Single => {
                ObjectArray::Single(self.elements(key, entry.kind, count)?)
            }
            ObjectValueKind::Double => {
                ObjectArray::Double(self.elements(key, entry.kind, count)?)
            }
            ObjectValueKind::Vector2 => ObjectArray::Vector2(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(vector2)
                    .collect(),
            ),
            ObjectValueKind::Vector3 => ObjectArray::Vector3(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(vector3)
                    .collect(),
            ),
            ObjectValueKind::Vector4 => ObjectArray::Vector4(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(vector4)
                    .collect(),
            ),
            ObjectValueKind::Matrix => ObjectArray::Matrix(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(matrix)
                    .collect(),
            ),
            ObjectValueKind::Quaternion => ObjectArray::Quaternion(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(quaternion)
                    .collect(),
            ),
            ObjectValueKind::Color => ObjectArray::Color(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(color)
                    .collect(),
            ),
            ObjectValueKind::BoundingSphere => ObjectArray::BoundingSphere(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(bounding_sphere)
                    .collect(),
            ),
            ObjectValueKind::BoundingBox => ObjectArray::BoundingBox(
                self.elements(key, entry.kind, count)?
                    .into_iter()
                    .map(bounding_box)
                    .collect(),
            ),
            ObjectValueKind::Unknown
            | ObjectValueKind::Text
            | ObjectValueKind::ForeignObject => {
                return Err(CnaError::InvalidInput(
                    "this ABI has no array shape for that entry kind",
                ))
            }
        })
    }

    /// The string under a key.
    pub fn text(&self, key: &str) -> Result<String> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, borrowed key, live outputs.
            |bytes| unsafe {
                (api.object_dictionary_ext_get_string_size)(handle, string_view(key), bytes)
            },
            |destination, capacity, written| unsafe {
                (api.object_dictionary_ext_copy_string)(
                    handle,
                    string_view(key),
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// The pointer a caller's own reflective reader produced.
    ///
    /// CNA never dereferences, copies or frees it; it is the caller's own and
    /// stays valid as long as the caller keeps it so. Returned raw for exactly
    /// that reason -- a safe Rust value here would be claiming an ownership
    /// nobody has.
    pub fn foreign_object(&self, key: &str) -> Result<*mut core::ffi::c_void> {
        let handle = self.get()?;
        let mut value = core::ptr::null_mut();
        // SAFETY: the handle is owned, the key is borrowed for the call, and
        // the output is a live local. The pointer is not dereferenced here.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_get_foreign_object)(
                handle,
                string_view(key),
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Reads one value of the type CNA said the entry holds.
    fn scalar<T: Copy + Default>(&self, key: &str, kind: ObjectValueKind) -> Result<T> {
        let handle = self.get()?;
        let mut value = T::default();
        // SAFETY: the handle is owned, the key is borrowed, and the
        // destination is a live local of exactly the type `kind` names -- the
        // kind came from `entry()`, not from a caller.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_copy_value)(
                handle,
                string_view(key),
                kind.to_native(),
                core::ptr::addr_of_mut!(value).cast::<core::ffi::c_void>(),
                core::mem::size_of::<T>() as u64,
            )
        })?;
        Ok(value)
    }

    /// Reads `count` elements of the type CNA said the entry holds.
    fn elements<T: Copy + Default>(
        &self,
        key: &str,
        kind: ObjectValueKind,
        count: u64,
    ) -> Result<Vec<T>> {
        let handle = self.get()?;
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("more elements than fit in memory"))?;
        let mut buffer = vec![T::default(); capacity];
        let byte_capacity = (capacity * core::mem::size_of::<T>()) as u64;
        let mut written = 0_u64;
        // SAFETY: as in `scalar`, and the destination holds exactly
        // `count * size_of::<T>()` bytes, which is what the element count CNA
        // itself reported multiplies out to.
        self.native.check(unsafe {
            (self.native.runtime.object_dictionary_ext_copy_array)(
                handle,
                string_view(key),
                kind.to_native(),
                buffer.as_mut_ptr().cast::<core::ffi::c_void>(),
                byte_capacity,
                &mut written,
            )
        })?;
        let elements = (written as usize) / core::mem::size_of::<T>();
        buffer.truncate(elements.min(capacity));
        Ok(buffer)
    }
}

const fn vector2(value: sys::CNA_Vector2) -> Vector2 {
    Vector2 { X: value.x, Y: value.y }
}

const fn vector3(value: sys::CNA_Vector3) -> Vector3 {
    Vector3 { X: value.x, Y: value.y, Z: value.z }
}

const fn vector4(value: sys::CNA_Vector4) -> Vector4 {
    Vector4 { X: value.x, Y: value.y, Z: value.z, W: value.w }
}

const fn quaternion(value: sys::CNA_Quaternion) -> Quaternion {
    Quaternion { X: value.x, Y: value.y, Z: value.z, W: value.w }
}

const fn matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix {
        M11: value.m11, M12: value.m12, M13: value.m13, M14: value.m14,
        M21: value.m21, M22: value.m22, M23: value.m23, M24: value.m24,
        M31: value.m31, M32: value.m32, M33: value.m33, M34: value.m34,
        M41: value.m41, M42: value.m42, M43: value.m43, M44: value.m44,
    }
}

fn color(value: sys::CNA_Color) -> Color {
    // The same packing `Color` uses internally: R in the low byte, then G, B,
    // A. Built through the public setter rather than reaching into the packed
    // field, so a change to that representation cannot silently desync.
    let mut result = Color::default();
    result.SetPackedValue(
        u32::from(value.r)
            | (u32::from(value.g) << 8)
            | (u32::from(value.b) << 16)
            | (u32::from(value.a) << 24),
    );
    result
}

const fn bounding_sphere(value: sys::CNA_BoundingSphere) -> BoundingSphere {
    BoundingSphere {
        Center: vector3(value.center),
        Radius: value.radius,
    }
}

const fn bounding_box(value: sys::CNA_BoundingBox) -> BoundingBox {
    BoundingBox {
        Min: vector3(value.min),
        Max: vector3(value.max),
    }
}
