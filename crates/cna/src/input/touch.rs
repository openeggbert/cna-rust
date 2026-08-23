#![allow(non_snake_case)]

use core::any::Any;

use crate::value::{vector_support::xna_f32_hash, Vector2};

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
}

fn read_only_collection() -> ! {
    panic!("TouchCollection is read-only")
}
