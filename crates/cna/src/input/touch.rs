#![allow(non_snake_case, non_upper_case_globals)]

use core::{
    any::Any,
    mem::size_of,
    ops::{BitAnd, BitOr, BitOrAssign},
};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::window::WindowHandle;
use crate::game::{DisplayOrientation, GameContext, TimeSpan};
use crate::value::{vector_support::xna_f32_hash, Vector2};

/// Open flags representation of XNA's gesture selection.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GestureType(i32);

impl GestureType {
    pub const None: Self = Self(0);
    pub const Tap: Self = Self(1);
    pub const DoubleTap: Self = Self(2);
    pub const Hold: Self = Self(4);
    pub const HorizontalDrag: Self = Self(8);
    pub const VerticalDrag: Self = Self(16);
    pub const FreeDrag: Self = Self(32);
    pub const Pinch: Self = Self(64);
    pub const Flick: Self = Self(128);
    pub const DragComplete: Self = Self(256);
    pub const PinchComplete: Self = Self(512);

    const ALL_BITS: i32 = 0x3ff;

    const fn from_bits(value: i32) -> Self {
        Self(value)
    }

    pub(crate) const fn bits(self) -> i32 {
        self.0
    }
}

impl BitOr for GestureType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for GestureType {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for GestureType {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GestureSample {
    gesture_type: GestureType,
    timestamp: TimeSpan,
    position: Vector2,
    position2: Vector2,
    delta: Vector2,
    delta2: Vector2,
}

impl GestureSample {
    #[must_use]
    pub const fn new(
        gestureType: GestureType,
        timestamp: TimeSpan,
        position: Vector2,
        position2: Vector2,
        delta: Vector2,
        delta2: Vector2,
    ) -> Self {
        Self {
            gesture_type: gestureType,
            timestamp,
            position,
            position2,
            delta,
            delta2,
        }
    }

    #[must_use]
    pub const fn GestureType(&self) -> GestureType {
        self.gesture_type
    }
    #[must_use]
    pub const fn Timestamp(&self) -> TimeSpan {
        self.timestamp
    }
    #[must_use]
    pub const fn Position(&self) -> Vector2 {
        self.position
    }
    #[must_use]
    pub const fn Position2(&self) -> Vector2 {
        self.position2
    }
    #[must_use]
    pub const fn Delta(&self) -> Vector2 {
        self.delta
    }
    #[must_use]
    pub const fn Delta2(&self) -> Vector2 {
        self.delta2
    }

    fn from_native(value: &sys::CNA_GestureSample) -> Result<Self> {
        let bits = i32::try_from(value.gesture_type)
            .map_err(|_| CnaError::InvalidInput("gesture flags exceed i32"))?;
        if bits & !GestureType::ALL_BITS != 0 {
            return Err(CnaError::InvalidInput(
                "CNA returned undefined gesture flags",
            ));
        }
        Ok(Self::new(
            GestureType::from_bits(bits),
            TimeSpan::from_ticks(value.timestamp_ticks),
            vector(value.position),
            vector(value.position2),
            vector(value.delta),
            vector(value.delta2),
        ))
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TouchLocationState {
    Invalid = 0,
    Released = 1,
    Pressed = 2,
    Moved = 3,
}

impl Default for TouchLocationState {
    fn default() -> Self {
        Self::Invalid
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TouchLocation {
    id: i32,
    state: TouchLocationState,
    x: f32,
    y: f32,
    previous_state: TouchLocationState,
    previous_x: f32,
    previous_y: f32,
}

impl Default for TouchLocation {
    fn default() -> Self {
        Self {
            id: 0,
            state: TouchLocationState::Invalid,
            x: 0.0,
            y: 0.0,
            previous_state: TouchLocationState::Invalid,
            previous_x: 0.0,
            previous_y: 0.0,
        }
    }
}

impl PartialEq for TouchLocation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.state == other.state
            && self.x == other.x
            && self.y == other.y
            && self.previous_state == other.previous_state
            && self.previous_x == other.previous_x
            && self.previous_y == other.previous_y
    }
}

impl TouchLocation {
    #[must_use]
    pub const fn new(id: i32, state: TouchLocationState, position: Vector2) -> Self {
        Self {
            id,
            state,
            x: position.X,
            y: position.Y,
            previous_state: TouchLocationState::Invalid,
            previous_x: 0.0,
            previous_y: 0.0,
        }
    }

    #[must_use]
    pub const fn from_id_and_state_and_position_and_previous_state_and_previous_position(
        id: i32,
        state: TouchLocationState,
        position: Vector2,
        previousState: TouchLocationState,
        previousPosition: Vector2,
    ) -> Self {
        Self {
            id,
            state,
            x: position.X,
            y: position.Y,
            previous_state: previousState,
            previous_x: previousPosition.X,
            previous_y: previousPosition.Y,
        }
    }

    #[must_use]
    pub const fn State(&self) -> TouchLocationState {
        self.state
    }

    #[must_use]
    pub const fn Id(&self) -> i32 {
        self.id
    }

    #[must_use]
    pub const fn Position(&self) -> Vector2 {
        Vector2::from_x_and_y(self.x, self.y)
    }

    pub fn TryGetPreviousLocation(&self, previousLocation: &mut Self) -> bool {
        if self.previous_state == TouchLocationState::Invalid {
            *previousLocation = Self {
                id: -1,
                ..Self::default()
            };
            false
        } else {
            *previousLocation = Self::new(
                self.id,
                self.previous_state,
                Vector2::from_x_and_y(self.previous_x, self.previous_y),
            );
            true
        }
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{Position:{}}}", self.Position().ToString())
    }

    /// XNA's strongly typed `Equals` intentionally ignores current and
    /// previous state while its equality operator compares those fields.
    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.id == other.id
            && self.x == other.x
            && self.y == other.y
            && self.previous_x == other.previous_x
            && self.previous_y == other.previous_y
    }

    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.id
            .wrapping_add(xna_f32_hash(self.x))
            .wrapping_add(xna_f32_hash(self.y))
    }

    fn from_native(value: &sys::CNA_TouchLocation) -> Result<Self> {
        let state = touch_state(value.state)?;
        let previous_state = touch_state(value.previous_state)?;
        Ok(
            Self::from_id_and_state_and_position_and_previous_state_and_previous_position(
                value.id,
                state,
                vector(value.position),
                previous_state,
                vector(value.previous_position),
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchCollection {
    is_connected: bool,
    locations: [TouchLocation; 8],
    count: usize,
}

impl Default for TouchCollection {
    fn default() -> Self {
        Self {
            is_connected: false,
            locations: [TouchLocation::default(); 8],
            count: 0,
        }
    }
}

impl TouchCollection {
    #[must_use]
    pub fn new(touches: &[TouchLocation]) -> Self {
        assert!(
            touches.len() <= 8,
            "touches contains more than eight locations"
        );
        let mut result = Self {
            is_connected: true,
            ..Self::default()
        };
        result.locations[..touches.len()].copy_from_slice(touches);
        result.count = touches.len();
        result
    }

    #[must_use]
    pub const fn IsConnected(&self) -> bool {
        self.is_connected
    }

    #[must_use]
    pub fn Item(&self, index: i32) -> TouchLocation {
        self.locations[self.checked_index(index)]
    }

    pub fn SetItem(&mut self, index: i32, value: TouchLocation) {
        let _ = (index, value);
        read_only_collection()
    }

    #[must_use]
    pub fn Count(&self) -> i32 {
        self.count as i32
    }

    #[must_use]
    pub const fn IsReadOnly(&self) -> bool {
        true
    }

    pub fn FindById(&self, id: i32, touchLocation: &mut TouchLocation) -> bool {
        if let Some(location) = self.as_ref().iter().find(|location| location.Id() == id) {
            *touchLocation = *location;
            true
        } else {
            *touchLocation = TouchLocation::default();
            false
        }
    }

    #[must_use]
    pub fn IndexOf(&self, item: TouchLocation) -> i32 {
        self.as_ref()
            .iter()
            .position(|candidate| *candidate == item)
            .map_or(-1, |index| index as i32)
    }

    pub fn Insert(&mut self, index: i32, item: TouchLocation) {
        let _ = (index, item);
        read_only_collection()
    }

    pub fn RemoveAt(&mut self, index: i32) {
        let _ = index;
        read_only_collection()
    }

    pub fn Add(&mut self, item: TouchLocation) {
        let _ = item;
        read_only_collection()
    }

    pub fn Clear(&mut self) {
        read_only_collection()
    }

    #[must_use]
    pub fn Contains(&self, item: TouchLocation) -> bool {
        self.IndexOf(item) >= 0
    }

    pub fn CopyTo(&mut self, array: &mut [TouchLocation], arrayIndex: i32) {
        assert!(arrayIndex >= 0, "arrayIndex is negative");
        let start = arrayIndex as usize;
        let end = start
            .checked_add(self.count)
            .expect("arrayIndex plus Count overflows");
        assert!(end <= array.len(), "array is too short");
        array[start..end].copy_from_slice(self.as_ref());
    }

    #[must_use]
    pub fn Remove(&self, item: TouchLocation) -> bool {
        let _ = item;
        read_only_collection()
    }

    #[must_use]
    pub fn GetEnumerator(&self) -> TouchCollectionEnumerator {
        TouchCollectionEnumerator {
            collection: *self,
            position: -1,
        }
    }

    fn checked_index(&self, index: i32) -> usize {
        assert!(
            index >= 0 && (index as usize) < self.count,
            "index is out of range"
        );
        index as usize
    }

    fn from_native(value: &sys::CNA_TouchState) -> Result<Self> {
        let count = usize::try_from(value.touch_count)
            .map_err(|_| CnaError::InvalidInput("touch count exceeds usize"))?;
        if count > value.touches.len() {
            return Err(CnaError::InvalidInput(
                "CNA returned too many touch locations",
            ));
        }
        let mut result = Self {
            is_connected: value.is_connected != sys::CNA_FALSE,
            ..Self::default()
        };
        for (destination, source) in result.locations.iter_mut().zip(&value.touches).take(count) {
            *destination = TouchLocation::from_native(source)?;
        }
        result.count = count;
        Ok(result)
    }
}

impl AsRef<[TouchLocation]> for TouchCollection {
    fn as_ref(&self) -> &[TouchLocation] {
        &self.locations[..self.count]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchCollectionEnumerator {
    collection: TouchCollection,
    position: i32,
}

impl TouchCollectionEnumerator {
    #[must_use]
    pub fn Current(&self) -> TouchLocation {
        self.collection.Item(self.position)
    }

    pub fn MoveNext(&mut self) -> bool {
        self.position += 1;
        if self.position >= self.collection.Count() {
            self.position = self.collection.Count();
            false
        } else {
            true
        }
    }

    pub fn Dispose(&mut self) {}
}

impl Iterator for TouchCollectionEnumerator {
    type Item = TouchLocation;

    fn next(&mut self) -> Option<Self::Item> {
        self.MoveNext().then(|| self.Current())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TouchPanelCapabilities {
    is_connected: bool,
    maximum_touch_count: i32,
}

impl TouchPanelCapabilities {
    #[must_use]
    pub const fn IsConnected(&self) -> bool {
        self.is_connected
    }

    #[must_use]
    pub const fn MaximumTouchCount(&self) -> i32 {
        self.maximum_touch_count
    }

    fn from_native(value: &sys::CNA_TouchCapabilities) -> Result<Self> {
        Ok(Self {
            is_connected: value.is_connected != sys::CNA_FALSE,
            maximum_touch_count: i32::try_from(value.maximum_touch_count)
                .map_err(|_| CnaError::InvalidInput("touch count exceeds i32"))?,
        })
    }
}

pub struct TouchPanel;

impl TouchPanel {
    pub fn EnabledGestures(game: &GameContext<'_>) -> Result<GestureType> {
        let value = game.native.enabled_gestures(game.handle)?;
        let bits =
            i32::try_from(value).map_err(|_| CnaError::InvalidInput("gesture flags exceed i32"))?;
        if bits & !GestureType::ALL_BITS != 0 {
            return Err(CnaError::InvalidInput(
                "CNA returned undefined gesture flags",
            ));
        }
        Ok(GestureType::from_bits(bits))
    }

    pub fn SetEnabledGestures(game: &GameContext<'_>, value: GestureType) -> Result<()> {
        if value.bits() & !GestureType::ALL_BITS != 0 {
            return Err(CnaError::InvalidInput(
                "gesture flags contain undefined bits",
            ));
        }
        game.native
            .set_enabled_gestures(game.handle, value.bits() as u32)
    }

    pub fn IsGestureAvailable(game: &GameContext<'_>) -> Result<bool> {
        game.native.is_gesture_available(game.handle)
    }

    pub fn WindowHandle(game: &GameContext<'_>) -> Result<WindowHandle> {
        Ok(WindowHandle(game.native.touch_window_handle(game.handle)?))
    }

    pub fn SetWindowHandle(game: &GameContext<'_>, value: WindowHandle) -> Result<()> {
        game.native.set_touch_window_handle(game.handle, value.0)
    }

    pub fn DisplayOrientation(game: &GameContext<'_>) -> Result<DisplayOrientation> {
        let value = i32::try_from(game.native.touch_display_orientation(game.handle)?)
            .map_err(|_| CnaError::InvalidInput("display orientation exceeds i32"))?;
        Ok(DisplayOrientation::from_bits(value))
    }

    pub fn SetDisplayOrientation(game: &GameContext<'_>, value: DisplayOrientation) -> Result<()> {
        game.native
            .set_touch_display_orientation(game.handle, value.bits() as u32)
    }

    pub fn DisplayWidth(game: &GameContext<'_>) -> Result<i32> {
        game.native.touch_display_width(game.handle)
    }

    pub fn SetDisplayWidth(game: &GameContext<'_>, value: i32) -> Result<()> {
        game.native.set_touch_display_width(game.handle, value)
    }

    pub fn DisplayHeight(game: &GameContext<'_>) -> Result<i32> {
        game.native.touch_display_height(game.handle)
    }

    pub fn SetDisplayHeight(game: &GameContext<'_>, value: i32) -> Result<()> {
        game.native.set_touch_display_height(game.handle, value)
    }

    pub fn GetCapabilities(game: &GameContext<'_>) -> Result<TouchPanelCapabilities> {
        let mut value = sys::CNA_TouchCapabilities {
            struct_size: size_of::<sys::CNA_TouchCapabilities>() as u32,
            struct_version: 1,
            ..sys::CNA_TouchCapabilities::default()
        };
        game.native.touch_capabilities(game.handle, &mut value)?;
        TouchPanelCapabilities::from_native(&value)
    }

    pub fn ReadGesture(game: &GameContext<'_>) -> Result<GestureSample> {
        let mut value = sys::CNA_GestureSample {
            struct_size: size_of::<sys::CNA_GestureSample>() as u32,
            struct_version: 1,
            ..sys::CNA_GestureSample::default()
        };
        game.native.read_gesture(game.handle, &mut value)?;
        GestureSample::from_native(&value)
    }

    pub fn GetState(game: &GameContext<'_>) -> Result<TouchCollection> {
        let mut value = sys::CNA_TouchState {
            struct_size: size_of::<sys::CNA_TouchState>() as u32,
            struct_version: 1,
            ..sys::CNA_TouchState::default()
        };
        game.native.touch_state(game.handle, &mut value)?;
        TouchCollection::from_native(&value)
    }
}

const fn vector(value: sys::CNA_Vector2) -> Vector2 {
    Vector2::from_x_and_y(value.x, value.y)
}

fn touch_state(value: sys::CNA_TouchLocationState) -> Result<TouchLocationState> {
    match value {
        0 => Ok(TouchLocationState::Invalid),
        1 => Ok(TouchLocationState::Released),
        2 => Ok(TouchLocationState::Pressed),
        3 => Ok(TouchLocationState::Moved),
        _ => Err(CnaError::InvalidInput(
            "CNA returned an undefined touch state",
        )),
    }
}

fn read_only_collection() -> ! {
    panic!("TouchCollection is read-only")
}
