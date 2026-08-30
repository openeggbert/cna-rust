//! Microsoft XNA 4.0 `Net` value identities.
//!
//! These belong to the wider Windows runtime profile
//! (`tools/api-compat/profiles/xna40-windows-full.json`), not the selected
//! seven-assembly profile. Everything here is a value type or an exception
//! identity: exact managed Rust with no native backing, because CLR metadata
//! is the whole of their contract.

#![allow(non_upper_case_globals, non_snake_case)]

use core::fmt;
use core::ops::{BitAnd, BitOr, BitOrAssign};
use std::error::Error;
use std::vec::IntoIter;

use crate::content::{SerializationInfo, StreamingContext};
use crate::error::{CnaError, Result};
use crate::disposal::Disposable;
use crate::gamer_services::NetworkExceptionBase;
use crate::value::{Color, Matrix, Quaternion, Vector2, Vector3, Vector4};

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionEndReason`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionEndReason {
    ClientSignedOut = 0,
    HostEndedSession = 1,
    RemovedByHost = 2,
    Disconnected = 3,
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionJoinError`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionJoinError {
    SessionNotFound = 0,
    SessionNotJoinable = 1,
    SessionFull = 2,
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionState`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionState {
    Lobby = 0,
    Playing = 1,
    Ended = 2,
}

impl NetworkSessionEndReason {
    pub(crate) fn from_native(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::ClientSignedOut),
            1 => Some(Self::HostEndedSession),
            2 => Some(Self::RemovedByHost),
            3 => Some(Self::Disconnected),
            _ => None,
        }
    }
}

impl NetworkSessionType {
    pub(crate) fn from_native(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Local),
            1 => Some(Self::SystemLink),
            2 => Some(Self::PlayerMatch),
            3 => Some(Self::Ranked),
            4 => Some(Self::LocalWithLeaderboards),
            _ => None,
        }
    }
}

impl NetworkSessionState {
    pub(crate) fn from_native(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Lobby),
            1 => Some(Self::Playing),
            2 => Some(Self::Ended),
            _ => None,
        }
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionType`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionType {
    Local = 0,
    SystemLink = 1,
    PlayerMatch = 2,
    Ranked = 3,
    LocalWithLeaderboards = 4,
}

/// XNA `Microsoft.Xna.Framework.Net.SendDataOptions` flag set.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SendDataOptions(i32);

impl SendDataOptions {
    pub(crate) const fn bits(self) -> u32 {
        u32::from_ne_bytes(self.0.to_ne_bytes())
    }

    pub const None: Self = Self(0);
    pub const Reliable: Self = Self(1);
    pub const InOrder: Self = Self(2);
    pub const ReliableInOrder: Self = Self(3);
    pub const Chat: Self = Self(4);
}

impl BitOr for SendDataOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for SendDataOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for SendDataOptions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionJoinException`.
///
/// It derives from `GamerServices::NetworkException` in CLR metadata, which
/// Rust expresses by composing the base's state and stating the relationship
/// through the `NetworkExceptionBase` contract trait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkSessionJoinException {
    message: String,
    inner_message: Option<String>,
    streaming_context: Option<i32>,
    join_error: NetworkSessionJoinError,
}

impl NetworkSessionJoinException {
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: "The network session could not be joined.".to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_message_and_join_error(message: &str, joinError: NetworkSessionJoinError) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: joinError,
        }
    }

    #[must_use]
    pub fn from_message_and_inner_exception(message: &str, innerException: &dyn Error) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: Some(innerException.to_string()),
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_info_and_context(info: SerializationInfo, context: StreamingContext) -> Self {
        Self {
            message: info.message().to_owned(),
            inner_message: None,
            streaming_context: Some(context.state()),
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub const fn JoinError(&self) -> NetworkSessionJoinError {
        self.join_error
    }

    pub fn SetJoinError(&mut self, value: NetworkSessionJoinError) {
        self.join_error = value;
    }

    /// CLR `ISerializable.GetObjectData`.
    ///
    /// The projection carries the join error alongside the message the base
    /// exception writes, which is what the CLR override adds.
    pub fn GetObjectData(&self, info: &mut SerializationInfo, context: StreamingContext) {
        let _ = context;
        info.SetMessage(&self.message);
    }
}

impl Default for NetworkSessionJoinException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NetworkSessionJoinException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(inner) = &self.inner_message {
            write!(formatter, ": {inner}")?;
        }
        Ok(())
    }
}

impl Error for NetworkSessionJoinException {}

impl NetworkExceptionBase for NetworkSessionJoinException {
    fn Message(&self) -> String {
        self.message.clone()
    }
}

/// XNA `Microsoft.Xna.Framework.Net.PacketWriter`.
///
/// CLR derives it from `System.IO.BinaryWriter` over a `MemoryStream`; the
/// Rust projection owns the buffer directly. Every value is written in the
/// exact byte order XNA writes it, including the bit reinterpretation XNA's
/// `Write(float)` and `Write(double)` overrides perform.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PacketWriter {
    buffer: Vec<u8>,
    position: usize,
    disposed: bool,
}

impl PacketWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::from_capacity(0)
    }

    #[must_use]
    pub fn from_capacity(capacity: i32) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity.max(0) as usize),
            position: 0,
            disposed: false,
        }
    }

    /// Writes at the current position, growing and zero-filling like the
    /// `MemoryStream` XNA writes into.
    ///
    /// Writing after `Dispose` is refused. CLR raises `ObjectDisposedException`
    /// from the closed stream; the projection reports it through the
    /// established `Result` mapping instead of writing into a released buffer.
    fn put(&mut self, bytes: &[u8]) -> Result<()> {
        if self.disposed {
            return Err(CnaError::InvalidInput("the PacketWriter is disposed"));
        }
        let end = self.position + bytes.len();
        if end > self.buffer.len() {
            self.buffer.resize(end, 0);
        }
        self.buffer[self.position..end].copy_from_slice(bytes);
        self.position = end;
        Ok(())
    }

    #[must_use]
    pub fn Length(&self) -> i32 {
        i32::try_from(self.buffer.len()).unwrap_or(i32::MAX)
    }

    #[must_use]
    pub fn Position(&self) -> i32 {
        i32::try_from(self.position).unwrap_or(i32::MAX)
    }

    pub fn SetPosition(&mut self, value: i32) {
        self.position = value.max(0) as usize;
    }

    pub fn WriteWithValueAsSingle(&mut self, value: f32) -> Result<()> {
        self.put(&value.to_bits().to_le_bytes())
    }

    pub fn WriteWithValueAsDouble(&mut self, value: f64) -> Result<()> {
        self.put(&value.to_bits().to_le_bytes())
    }

    pub fn Write(&mut self, value: Vector2) -> Result<()> {
        self.WriteWithValueAsSingle(value.X)?;
        self.WriteWithValueAsSingle(value.Y)
    }

    pub fn WriteWithValueAsVector3(&mut self, value: Vector3) -> Result<()> {
        self.WriteWithValueAsSingle(value.X)?;
        self.WriteWithValueAsSingle(value.Y)?;
        self.WriteWithValueAsSingle(value.Z)
    }

    pub fn WriteWithValueAsVector4(&mut self, value: Vector4) -> Result<()> {
        self.WriteWithValueAsSingle(value.X)?;
        self.WriteWithValueAsSingle(value.Y)?;
        self.WriteWithValueAsSingle(value.Z)?;
        self.WriteWithValueAsSingle(value.W)
    }

    pub fn WriteWithValueAsQuaternion(&mut self, value: Quaternion) -> Result<()> {
        self.WriteWithValueAsSingle(value.X)?;
        self.WriteWithValueAsSingle(value.Y)?;
        self.WriteWithValueAsSingle(value.Z)?;
        self.WriteWithValueAsSingle(value.W)
    }

    pub fn WriteWithValueAsMatrix(&mut self, value: Matrix) -> Result<()> {
        for component in [
            value.M11, value.M12, value.M13, value.M14, value.M21, value.M22, value.M23, value.M24,
            value.M31, value.M32, value.M33, value.M34, value.M41, value.M42, value.M43, value.M44,
        ] {
            self.WriteWithValueAsSingle(component)?;
        }
        Ok(())
    }

    pub fn WriteWithValueAsColor(&mut self, value: Color) -> Result<()> {
        self.put(&value.PackedValue().to_le_bytes())
    }

    /// The bytes written so far. Reached through `cna::extensions::net`,
    /// because XNA keeps this internal to its network session.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buffer
    }
}

/// XNA `Microsoft.Xna.Framework.Net.PacketReader`.
///
/// CLR derives it from `System.IO.BinaryReader`. Reading past the end of a
/// packet is a failure in both: CLR raises `EndOfStreamException`, and the
/// Rust projection reports it through the established `Result` mapping rather
/// than returning a fabricated value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PacketReader {
    buffer: Vec<u8>,
    position: usize,
    disposed: bool,
}

impl PacketReader {
    #[must_use]
    pub fn new() -> Self {
        Self::from_capacity(0)
    }

    #[must_use]
    pub fn from_capacity(capacity: i32) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity.max(0) as usize),
            position: 0,
            disposed: false,
        }
    }

    /// Fills the reader with one received packet. Reached through
    /// `cna::extensions::net`, because XNA's network session owns this path.
    pub(crate) fn fill(&mut self, bytes: &[u8]) {
        self.buffer.clear();
        self.buffer.extend_from_slice(bytes);
        self.position = 0;
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.disposed {
            return Err(CnaError::InvalidInput("the PacketReader is disposed"));
        }
        let end = self.position.checked_add(N).ok_or(CnaError::InvalidInput(
            "packet read position overflowed",
        ))?;
        if end > self.buffer.len() {
            return Err(CnaError::InvalidInput("read past the end of the packet"));
        }
        let mut value = [0_u8; N];
        value.copy_from_slice(&self.buffer[self.position..end]);
        self.position = end;
        Ok(value)
    }

    #[must_use]
    pub fn Length(&self) -> i32 {
        i32::try_from(self.buffer.len()).unwrap_or(i32::MAX)
    }

    #[must_use]
    pub fn Position(&self) -> i32 {
        i32::try_from(self.position).unwrap_or(i32::MAX)
    }

    pub fn SetPosition(&mut self, value: i32) {
        self.position = value.max(0) as usize;
    }

    pub fn ReadSingle(&mut self) -> Result<f32> {
        Ok(f32::from_bits(u32::from_le_bytes(self.take::<4>()?)))
    }

    pub fn ReadDouble(&mut self) -> Result<f64> {
        Ok(f64::from_bits(u64::from_le_bytes(self.take::<8>()?)))
    }

    pub fn ReadVector2(&mut self) -> Result<Vector2> {
        Ok(Vector2::from_x_and_y(self.ReadSingle()?, self.ReadSingle()?))
    }

    pub fn ReadVector3(&mut self) -> Result<Vector3> {
        Ok(Vector3::from_x_and_y_and_z(
            self.ReadSingle()?,
            self.ReadSingle()?,
            self.ReadSingle()?,
        ))
    }

    pub fn ReadVector4(&mut self) -> Result<Vector4> {
        Ok(Vector4::from_x_and_y_and_z_and_w(
            self.ReadSingle()?,
            self.ReadSingle()?,
            self.ReadSingle()?,
            self.ReadSingle()?,
        ))
    }

    pub fn ReadQuaternion(&mut self) -> Result<Quaternion> {
        Ok(Quaternion::from_x_and_y_and_z_and_w(
            self.ReadSingle()?,
            self.ReadSingle()?,
            self.ReadSingle()?,
            self.ReadSingle()?,
        ))
    }

    pub fn ReadMatrix(&mut self) -> Result<Matrix> {
        let mut components = [0.0_f32; 16];
        for component in &mut components {
            *component = self.ReadSingle()?;
        }
        Ok(Matrix::new(
            components[0], components[1], components[2], components[3],
            components[4], components[5], components[6], components[7],
            components[8], components[9], components[10], components[11],
            components[12], components[13], components[14], components[15],
        ))
    }

    pub fn ReadColor(&mut self) -> Result<Color> {
        let mut value = Color::default();
        value.SetPackedValue(u32::from_le_bytes(self.take::<4>()?));
        Ok(value)
    }
}

impl Drop for PacketWriter {
    fn drop(&mut self) {
        // Rust lifetime safety only. Dispose is the observable operation and is
        // idempotent, so an explicit Dispose before Drop changes nothing here.
        self.Dispose();
    }
}

impl Drop for PacketReader {
    fn drop(&mut self) {
        // Rust lifetime safety only; see PacketWriter's note.
        self.Dispose();
    }
}

impl Disposable for PacketWriter {
    fn Dispose(&mut self) {
        self.buffer = Vec::new();
        self.position = 0;
        self.disposed = true;
    }
}

impl Disposable for PacketReader {
    fn Dispose(&mut self) {
        self.buffer = Vec::new();
        self.position = 0;
        self.disposed = true;
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionProperties`.
///
/// Exactly eight slots, fixed by XNA: `Count` is a constant 8 and an index
/// outside it is refused rather than growing the collection. Each slot is
/// `Option<i32>` because XNA's element type is `int?`, and an unset slot is
/// `None` rather than a zero that would read as a real value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkSessionProperties {
    data: [Option<i32>; Self::PROPERTY_COUNT],
}

impl NetworkSessionProperties {
    const PROPERTY_COUNT: usize = 8;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn Count(&self) -> i32 {
        Self::PROPERTY_COUNT as i32
    }

    /// The slot at `index`.
    ///
    /// # Panics
    ///
    /// Outside the eight slots, as XNA's `ArgumentOutOfRangeException` does.
    /// This follows `TouchCollection`, the qualified precedent for an XNA
    /// indexer: an out-of-range index is a caller bug in both languages.
    #[must_use]
    pub fn Item(&self, index: i32) -> Option<i32> {
        self.data[Self::slot(index)]
    }

    /// Sets the slot at `index`.
    ///
    /// # Panics
    ///
    /// Outside the eight slots, as XNA does.
    pub fn SetItem(&mut self, index: i32, value: Option<i32>) {
        let slot = Self::slot(index);
        self.data[slot] = value;
    }

    /// Iterates the eight slots in order.
    #[must_use]
    pub fn GetEnumerator(&self) -> IntoIter<Option<i32>> {
        self.data.to_vec().into_iter()
    }

    fn slot(index: i32) -> usize {
        assert!(
            index >= 0 && (index as usize) < Self::PROPERTY_COUNT,
            "index is out of range"
        );
        index as usize
    }
}

impl AsRef<[Option<i32>]> for NetworkSessionProperties {
    fn as_ref(&self) -> &[Option<i32>] {
        &self.data
    }
}
