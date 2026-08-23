#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::content::ContentLoadable;
use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::value::{Color, Vector2, Vector3};

use super::resource::EventHandlers;
use super::{GraphicsDevice, GraphicsResource, VertexElementFormat, VertexElementUsage};

/// XNA vertex-declaration element with an ABI-stable 16-byte representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct VertexElement {
    offset: i32,
    element_format: VertexElementFormat,
    element_usage: VertexElementUsage,
    usage_index: i32,
}

impl Default for VertexElement {
    fn default() -> Self {
        Self::new(
            0,
            VertexElementFormat::Single,
            VertexElementUsage::Position,
            0,
        )
    }
}

#[allow(non_snake_case)]
impl VertexElement {
    #[must_use]
    pub const fn new(
        offset: i32,
        elementFormat: VertexElementFormat,
        elementUsage: VertexElementUsage,
        usageIndex: i32,
    ) -> Self {
        Self {
            offset,
            element_format: elementFormat,
            element_usage: elementUsage,
            usage_index: usageIndex,
        }
    }

    #[must_use]
    pub const fn Offset(&self) -> i32 {
        self.offset
    }

    pub fn SetOffset(&mut self, value: i32) {
        self.offset = value;
    }

    #[must_use]
    pub const fn VertexElementFormat(&self) -> VertexElementFormat {
        self.element_format
    }

    pub fn SetVertexElementFormat(&mut self, value: VertexElementFormat) {
        self.element_format = value;
    }

    #[must_use]
    pub const fn VertexElementUsage(&self) -> VertexElementUsage {
        self.element_usage
    }

    pub fn SetVertexElementUsage(&mut self, value: VertexElementUsage) {
        self.element_usage = value;
    }

    #[must_use]
    pub const fn UsageIndex(&self) -> i32 {
        self.usage_index
    }

    pub fn SetUsageIndex(&mut self, value: i32) {
        self.usage_index = value;
    }

    #[must_use]
    pub const fn GetHashCode(&self) -> i32 {
        0
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{{{Offset:{} Format:{:?} Usage:{:?} UsageIndex: {}}}}}",
            self.offset, self.element_format, self.element_usage, self.usage_index
        )
    }

    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>() == Some(self)
    }

    pub(crate) const fn byte_size(&self) -> i32 {
        self.element_format.byte_size()
    }
}

impl VertexElementFormat {
    pub(crate) const fn byte_size(self) -> i32 {
        match self {
            Self::Single
            | Self::Color
            | Self::Byte4
            | Self::Short2
            | Self::NormalizedShort2
            | Self::HalfVector2 => 4,
            Self::Vector2 | Self::Short4 | Self::NormalizedShort4 | Self::HalfVector4 => 8,
            Self::Vector3 => 12,
            Self::Vector4 => 16,
        }
    }
}

struct DeclarationState {
    disposed: AtomicBool,
    name: Mutex<String>,
    tag: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
    disposing: EventHandlers<EventArgs>,
}

/// Managed XNA vertex declaration with validated element/stride semantics.
pub struct VertexDeclaration {
    vertex_stride: i32,
    elements: Vec<VertexElement>,
    state: DeclarationState,
}

impl ContentLoadable for VertexDeclaration {}

#[allow(non_snake_case)]
impl VertexDeclaration {
    pub fn new(elements: &[VertexElement]) -> Result<Self> {
        let stride = inferred_stride(elements)?;
        Self::create(stride, elements)
    }

    pub fn from_vertex_stride_and_elements(
        vertexStride: i32,
        elements: &[VertexElement],
    ) -> Result<Self> {
        Self::create(vertexStride, elements)
    }

    fn create(vertex_stride: i32, elements: &[VertexElement]) -> Result<Self> {
        if vertex_stride <= 0 {
            return Err(CnaError::InvalidInput("vertex stride must be positive"));
        }
        validate_elements(elements, vertex_stride)?;
        Ok(Self {
            vertex_stride,
            elements: elements.to_vec(),
            state: DeclarationState {
                disposed: AtomicBool::new(false),
                name: Mutex::new(String::new()),
                tag: Mutex::new(None),
                disposing: EventHandlers::new(),
            },
        })
    }

    #[must_use]
    pub const fn VertexStride(&self) -> i32 {
        self.vertex_stride
    }

    #[must_use]
    pub fn GetVertexElements(&self) -> Vec<VertexElement> {
        self.elements.clone()
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        <Self as GraphicsResource>::Dispose(self, value)
    }

    pub(crate) fn ensure_open(&self) -> Result<()> {
        if self.state.disposed.load(Ordering::Acquire) {
            Err(CnaError::InvalidInput("vertex declaration is disposed"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn detached_copy(&self) -> Result<Self> {
        self.ensure_open()?;
        Self::from_vertex_stride_and_elements(self.vertex_stride, &self.elements)
    }

    pub(crate) fn native_elements(&self) -> Vec<cna_sys::CNA_VertexElement> {
        self.elements
            .iter()
            .map(|element| cna_sys::CNA_VertexElement {
                offset: element.offset,
                format: element.element_format as u32,
                usage: element.element_usage as u32,
                usage_index: element.usage_index,
            })
            .collect()
    }

    pub(crate) fn structurally_equals(&self, other: &Self) -> bool {
        self.vertex_stride == other.vertex_stride && self.elements == other.elements
    }
}

impl GraphicsResource for VertexDeclaration {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        None
    }

    fn IsDisposed(&self) -> bool {
        self.state.disposed.load(Ordering::Acquire)
    }

    fn Name(&self) -> String {
        self.state
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn SetName(&mut self, value: &str) {
        *self
            .state
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value.to_owned();
    }

    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.state
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        *self
            .state
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }

    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.disposing.add(handler)
    }

    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.disposing.remove(registration)
    }

    fn Dispose(&mut self, disposing: bool) -> Result<()> {
        if self.state.disposed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        if disposing && self.state.disposing.emit(self, EventArgs) {
            Err(CnaError::Callback(
                "Rust event-handler panic was contained while disposing VertexDeclaration"
                    .to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for VertexDeclaration {
    fn drop(&mut self) {
        self.state.disposed.store(true, Ordering::Release);
    }
}

fn inferred_stride(elements: &[VertexElement]) -> Result<i32> {
    if elements.is_empty() {
        return Err(CnaError::InvalidInput(
            "vertex declaration must contain at least one element",
        ));
    }
    elements.iter().try_fold(0, |stride, element| {
        if element.Offset() < 0 {
            return Err(CnaError::InvalidInput(
                "vertex element offset must not be negative",
            ));
        }
        element
            .Offset()
            .checked_add(element.byte_size())
            .map(|end| stride.max(end))
            .ok_or(CnaError::InvalidInput(
                "vertex declaration stride overflows",
            ))
    })
}

fn validate_elements(elements: &[VertexElement], vertex_stride: i32) -> Result<()> {
    if elements.is_empty() {
        return Err(CnaError::InvalidInput(
            "vertex declaration must contain at least one element",
        ));
    }
    for element in elements {
        if element.Offset() < 0 || element.UsageIndex() < 0 {
            return Err(CnaError::InvalidInput(
                "vertex element offset and usage index must not be negative",
            ));
        }
        let end = element
            .Offset()
            .checked_add(element.byte_size())
            .ok_or(CnaError::InvalidInput("vertex element range overflows"))?;
        if end > vertex_stride {
            return Err(CnaError::InvalidInput(
                "vertex element extends beyond the declared stride",
            ));
        }
    }
    Ok(())
}

/// Rust projection of XNA's vertex-type declaration property.
pub trait IVertexType {
    fn VertexDeclaration(&self) -> &VertexDeclaration;
}

fn static_declaration(
    slot: &'static OnceLock<VertexDeclaration>,
    stride: i32,
    elements: &[VertexElement],
) -> &'static VertexDeclaration {
    slot.get_or_init(|| {
        VertexDeclaration::from_vertex_stride_and_elements(stride, elements)
            .expect("built-in vertex declaration is valid")
    })
}

macro_rules! vertex_value_behavior {
    ($type:ty, $format:expr) => {
        impl $type {
            #[must_use]
            pub const fn GetHashCode(&self) -> i32 {
                0
            }

            #[must_use]
            pub fn ToString(&self) -> String {
                $format(self)
            }

            #[must_use]
            pub fn Equals(&self, obj: &dyn Any) -> bool {
                obj.downcast_ref::<Self>() == Some(self)
            }
        }
    };
}

/// Built-in 16-byte position/color vertex.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct VertexPositionColor {
    pub Position: Vector3,
    pub Color: Color,
}

#[allow(non_snake_case)]
impl VertexPositionColor {
    #[must_use]
    pub const fn new(position: Vector3, color: Color) -> Self {
        Self {
            Position: position,
            Color: color,
        }
    }

    #[must_use]
    pub fn VertexDeclaration() -> &'static VertexDeclaration {
        static SLOT: OnceLock<VertexDeclaration> = OnceLock::new();
        static_declaration(
            &SLOT,
            16,
            &[
                VertexElement::new(
                    0,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Position,
                    0,
                ),
                VertexElement::new(12, VertexElementFormat::Color, VertexElementUsage::Color, 0),
            ],
        )
    }
}

impl IVertexType for VertexPositionColor {
    fn VertexDeclaration(&self) -> &VertexDeclaration {
        Self::VertexDeclaration()
    }
}

vertex_value_behavior!(VertexPositionColor, |value: &VertexPositionColor| {
    format!(
        "{{Position:{} Color:{}}}",
        value.Position.ToString(),
        value.Color.ToString()
    )
});

/// Built-in 24-byte position/color/texture vertex.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct VertexPositionColorTexture {
    pub Position: Vector3,
    pub Color: Color,
    pub TextureCoordinate: Vector2,
}

#[allow(non_snake_case)]
impl VertexPositionColorTexture {
    #[must_use]
    pub const fn new(position: Vector3, color: Color, textureCoordinate: Vector2) -> Self {
        Self {
            Position: position,
            Color: color,
            TextureCoordinate: textureCoordinate,
        }
    }

    #[must_use]
    pub fn VertexDeclaration() -> &'static VertexDeclaration {
        static SLOT: OnceLock<VertexDeclaration> = OnceLock::new();
        static_declaration(
            &SLOT,
            24,
            &[
                VertexElement::new(
                    0,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Position,
                    0,
                ),
                VertexElement::new(12, VertexElementFormat::Color, VertexElementUsage::Color, 0),
                VertexElement::new(
                    16,
                    VertexElementFormat::Vector2,
                    VertexElementUsage::TextureCoordinate,
                    0,
                ),
            ],
        )
    }
}

impl IVertexType for VertexPositionColorTexture {
    fn VertexDeclaration(&self) -> &VertexDeclaration {
        Self::VertexDeclaration()
    }
}

vertex_value_behavior!(
    VertexPositionColorTexture,
    |value: &VertexPositionColorTexture| {
        format!(
            "{{Position:{} Color:{} TextureCoordinate:{}}}",
            value.Position.ToString(),
            value.Color.ToString(),
            value.TextureCoordinate.ToString()
        )
    }
);

/// Built-in 32-byte position/normal/texture vertex.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct VertexPositionNormalTexture {
    pub Position: Vector3,
    pub Normal: Vector3,
    pub TextureCoordinate: Vector2,
}

#[allow(non_snake_case)]
impl VertexPositionNormalTexture {
    #[must_use]
    pub const fn new(position: Vector3, normal: Vector3, textureCoordinate: Vector2) -> Self {
        Self {
            Position: position,
            Normal: normal,
            TextureCoordinate: textureCoordinate,
        }
    }

    #[must_use]
    pub fn VertexDeclaration() -> &'static VertexDeclaration {
        static SLOT: OnceLock<VertexDeclaration> = OnceLock::new();
        static_declaration(
            &SLOT,
            32,
            &[
                VertexElement::new(
                    0,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Position,
                    0,
                ),
                VertexElement::new(
                    12,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Normal,
                    0,
                ),
                VertexElement::new(
                    24,
                    VertexElementFormat::Vector2,
                    VertexElementUsage::TextureCoordinate,
                    0,
                ),
            ],
        )
    }
}

impl IVertexType for VertexPositionNormalTexture {
    fn VertexDeclaration(&self) -> &VertexDeclaration {
        Self::VertexDeclaration()
    }
}

vertex_value_behavior!(
    VertexPositionNormalTexture,
    |value: &VertexPositionNormalTexture| {
        format!(
            "{{Position:{} Normal:{} TextureCoordinate:{}}}",
            value.Position.ToString(),
            value.Normal.ToString(),
            value.TextureCoordinate.ToString()
        )
    }
);

/// Built-in 20-byte position/texture vertex.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct VertexPositionTexture {
    pub Position: Vector3,
    pub TextureCoordinate: Vector2,
}

#[allow(non_snake_case)]
impl VertexPositionTexture {
    #[must_use]
    pub const fn new(position: Vector3, textureCoordinate: Vector2) -> Self {
        Self {
            Position: position,
            TextureCoordinate: textureCoordinate,
        }
    }

    #[must_use]
    pub fn VertexDeclaration() -> &'static VertexDeclaration {
        static SLOT: OnceLock<VertexDeclaration> = OnceLock::new();
        static_declaration(
            &SLOT,
            20,
            &[
                VertexElement::new(
                    0,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Position,
                    0,
                ),
                VertexElement::new(
                    12,
                    VertexElementFormat::Vector2,
                    VertexElementUsage::TextureCoordinate,
                    0,
                ),
            ],
        )
    }
}

impl IVertexType for VertexPositionTexture {
    fn VertexDeclaration(&self) -> &VertexDeclaration {
        Self::VertexDeclaration()
    }
}

vertex_value_behavior!(VertexPositionTexture, |value: &VertexPositionTexture| {
    format!(
        "{{Position:{} TextureCoordinate:{}}}",
        value.Position.ToString(),
        value.TextureCoordinate.ToString()
    )
});

#[cfg(test)]
mod tests {
    use core::mem::{align_of, size_of};

    use super::*;

    fn offset<T, F>(value: &T, field: &F) -> usize {
        (field as *const F as usize) - (value as *const T as usize)
    }

    #[test]
    fn built_in_vertex_memory_layouts_match_the_c_abi() {
        let position_color = VertexPositionColor::default();
        assert_eq!(
            (
                size_of::<VertexPositionColor>(),
                align_of::<VertexPositionColor>()
            ),
            (16, 4)
        );
        assert_eq!(offset(&position_color, &position_color.Position), 0);
        assert_eq!(offset(&position_color, &position_color.Color), 12);

        let color_texture = VertexPositionColorTexture::default();
        assert_eq!(
            (
                size_of::<VertexPositionColorTexture>(),
                align_of::<VertexPositionColorTexture>()
            ),
            (24, 4)
        );
        assert_eq!(offset(&color_texture, &color_texture.Position), 0);
        assert_eq!(offset(&color_texture, &color_texture.Color), 12);
        assert_eq!(offset(&color_texture, &color_texture.TextureCoordinate), 16);

        let normal_texture = VertexPositionNormalTexture::default();
        assert_eq!(
            (
                size_of::<VertexPositionNormalTexture>(),
                align_of::<VertexPositionNormalTexture>()
            ),
            (32, 4)
        );
        assert_eq!(offset(&normal_texture, &normal_texture.Position), 0);
        assert_eq!(offset(&normal_texture, &normal_texture.Normal), 12);
        assert_eq!(
            offset(&normal_texture, &normal_texture.TextureCoordinate),
            24
        );

        let texture = VertexPositionTexture::default();
        assert_eq!(
            (
                size_of::<VertexPositionTexture>(),
                align_of::<VertexPositionTexture>()
            ),
            (20, 4)
        );
        assert_eq!(offset(&texture, &texture.Position), 0);
        assert_eq!(offset(&texture, &texture.TextureCoordinate), 12);
    }

    #[test]
    fn built_in_declarations_have_stable_identity_and_expected_strides() {
        assert!(core::ptr::eq(
            VertexPositionColor::VertexDeclaration(),
            VertexPositionColor::VertexDeclaration()
        ));
        assert_eq!(VertexPositionColor::VertexDeclaration().VertexStride(), 16);
        assert_eq!(
            VertexPositionColorTexture::VertexDeclaration().VertexStride(),
            24
        );
        assert_eq!(
            VertexPositionNormalTexture::VertexDeclaration().VertexStride(),
            32
        );
        assert_eq!(
            VertexPositionTexture::VertexDeclaration().VertexStride(),
            20
        );
    }
}
