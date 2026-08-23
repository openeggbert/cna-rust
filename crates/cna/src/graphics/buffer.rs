#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::too_many_arguments
)]

use core::any::{Any, TypeId};
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};

use super::resource::{EventHandlers, ResourceKind, ResourceState};
use super::vertex::{
    VertexPositionColor, VertexPositionColorTexture, VertexPositionNormalTexture,
    VertexPositionTexture,
};
use super::{
    BufferUsage, GraphicsDevice, GraphicsResource, IndexElementSize, SetDataOptions,
    VertexDeclaration,
};

/// Safe extension bound for typed vertex transfers.
///
/// Implementations explicitly encode every field, so Rust padding is never
/// exposed as initialized memory to CNA.
pub trait VertexData: Copy + Send + Sync + 'static {
    fn vertex_declaration() -> &'static VertexDeclaration;
    fn write_bytes(&self, destination: &mut Vec<u8>);
    fn read_bytes(source: &[u8]) -> Result<Self>;
}

/// Safe extension bound for XNA's 16-bit and 32-bit index transfers.
pub trait IndexData: Copy + Send + Sync + 'static {
    const ELEMENT_SIZE: IndexElementSize;
    fn to_bits(self) -> u32;
    fn from_bits(value: u32) -> Self;
}

macro_rules! index_data {
    ($type:ty, $size:expr) => {
        impl IndexData for $type {
            const ELEMENT_SIZE: IndexElementSize = $size;

            fn to_bits(self) -> u32 {
                self as u32
            }

            fn from_bits(value: u32) -> Self {
                value as Self
            }
        }
    };
}

index_data!(u16, IndexElementSize::SixteenBits);
index_data!(i16, IndexElementSize::SixteenBits);
index_data!(u32, IndexElementSize::ThirtyTwoBits);
index_data!(i32, IndexElementSize::ThirtyTwoBits);

fn put_f32(destination: &mut Vec<u8>, value: f32) {
    destination.extend_from_slice(&value.to_ne_bytes());
}

fn take_f32(source: &[u8], offset: usize) -> Result<f32> {
    let bytes = source
        .get(offset..offset + 4)
        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?;
    Ok(f32::from_ne_bytes(
        bytes
            .try_into()
            .expect("a four-byte range always converts to [u8; 4]"),
    ))
}

macro_rules! vertex_data {
    ($type:ty, $write:expr, $read:expr) => {
        impl VertexData for $type {
            fn vertex_declaration() -> &'static VertexDeclaration {
                <$type>::VertexDeclaration()
            }

            fn write_bytes(&self, destination: &mut Vec<u8>) {
                $write(self, destination);
            }

            fn read_bytes(source: &[u8]) -> Result<Self> {
                $read(source)
            }
        }
    };
}

vertex_data!(
    VertexPositionColor,
    |value: &VertexPositionColor, out: &mut Vec<u8>| {
        put_f32(out, value.Position.X);
        put_f32(out, value.Position.Y);
        put_f32(out, value.Position.Z);
        out.extend_from_slice(&[
            value.Color.R(),
            value.Color.G(),
            value.Color.B(),
            value.Color.A(),
        ]);
    },
    |source: &[u8]| {
        Ok(VertexPositionColor::new(
            crate::value::Vector3::from_x_and_y_and_z(
                take_f32(source, 0)?,
                take_f32(source, 4)?,
                take_f32(source, 8)?,
            ),
            crate::value::Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                i32::from(
                    *source
                        .get(12)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(13)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(14)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(15)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
            ),
        ))
    }
);

vertex_data!(
    VertexPositionColorTexture,
    |value: &VertexPositionColorTexture, out: &mut Vec<u8>| {
        put_f32(out, value.Position.X);
        put_f32(out, value.Position.Y);
        put_f32(out, value.Position.Z);
        out.extend_from_slice(&[
            value.Color.R(),
            value.Color.G(),
            value.Color.B(),
            value.Color.A(),
        ]);
        put_f32(out, value.TextureCoordinate.X);
        put_f32(out, value.TextureCoordinate.Y);
    },
    |source: &[u8]| {
        Ok(VertexPositionColorTexture::new(
            crate::value::Vector3::from_x_and_y_and_z(
                take_f32(source, 0)?,
                take_f32(source, 4)?,
                take_f32(source, 8)?,
            ),
            crate::value::Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                i32::from(
                    *source
                        .get(12)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(13)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(14)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
                i32::from(
                    *source
                        .get(15)
                        .ok_or(CnaError::InvalidInput("vertex byte payload is truncated"))?,
                ),
            ),
            crate::value::Vector2::from_x_and_y(take_f32(source, 16)?, take_f32(source, 20)?),
        ))
    }
);

vertex_data!(
    VertexPositionNormalTexture,
    |value: &VertexPositionNormalTexture, out: &mut Vec<u8>| {
        for component in [
            value.Position.X,
            value.Position.Y,
            value.Position.Z,
            value.Normal.X,
            value.Normal.Y,
            value.Normal.Z,
            value.TextureCoordinate.X,
            value.TextureCoordinate.Y,
        ] {
            put_f32(out, component);
        }
    },
    |source: &[u8]| {
        Ok(VertexPositionNormalTexture::new(
            crate::value::Vector3::from_x_and_y_and_z(
                take_f32(source, 0)?,
                take_f32(source, 4)?,
                take_f32(source, 8)?,
            ),
            crate::value::Vector3::from_x_and_y_and_z(
                take_f32(source, 12)?,
                take_f32(source, 16)?,
                take_f32(source, 20)?,
            ),
            crate::value::Vector2::from_x_and_y(take_f32(source, 24)?, take_f32(source, 28)?),
        ))
    }
);

vertex_data!(
    VertexPositionTexture,
    |value: &VertexPositionTexture, out: &mut Vec<u8>| {
        put_f32(out, value.Position.X);
        put_f32(out, value.Position.Y);
        put_f32(out, value.Position.Z);
        put_f32(out, value.TextureCoordinate.X);
        put_f32(out, value.TextureCoordinate.Y);
    },
    |source: &[u8]| {
        Ok(VertexPositionTexture::new(
            crate::value::Vector3::from_x_and_y_and_z(
                take_f32(source, 0)?,
                take_f32(source, 4)?,
                take_f32(source, 8)?,
            ),
            crate::value::Vector2::from_x_and_y(take_f32(source, 12)?, take_f32(source, 16)?),
        ))
    }
);

/// Behavior inherited by XNA `DynamicVertexBuffer` through safe composition.
pub trait VertexBufferBase: GraphicsResource {}

/// Owned CNA vertex buffer with a durable borrowed device identity.
#[derive(Clone)]
pub struct VertexBuffer {
    state: Arc<ResourceState>,
    declaration: Arc<VertexDeclaration>,
    vertex_count: i32,
    usage: BufferUsage,
    dynamic: bool,
}

#[allow(non_snake_case)]
impl VertexBuffer {
    pub fn from_graphics_device_and_vertex_type_and_vertex_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        vertexType: TypeId,
        vertexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        let declaration = declaration_for_type(vertexType)?;
        Self::create(graphicsDevice, declaration, vertexCount, usage, false)
    }

    pub fn new(
        graphicsDevice: &GraphicsDevice,
        vertexDeclaration: &VertexDeclaration,
        vertexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Self::create(graphicsDevice, vertexDeclaration, vertexCount, usage, false)
    }

    fn create(
        graphics_device: &GraphicsDevice,
        declaration: &VertexDeclaration,
        vertex_count: i32,
        usage: BufferUsage,
        dynamic: bool,
    ) -> Result<Self> {
        validate_usage(usage)?;
        if vertex_count < 0 {
            return Err(CnaError::InvalidInput("vertex count must not be negative"));
        }
        declaration.ensure_open()?;
        let native = graphics_device.state.native();
        let mut native_declaration = sys::CNA_INVALID_HANDLE;
        native.create_vertex_declaration(
            declaration.VertexStride(),
            &declaration.native_elements(),
            &mut native_declaration,
        )?;
        let info = sys::CNA_VertexBufferCreateInfo {
            struct_size: size_of::<sys::CNA_VertexBufferCreateInfo>() as u32,
            struct_version: 1,
            vertex_declaration: native_declaration,
            vertex_count,
            buffer_usage: usage.bits(),
            dynamic: if dynamic {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            },
            reserved: [0; 7],
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        let create = native.create_vertex_buffer(graphics_device.handle()?, &info, &mut handle);
        let destroy_declaration = native.destroy_vertex_declaration(native_declaration);
        create?;
        if let Err(error) = destroy_declaration {
            let _ = native.destroy_vertex_buffer(handle);
            return Err(error);
        }
        let mut native_info = sys::CNA_VertexBufferInfo {
            struct_size: size_of::<sys::CNA_VertexBufferInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_VertexBufferInfo::default()
        };
        if let Err(error) = native.vertex_buffer_info(handle, &mut native_info) {
            let _ = native.destroy_vertex_buffer(handle);
            return Err(error);
        }
        if native_info.vertex_count != vertex_count
            || native_info.vertex_stride != declaration.VertexStride()
            || native_info.buffer_usage != usage.bits()
            || native_info.dynamic
                != if dynamic {
                    sys::CNA_TRUE
                } else {
                    sys::CNA_FALSE
                }
        {
            let _ = native.destroy_vertex_buffer(handle);
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned inconsistent vertex-buffer metadata".to_owned(),
            });
        }
        Ok(Self {
            state: ResourceState::new(graphics_device, handle, ResourceKind::VertexBuffer),
            declaration: Arc::new(declaration.detached_copy()?),
            vertex_count,
            usage,
            dynamic,
        })
    }

    #[must_use]
    pub fn VertexDeclaration(&self) -> &VertexDeclaration {
        &self.declaration
    }

    #[must_use]
    pub const fn VertexCount(&self) -> i32 {
        self.vertex_count
    }

    #[must_use]
    pub const fn BufferUsage(&self) -> BufferUsage {
        self.usage
    }

    pub fn SetData<T: VertexData>(&self, data: &[T]) -> Result<()> {
        let count = count_i32(data.len(), "vertex data array is too large")?;
        self.SetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn SetDataWithDataAndStartIndexAndElementCount<T: VertexData>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.set_data(
            0,
            data,
            startIndex,
            elementCount,
            self.declaration.VertexStride(),
            SetDataOptions::None,
        )
    }

    pub fn SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndVertexStride<
        T: VertexData,
    >(
        &self,
        offsetInBytes: i32,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
        vertexStride: i32,
    ) -> Result<()> {
        self.set_data(
            offsetInBytes,
            data,
            startIndex,
            elementCount,
            vertexStride,
            SetDataOptions::None,
        )
    }

    pub fn GetData<T: VertexData>(&self, data: &mut [T]) -> Result<()> {
        let count = count_i32(data.len(), "vertex data array is too large")?;
        self.GetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn GetDataWithDataAndStartIndexAndElementCount<T: VertexData>(
        &self,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.get_data(
            0,
            data,
            startIndex,
            elementCount,
            self.declaration.VertexStride(),
        )
    }

    pub fn GetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndVertexStride<
        T: VertexData,
    >(
        &self,
        offsetInBytes: i32,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
        vertexStride: i32,
    ) -> Result<()> {
        self.get_data(offsetInBytes, data, startIndex, elementCount, vertexStride)
    }

    fn set_data<T: VertexData>(
        &self,
        offset: i32,
        data: &[T],
        start: i32,
        count: i32,
        stride: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        let range = validate_vertex_transfer(
            self,
            offset,
            data.len(),
            start,
            count,
            stride,
            T::vertex_declaration(),
        )?;
        validate_options(options, self.dynamic)?;
        if options != SetDataOptions::None {
            if offset != 0 {
                return Err(CnaError::InvalidInput(
                    "CNA ABI 0.7 cannot combine a vertex-buffer byte offset with Discard or NoOverwrite",
                ));
            }
            let vertex_type = native_vertex_type::<T>().ok_or(CnaError::InvalidInput(
                "Discard and NoOverwrite require a built-in XNA vertex layout in CNA ABI 0.7",
            ))?;
            let transfer = sys::CNA_VertexBufferTransfer {
                struct_size: size_of::<sys::CNA_VertexBufferTransfer>() as u32,
                struct_version: 1,
                vertex_type,
                options: options.bits(),
                start_index: range.start as u64,
                element_count: range.len() as u64,
            };
            // SAFETY: TypeId was matched against the exact repr(C) built-in T layout.
            return unsafe {
                self.state.device().state.native().set_typed_vertex_data(
                    self.state.require_handle()?,
                    &transfer,
                    data,
                )
            };
        }
        let bytes = encode_vertices(&data[range]);
        self.state.device().state.native().set_raw_vertex_data(
            self.state.require_handle()?,
            (offset != 0).then_some(offset as u64),
            &bytes,
            count as u64,
            stride as u32,
        )
    }

    fn get_data<T: VertexData>(
        &self,
        offset: i32,
        data: &mut [T],
        start: i32,
        count: i32,
        stride: i32,
    ) -> Result<()> {
        let range = validate_vertex_transfer(
            self,
            offset,
            data.len(),
            start,
            count,
            stride,
            T::vertex_declaration(),
        )?;
        let byte_count = range
            .len()
            .checked_mul(stride as usize)
            .ok_or(CnaError::InvalidInput("vertex transfer size overflows"))?;
        let mut bytes = vec![0; byte_count];
        self.state.device().state.native().get_raw_vertex_data(
            self.state.require_handle()?,
            offset as u64,
            &mut bytes,
            count as u64,
            stride as u32,
        )?;
        for (slot, chunk) in data[range]
            .iter_mut()
            .zip(bytes.chunks_exact(stride as usize))
        {
            *slot = T::read_bytes(chunk)?;
        }
        Ok(())
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }

    pub(crate) fn is_same_device(&self, device: &GraphicsDevice) -> bool {
        self.state.device().is_same_device(device)
    }
}

impl VertexBufferBase for VertexBuffer {}

impl Drop for VertexBuffer {
    fn drop(&mut self) {}
}

impl GraphicsResource for VertexBuffer {
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
        Self::Dispose(self, value)
    }
}

/// Owned dynamic vertex buffer. CNA represents it with the same owned handle kind.
pub struct DynamicVertexBuffer {
    inner: VertexBuffer,
    content_lost: EventHandlers<EventArgs>,
}

#[allow(non_snake_case)]
impl DynamicVertexBuffer {
    pub fn from_graphics_device_and_vertex_type_and_vertex_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        vertexType: TypeId,
        vertexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        let declaration = declaration_for_type(vertexType)?;
        Self::create(graphicsDevice, declaration, vertexCount, usage)
    }

    pub fn new(
        graphicsDevice: &GraphicsDevice,
        vertexDeclaration: &VertexDeclaration,
        vertexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Self::create(graphicsDevice, vertexDeclaration, vertexCount, usage)
    }

    fn create(
        graphics_device: &GraphicsDevice,
        declaration: &VertexDeclaration,
        count: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Ok(Self {
            inner: VertexBuffer::create(graphics_device, declaration, count, usage, true)?,
            content_lost: EventHandlers::new(),
        })
    }

    pub fn SetData<T: VertexData>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        self.inner.set_data(
            0,
            data,
            startIndex,
            elementCount,
            self.inner.VertexDeclaration().VertexStride(),
            options,
        )
    }

    pub fn SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndVertexStrideAndOptions<
        T: VertexData,
    >(
        &self,
        offsetInBytes: i32,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
        vertexStride: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        self.inner.set_data(
            offsetInBytes,
            data,
            startIndex,
            elementCount,
            vertexStride,
            options,
        )
    }

    pub fn IsContentLost(&self) -> Result<bool> {
        let mut info = sys::CNA_VertexBufferInfo {
            struct_size: size_of::<sys::CNA_VertexBufferInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_VertexBufferInfo::default()
        };
        self.inner
            .state
            .device()
            .state
            .native()
            .vertex_buffer_info(self.inner.handle()?, &mut info)?;
        Ok(info.is_content_lost != sys::CNA_FALSE)
    }

    pub fn AddContentLostHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.content_lost.add(handler)
    }
    pub fn RemoveContentLostHandler(&self, registration: u64) -> bool {
        self.content_lost.remove(registration)
    }
}

impl Deref for DynamicVertexBuffer {
    type Target = VertexBuffer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DynamicVertexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl VertexBufferBase for DynamicVertexBuffer {}
impl Drop for DynamicVertexBuffer {
    fn drop(&mut self) {}
}
impl GraphicsResource for DynamicVertexBuffer {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        self.inner.GraphicsDevice()
    }
    fn IsDisposed(&self) -> bool {
        self.inner.IsDisposed()
    }
    fn Name(&self) -> String {
        self.inner.Name()
    }
    fn SetName(&mut self, value: &str) {
        self.inner.SetName(value);
    }
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.inner.Tag()
    }
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.inner.SetTag(value);
    }
    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.inner.AddDisposingHandler(handler)
    }
    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.inner.RemoveDisposingHandler(registration)
    }
    fn Dispose(&mut self, value: bool) -> Result<()> {
        self.inner.Dispose(value)
    }
}

/// Safe retained vertex-buffer binding value.
#[derive(Clone)]
pub struct VertexBufferBinding {
    buffer: VertexBuffer,
    vertex_offset: i32,
    instance_frequency: i32,
}

#[allow(non_snake_case)]
impl VertexBufferBinding {
    pub fn from_vertex_buffer_and_vertex_offset_and_instance_frequency(
        vertexBuffer: &VertexBuffer,
        vertexOffset: i32,
        instanceFrequency: i32,
    ) -> Result<Self> {
        if vertexOffset < 0 || instanceFrequency < 0 {
            return Err(CnaError::InvalidInput(
                "vertex offset and instance frequency must not be negative",
            ));
        }
        if vertexOffset > vertexBuffer.VertexCount() {
            return Err(CnaError::InvalidInput(
                "vertex offset exceeds the buffer capacity",
            ));
        }
        vertexBuffer.handle()?;
        Ok(Self {
            buffer: vertexBuffer.clone(),
            vertex_offset: vertexOffset,
            instance_frequency: instanceFrequency,
        })
    }

    pub fn from_vertex_buffer_and_vertex_offset(
        vertexBuffer: &VertexBuffer,
        vertexOffset: i32,
    ) -> Result<Self> {
        Self::from_vertex_buffer_and_vertex_offset_and_instance_frequency(
            vertexBuffer,
            vertexOffset,
            0,
        )
    }

    pub fn new(vertexBuffer: &VertexBuffer) -> Result<Self> {
        Self::from_vertex_buffer_and_vertex_offset_and_instance_frequency(vertexBuffer, 0, 0)
    }

    #[must_use]
    pub const fn InstanceFrequency(&self) -> i32 {
        self.instance_frequency
    }
    #[must_use]
    pub const fn VertexOffset(&self) -> i32 {
        self.vertex_offset
    }
    #[must_use]
    pub fn VertexBuffer(&self) -> &VertexBuffer {
        &self.buffer
    }

    pub(crate) fn to_native(&self) -> Result<sys::CNA_VertexBufferBinding> {
        let mut binding = sys::CNA_VertexBufferBinding::default();
        self.buffer
            .state
            .device()
            .state
            .native()
            .initialize_vertex_buffer_binding(
                self.buffer.handle()?,
                self.vertex_offset,
                self.instance_frequency,
                &mut binding,
            )?;
        Ok(binding)
    }
}

impl TryFrom<&VertexBuffer> for VertexBufferBinding {
    type Error = CnaError;
    fn try_from(value: &VertexBuffer) -> Result<Self> {
        Self::new(value)
    }
}

impl PartialEq for VertexBufferBinding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.buffer.state, &other.buffer.state)
            && self.vertex_offset == other.vertex_offset
            && self.instance_frequency == other.instance_frequency
    }
}

/// Behavior inherited by XNA `DynamicIndexBuffer` through safe composition.
pub trait IndexBufferBase: GraphicsResource {}

/// Owned CNA index buffer.
#[derive(Clone)]
pub struct IndexBuffer {
    state: Arc<ResourceState>,
    index_count: i32,
    element_size: IndexElementSize,
    usage: BufferUsage,
    dynamic: bool,
}

#[allow(non_snake_case)]
impl IndexBuffer {
    pub fn from_graphics_device_and_index_type_and_index_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        indexType: TypeId,
        indexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        let size = if indexType == TypeId::of::<u16>() || indexType == TypeId::of::<i16>() {
            IndexElementSize::SixteenBits
        } else if indexType == TypeId::of::<u32>() || indexType == TypeId::of::<i32>() {
            IndexElementSize::ThirtyTwoBits
        } else {
            return Err(CnaError::InvalidInput(
                "index TypeId must identify a 16-bit or 32-bit integer",
            ));
        };
        Self::create(graphicsDevice, size, indexCount, usage, false)
    }

    pub fn new(
        graphicsDevice: &GraphicsDevice,
        indexElementSize: IndexElementSize,
        indexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Self::create(graphicsDevice, indexElementSize, indexCount, usage, false)
    }

    fn create(
        device: &GraphicsDevice,
        element_size: IndexElementSize,
        count: i32,
        usage: BufferUsage,
        dynamic: bool,
    ) -> Result<Self> {
        validate_usage(usage)?;
        if count < 0 {
            return Err(CnaError::InvalidInput("index count must not be negative"));
        }
        let info = sys::CNA_IndexBufferCreateInfo {
            struct_size: size_of::<sys::CNA_IndexBufferCreateInfo>() as u32,
            struct_version: 1,
            index_count: count,
            index_element_size: element_size as u32,
            buffer_usage: usage.bits(),
            dynamic: if dynamic {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            },
            reserved: [0; 3],
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        device
            .state
            .native()
            .create_index_buffer(device.handle()?, &info, &mut handle)?;
        let mut native_info = sys::CNA_IndexBufferInfo {
            struct_size: size_of::<sys::CNA_IndexBufferInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_IndexBufferInfo::default()
        };
        if let Err(error) = device
            .state
            .native()
            .index_buffer_info(handle, &mut native_info)
        {
            let _ = device.state.native().destroy_index_buffer(handle);
            return Err(error);
        }
        if native_info.index_count != count
            || native_info.index_element_size != element_size as u32
            || native_info.buffer_usage != usage.bits()
            || native_info.dynamic
                != if dynamic {
                    sys::CNA_TRUE
                } else {
                    sys::CNA_FALSE
                }
        {
            let _ = device.state.native().destroy_index_buffer(handle);
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned inconsistent index-buffer metadata".to_owned(),
            });
        }
        Ok(Self {
            state: ResourceState::new(device, handle, ResourceKind::IndexBuffer),
            index_count: count,
            element_size,
            usage,
            dynamic,
        })
    }

    #[must_use]
    pub const fn IndexCount(&self) -> i32 {
        self.index_count
    }
    #[must_use]
    pub const fn IndexElementSize(&self) -> IndexElementSize {
        self.element_size
    }
    #[must_use]
    pub const fn BufferUsage(&self) -> BufferUsage {
        self.usage
    }

    pub fn SetData<T: IndexData>(&self, data: &[T]) -> Result<()> {
        let count = count_i32(data.len(), "index data array is too large")?;
        self.SetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn SetDataWithDataAndStartIndexAndElementCount<T: IndexData>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.set_data(0, data, startIndex, elementCount, SetDataOptions::None)
    }

    pub fn SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCount<T: IndexData>(
        &self,
        offsetInBytes: i32,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.set_data(
            offsetInBytes,
            data,
            startIndex,
            elementCount,
            SetDataOptions::None,
        )
    }

    pub fn GetData<T: IndexData>(&self, data: &mut [T]) -> Result<()> {
        let count = count_i32(data.len(), "index data array is too large")?;
        self.GetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn GetDataWithDataAndStartIndexAndElementCount<T: IndexData>(
        &self,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.get_data(0, data, startIndex, elementCount)
    }

    pub fn GetDataWithOffsetInBytesAndDataAndStartIndexAndElementCount<T: IndexData>(
        &self,
        offsetInBytes: i32,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.get_data(offsetInBytes, data, startIndex, elementCount)
    }

    fn set_data<T: IndexData>(
        &self,
        offset: i32,
        data: &[T],
        start: i32,
        count: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        let range =
            validate_index_transfer(self, offset, data.len(), start, count, T::ELEMENT_SIZE)?;
        let range_len = range.len();
        validate_options(options, self.dynamic)?;
        if offset != 0 && options != SetDataOptions::None {
            return Err(CnaError::InvalidInput(
                "CNA ABI 0.7 cannot combine an index-buffer byte offset with Discard or NoOverwrite",
            ));
        }
        let transfer = sys::CNA_IndexBufferTransfer {
            struct_size: size_of::<sys::CNA_IndexBufferTransfer>() as u32,
            struct_version: 1,
            index_element_size: self.element_size as u32,
            options: options.bits(),
            start_index: 0,
            element_count: range_len as u64,
        };
        match self.element_size {
            IndexElementSize::SixteenBits => {
                let values = data[range]
                    .iter()
                    .map(|value| value.to_bits() as u16)
                    .collect::<Vec<_>>();
                self.state.device().state.native().set_index_data(
                    self.state.require_handle()?,
                    (offset != 0).then_some(offset as u64),
                    &transfer,
                    &values,
                )
            }
            IndexElementSize::ThirtyTwoBits => {
                let values = data[range]
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>();
                self.state.device().state.native().set_index_data(
                    self.state.require_handle()?,
                    (offset != 0).then_some(offset as u64),
                    &transfer,
                    &values,
                )
            }
        }
    }

    fn get_data<T: IndexData>(
        &self,
        offset: i32,
        data: &mut [T],
        start: i32,
        count: i32,
    ) -> Result<()> {
        let range =
            validate_index_transfer(self, offset, data.len(), start, count, T::ELEMENT_SIZE)?;
        let range_len = range.len();
        if offset != 0 {
            return Err(CnaError::InvalidInput(
                "CNA ABI 0.7 index readback cannot represent a nonzero buffer offset",
            ));
        }
        let transfer = sys::CNA_IndexBufferTransfer {
            struct_size: size_of::<sys::CNA_IndexBufferTransfer>() as u32,
            struct_version: 1,
            index_element_size: self.element_size as u32,
            options: sys::CNA_SET_DATA_NONE,
            start_index: 0,
            element_count: range.len() as u64,
        };
        let mut required = 0;
        match self.element_size {
            IndexElementSize::SixteenBits => {
                let mut values = vec![0_u16; range_len];
                self.state.device().state.native().get_index_data(
                    self.state.require_handle()?,
                    &transfer,
                    &mut values,
                    &mut required,
                )?;
                for (destination, value) in data[range.clone()].iter_mut().zip(values) {
                    *destination = T::from_bits(u32::from(value));
                }
            }
            IndexElementSize::ThirtyTwoBits => {
                let mut values = vec![0_u32; range_len];
                self.state.device().state.native().get_index_data(
                    self.state.require_handle()?,
                    &transfer,
                    &mut values,
                    &mut required,
                )?;
                for (destination, value) in data[range].iter_mut().zip(values) {
                    *destination = T::from_bits(value);
                }
            }
        }
        if required != range_len as u64 {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an inconsistent index readback count".to_owned(),
            });
        }
        Ok(())
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }
    pub(crate) fn is_same_device(&self, device: &GraphicsDevice) -> bool {
        self.state.device().is_same_device(device)
    }
}

impl IndexBufferBase for IndexBuffer {}
impl Drop for IndexBuffer {
    fn drop(&mut self) {}
}
impl GraphicsResource for IndexBuffer {
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
        Self::Dispose(self, value)
    }
}

/// Owned dynamic index buffer.
pub struct DynamicIndexBuffer {
    inner: IndexBuffer,
    content_lost: EventHandlers<EventArgs>,
}

#[allow(non_snake_case)]
impl DynamicIndexBuffer {
    pub fn from_graphics_device_and_index_type_and_index_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        indexType: TypeId,
        indexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        let size = if indexType == TypeId::of::<u16>() || indexType == TypeId::of::<i16>() {
            IndexElementSize::SixteenBits
        } else if indexType == TypeId::of::<u32>() || indexType == TypeId::of::<i32>() {
            IndexElementSize::ThirtyTwoBits
        } else {
            return Err(CnaError::InvalidInput(
                "index TypeId must identify a 16-bit or 32-bit integer",
            ));
        };
        Self::create(graphicsDevice, size, indexCount, usage)
    }

    pub fn new(
        graphicsDevice: &GraphicsDevice,
        indexElementSize: IndexElementSize,
        indexCount: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Self::create(graphicsDevice, indexElementSize, indexCount, usage)
    }

    fn create(
        device: &GraphicsDevice,
        size: IndexElementSize,
        count: i32,
        usage: BufferUsage,
    ) -> Result<Self> {
        Ok(Self {
            inner: IndexBuffer::create(device, size, count, usage, true)?,
            content_lost: EventHandlers::new(),
        })
    }

    pub fn SetData<T: IndexData>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        self.inner
            .set_data(0, data, startIndex, elementCount, options)
    }

    pub fn SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndOptions<T: IndexData>(
        &self,
        offsetInBytes: i32,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
        options: SetDataOptions,
    ) -> Result<()> {
        self.inner
            .set_data(offsetInBytes, data, startIndex, elementCount, options)
    }

    pub fn IsContentLost(&self) -> Result<bool> {
        let mut info = sys::CNA_IndexBufferInfo {
            struct_size: size_of::<sys::CNA_IndexBufferInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_IndexBufferInfo::default()
        };
        self.inner
            .state
            .device()
            .state
            .native()
            .index_buffer_info(self.inner.handle()?, &mut info)?;
        Ok(info.is_content_lost != sys::CNA_FALSE)
    }

    pub fn AddContentLostHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.content_lost.add(handler)
    }
    pub fn RemoveContentLostHandler(&self, registration: u64) -> bool {
        self.content_lost.remove(registration)
    }
}

impl Deref for DynamicIndexBuffer {
    type Target = IndexBuffer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for DynamicIndexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl IndexBufferBase for DynamicIndexBuffer {}
impl Drop for DynamicIndexBuffer {
    fn drop(&mut self) {}
}
impl GraphicsResource for DynamicIndexBuffer {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        self.inner.GraphicsDevice()
    }
    fn IsDisposed(&self) -> bool {
        self.inner.IsDisposed()
    }
    fn Name(&self) -> String {
        self.inner.Name()
    }
    fn SetName(&mut self, value: &str) {
        self.inner.SetName(value);
    }
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.inner.Tag()
    }
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.inner.SetTag(value);
    }
    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.inner.AddDisposingHandler(handler)
    }
    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.inner.RemoveDisposingHandler(registration)
    }
    fn Dispose(&mut self, value: bool) -> Result<()> {
        self.inner.Dispose(value)
    }
}

fn declaration_for_type(vertex_type: TypeId) -> Result<&'static VertexDeclaration> {
    if vertex_type == TypeId::of::<VertexPositionColor>() {
        Ok(VertexPositionColor::VertexDeclaration())
    } else if vertex_type == TypeId::of::<VertexPositionColorTexture>() {
        Ok(VertexPositionColorTexture::VertexDeclaration())
    } else if vertex_type == TypeId::of::<VertexPositionNormalTexture>() {
        Ok(VertexPositionNormalTexture::VertexDeclaration())
    } else if vertex_type == TypeId::of::<VertexPositionTexture>() {
        Ok(VertexPositionTexture::VertexDeclaration())
    } else {
        Err(CnaError::InvalidInput("vertex TypeId has no registered XNA vertex declaration; use the VertexDeclaration constructor overload"))
    }
}

fn native_vertex_type<T: VertexData>() -> Option<sys::CNA_VertexType> {
    let type_id = TypeId::of::<T>();
    if type_id == TypeId::of::<VertexPositionColor>() {
        Some(sys::CNA_VERTEX_TYPE_POSITION_COLOR)
    } else if type_id == TypeId::of::<VertexPositionColorTexture>() {
        Some(sys::CNA_VERTEX_TYPE_POSITION_COLOR_TEXTURE)
    } else if type_id == TypeId::of::<VertexPositionNormalTexture>() {
        Some(sys::CNA_VERTEX_TYPE_POSITION_NORMAL_TEXTURE)
    } else if type_id == TypeId::of::<VertexPositionTexture>() {
        Some(sys::CNA_VERTEX_TYPE_POSITION_TEXTURE)
    } else {
        None
    }
}

fn validate_usage(usage: BufferUsage) -> Result<()> {
    if usage.bits() & !sys::CNA_BUFFER_USAGE_WRITE_ONLY == 0 {
        Ok(())
    } else {
        Err(CnaError::InvalidInput(
            "buffer usage contains unknown flags",
        ))
    }
}

fn validate_options(options: SetDataOptions, dynamic: bool) -> Result<()> {
    let bits = options.bits();
    if bits > sys::CNA_SET_DATA_NO_OVERWRITE
        || bits == (sys::CNA_SET_DATA_DISCARD | sys::CNA_SET_DATA_NO_OVERWRITE)
    {
        return Err(CnaError::InvalidInput(
            "set-data options contain an invalid combination",
        ));
    }
    if !dynamic && options != SetDataOptions::None {
        return Err(CnaError::InvalidInput(
            "Discard and NoOverwrite require a dynamic buffer",
        ));
    }
    Ok(())
}

fn count_i32(length: usize, message: &'static str) -> Result<i32> {
    i32::try_from(length).map_err(|_| CnaError::InvalidInput(message))
}

fn checked_range(length: usize, start: i32, count: i32) -> Result<core::ops::Range<usize>> {
    let start = usize::try_from(start)
        .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
    let count = usize::try_from(count)
        .map_err(|_| CnaError::InvalidInput("element count must not be negative"))?;
    let end = start
        .checked_add(count)
        .ok_or(CnaError::InvalidInput("array window overflows"))?;
    if end > length {
        Err(CnaError::InvalidInput(
            "array window exceeds the supplied slice",
        ))
    } else {
        Ok(start..end)
    }
}

fn validate_vertex_transfer(
    buffer: &VertexBuffer,
    offset: i32,
    length: usize,
    start: i32,
    count: i32,
    stride: i32,
    declaration: &VertexDeclaration,
) -> Result<core::ops::Range<usize>> {
    if offset < 0 {
        return Err(CnaError::InvalidInput(
            "buffer byte offset must not be negative",
        ));
    }
    if stride <= 0
        || stride != buffer.declaration.VertexStride()
        || stride != declaration.VertexStride()
        || !buffer.declaration.structurally_equals(declaration)
    {
        return Err(CnaError::InvalidInput(
            "vertex data declaration and stride must match the buffer declaration",
        ));
    }
    let range = checked_range(length, start, count)?;
    let end = (offset as usize)
        .checked_add(
            range
                .len()
                .checked_mul(stride as usize)
                .ok_or(CnaError::InvalidInput("vertex transfer size overflows"))?,
        )
        .ok_or(CnaError::InvalidInput("vertex transfer range overflows"))?;
    let capacity = (buffer.vertex_count as usize)
        .checked_mul(buffer.declaration.VertexStride() as usize)
        .ok_or(CnaError::InvalidInput("vertex buffer capacity overflows"))?;
    if offset as usize % stride as usize != 0 || end > capacity {
        return Err(CnaError::InvalidInput(
            "vertex transfer exceeds or misaligns the buffer capacity",
        ));
    }
    Ok(range)
}

pub(crate) fn encode_vertices<T: VertexData>(data: &[T]) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(data.len() * T::vertex_declaration().VertexStride() as usize);
    for value in data {
        value.write_bytes(&mut bytes);
    }
    bytes
}

fn validate_index_transfer(
    buffer: &IndexBuffer,
    offset: i32,
    length: usize,
    start: i32,
    count: i32,
    element_size: IndexElementSize,
) -> Result<core::ops::Range<usize>> {
    if element_size != buffer.element_size {
        return Err(CnaError::InvalidInput(
            "index element type does not match the buffer",
        ));
    }
    if offset < 0 {
        return Err(CnaError::InvalidInput(
            "buffer byte offset must not be negative",
        ));
    }
    let range = checked_range(length, start, count)?;
    let width = match element_size {
        IndexElementSize::SixteenBits => 2,
        IndexElementSize::ThirtyTwoBits => 4,
    };
    let end = (offset as usize)
        .checked_add(
            range
                .len()
                .checked_mul(width)
                .ok_or(CnaError::InvalidInput("index transfer size overflows"))?,
        )
        .ok_or(CnaError::InvalidInput("index transfer range overflows"))?;
    let capacity = (buffer.index_count as usize)
        .checked_mul(width)
        .ok_or(CnaError::InvalidInput("index buffer capacity overflows"))?;
    if offset as usize % width != 0 || end > capacity {
        return Err(CnaError::InvalidInput(
            "index transfer exceeds or misaligns the buffer capacity",
        ));
    }
    Ok(range)
}
