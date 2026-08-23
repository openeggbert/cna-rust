#![allow(non_snake_case, clippy::cast_possible_truncation)]

use core::any::Any;
use core::cell::RefCell;
use core::cmp::Ordering;
use std::rc::Rc;
use std::vec::IntoIter;

use super::vector_support::xna_f32_hash;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CurveContinuity {
    #[default]
    Smooth = 0,
    Step = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CurveLoopType {
    #[default]
    Constant = 0,
    Cycle = 1,
    CycleOffset = 2,
    Oscillate = 3,
    Linear = 4,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CurveTangent {
    #[default]
    Flat = 0,
    Linear = 1,
    Smooth = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CurveKeyData {
    position: f32,
    value: f32,
    tangent_in: f32,
    tangent_out: f32,
    continuity: CurveContinuity,
}

/// Shared-reference XNA curve key. Rust `Clone` preserves object identity;
/// the XNA-named `Clone` method creates a distinct key object.
#[derive(Clone, Debug)]
pub struct CurveKey(Rc<RefCell<CurveKeyData>>);

impl CurveKey {
    pub fn new(position: f32, value: f32) -> Self {
        Self::from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
            position,
            value,
            0.0,
            0.0,
            CurveContinuity::Smooth,
        )
    }

    pub fn from_position_and_value_and_tangent_in_and_tangent_out(
        position: f32,
        value: f32,
        tangentIn: f32,
        tangentOut: f32,
    ) -> Self {
        Self::from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
            position,
            value,
            tangentIn,
            tangentOut,
            CurveContinuity::Smooth,
        )
    }

    pub fn from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
        position: f32,
        value: f32,
        tangentIn: f32,
        tangentOut: f32,
        continuity: CurveContinuity,
    ) -> Self {
        Self(Rc::new(RefCell::new(CurveKeyData {
            position,
            value,
            tangent_in: tangentIn,
            tangent_out: tangentOut,
            continuity,
        })))
    }

    pub fn Position(&self) -> f32 {
        self.0.borrow().position
    }

    pub fn Value(&self) -> f32 {
        self.0.borrow().value
    }

    pub fn SetValue(&mut self, value: f32) {
        self.0.borrow_mut().value = value;
    }

    pub fn TangentIn(&self) -> f32 {
        self.0.borrow().tangent_in
    }

    pub fn SetTangentIn(&mut self, value: f32) {
        self.0.borrow_mut().tangent_in = value;
    }

    pub fn TangentOut(&self) -> f32 {
        self.0.borrow().tangent_out
    }

    pub fn SetTangentOut(&mut self, value: f32) {
        self.0.borrow_mut().tangent_out = value;
    }

    pub fn Continuity(&self) -> CurveContinuity {
        self.0.borrow().continuity
    }

    pub fn SetContinuity(&mut self, value: CurveContinuity) {
        self.0.borrow_mut().continuity = value;
    }

    pub fn Clone(&self) -> Self {
        let value = *self.0.borrow();
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn CompareTo(&self, other: &Self) -> i32 {
        if self.Position() == other.Position() {
            0
        } else if self.Position() < other.Position() {
            -1
        } else {
            1
        }
    }

    pub fn Equals(&self, other: &Self) -> bool {
        self == other
    }

    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    pub fn GetHashCode(&self) -> i32 {
        let value = self.0.borrow();
        xna_f32_hash(value.position)
            .wrapping_add(xna_f32_hash(value.value))
            .wrapping_add(xna_f32_hash(value.tangent_in))
            .wrapping_add(xna_f32_hash(value.tangent_out))
            .wrapping_add(value.continuity as i32)
    }
}

impl PartialEq for CurveKey {
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow() == *other.0.borrow()
    }
}

impl PartialOrd for CurveKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.CompareTo(other) {
            value if value < 0 => Ordering::Less,
            0 => Ordering::Equal,
            _ => Ordering::Greater,
        })
    }
}

/// Sorted mutable collection retaining XNA key object identity.
#[derive(Debug, Default)]
pub struct CurveKeyCollection {
    keys: RefCell<Vec<CurveKey>>,
}

impl CurveKeyCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn Count(&self) -> i32 {
        self.keys.borrow().len() as i32
    }

    pub const fn IsReadOnly(&self) -> bool {
        false
    }

    pub fn Item(&self, index: i32) -> CurveKey {
        self.keys.borrow()[checked_index(index)].clone()
    }

    pub fn SetItem(&mut self, index: i32, value: &CurveKey) {
        let index = checked_index(index);
        let old_position = self.keys.borrow()[index].Position();
        if old_position == value.Position() {
            self.keys.borrow_mut()[index] = value.clone();
        } else {
            self.keys.borrow_mut().remove(index);
            self.Add(value);
        }
    }

    pub fn Add(&self, item: &CurveKey) {
        let mut keys = self.keys.borrow_mut();
        let mut index = 0;
        while index < keys.len() && keys[index].CompareTo(item) <= 0 {
            index += 1;
        }
        keys.insert(index, item.clone());
    }

    pub fn Clear(&self) {
        self.keys.borrow_mut().clear();
    }

    pub fn Contains(&self, item: &CurveKey) -> bool {
        self.keys.borrow().iter().any(|key| key == item)
    }

    pub fn CopyTo(&self, array: &mut [CurveKey], arrayIndex: i32) {
        let start = checked_index(arrayIndex);
        let keys = self.keys.borrow();
        let end = start.checked_add(keys.len()).expect("array index overflow");
        assert!(end <= array.len(), "destination array is too small");
        array[start..end].clone_from_slice(&keys);
    }

    pub fn IndexOf(&self, item: &CurveKey) -> i32 {
        self.keys
            .borrow()
            .iter()
            .position(|key| key == item)
            .map_or(-1, |index| index as i32)
    }

    pub fn Remove(&self, item: &CurveKey) -> bool {
        let Some(index) = self.keys.borrow().iter().position(|key| key == item) else {
            return false;
        };
        self.keys.borrow_mut().remove(index);
        true
    }

    pub fn RemoveAt(&self, index: i32) {
        self.keys.borrow_mut().remove(checked_index(index));
    }

    pub fn Clone(&self) -> Self {
        Self {
            keys: RefCell::new(self.keys.borrow().clone()),
        }
    }

    pub fn GetEnumerator(&self) -> IntoIter<CurveKey> {
        self.keys.borrow().clone().into_iter()
    }

    fn time_range(&self) -> (f32, f32) {
        let keys = self.keys.borrow();
        if keys.len() <= 1 {
            return (0.0, 0.0);
        }
        let range = keys[keys.len() - 1].Position() - keys[0].Position();
        let inverse = if range > f32::EPSILON {
            1.0 / range
        } else {
            0.0
        };
        (range, inverse)
    }
}

impl IntoIterator for CurveKeyCollection {
    type Item = CurveKey;
    type IntoIter = IntoIter<CurveKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_inner().into_iter()
    }
}

fn checked_index(index: i32) -> usize {
    usize::try_from(index).expect("index must be nonnegative")
}

/// Pure managed XNA scalar animation curve.
#[derive(Debug)]
pub struct Curve {
    keys: CurveKeyCollection,
    pre_loop: CurveLoopType,
    post_loop: CurveLoopType,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve {
    pub fn new() -> Self {
        Self {
            keys: CurveKeyCollection::new(),
            pre_loop: CurveLoopType::Constant,
            post_loop: CurveLoopType::Constant,
        }
    }

    pub const fn Keys(&self) -> &CurveKeyCollection {
        &self.keys
    }

    pub const fn PreLoop(&self) -> CurveLoopType {
        self.pre_loop
    }

    pub fn SetPreLoop(&mut self, value: CurveLoopType) {
        self.pre_loop = value;
    }

    pub const fn PostLoop(&self) -> CurveLoopType {
        self.post_loop
    }

    pub fn SetPostLoop(&mut self, value: CurveLoopType) {
        self.post_loop = value;
    }

    pub fn IsConstant(&self) -> bool {
        self.keys.Count() <= 1
    }

    pub fn Clone(&self) -> Self {
        Self {
            keys: self.keys.Clone(),
            pre_loop: self.pre_loop,
            post_loop: self.post_loop,
        }
    }

    pub fn ComputeTangent(&self, keyIndex: i32, tangentType: CurveTangent) {
        self.ComputeTangentWithKeyIndexAndTangentInTypeAndTangentOutType(
            keyIndex,
            tangentType,
            tangentType,
        );
    }

    pub fn ComputeTangentWithKeyIndexAndTangentInTypeAndTangentOutType(
        &self,
        keyIndex: i32,
        tangentInType: CurveTangent,
        tangentOutType: CurveTangent,
    ) {
        assert!(
            keyIndex >= 0 && keyIndex < self.keys.Count(),
            "key index is out of range"
        );
        let mut key = self.keys.Item(keyIndex);
        let previous = if keyIndex > 0 {
            self.keys.Item(keyIndex - 1)
        } else {
            key.clone()
        };
        let next = if keyIndex + 1 < self.keys.Count() {
            self.keys.Item(keyIndex + 1)
        } else {
            key.clone()
        };

        let tangent_in = match tangentInType {
            CurveTangent::Linear => key.Value() - previous.Value(),
            CurveTangent::Smooth => smooth_tangent(
                &previous,
                &next,
                (previous.Position() - key.Position()).abs(),
            ),
            CurveTangent::Flat => 0.0,
        };
        let tangent_out = match tangentOutType {
            CurveTangent::Linear => next.Value() - key.Value(),
            CurveTangent::Smooth => {
                smooth_tangent(&previous, &next, (next.Position() - key.Position()).abs())
            }
            CurveTangent::Flat => 0.0,
        };
        key.SetTangentIn(tangent_in);
        key.SetTangentOut(tangent_out);
    }

    pub fn ComputeTangents(&self, tangentType: CurveTangent) {
        self.ComputeTangentsWithTangentInTypeAndTangentOutType(tangentType, tangentType);
    }

    pub fn ComputeTangentsWithTangentInTypeAndTangentOutType(
        &self,
        tangentInType: CurveTangent,
        tangentOutType: CurveTangent,
    ) {
        for index in 0..self.keys.Count() {
            self.ComputeTangentWithKeyIndexAndTangentInTypeAndTangentOutType(
                index,
                tangentInType,
                tangentOutType,
            );
        }
    }

    pub fn Evaluate(&self, position: f32) -> f32 {
        let count = self.keys.Count();
        if count == 0 {
            return 0.0;
        }
        if count == 1 {
            return self.keys.Item(0).Value();
        }
        let first = self.keys.Item(0);
        let last = self.keys.Item(count - 1);
        let mut virtual_position = position;
        let mut value_offset = 0.0;
        if virtual_position < first.Position() {
            match self.pre_loop {
                CurveLoopType::Constant => return first.Value(),
                CurveLoopType::Linear => {
                    return first.Value()
                        - first.TangentIn() * (first.Position() - virtual_position);
                }
                _ => {
                    let cycle = self.calculate_cycle(virtual_position);
                    let (time_range, _) = self.keys.time_range();
                    let cycle_position = virtual_position - (first.Position() + cycle * time_range);
                    match self.pre_loop {
                        CurveLoopType::Cycle => {
                            virtual_position = first.Position() + cycle_position
                        }
                        CurveLoopType::CycleOffset => {
                            virtual_position = first.Position() + cycle_position;
                            value_offset = (last.Value() - first.Value()) * cycle;
                        }
                        _ => {
                            virtual_position = if (cycle as i32 & 1) == 0 {
                                first.Position() + cycle_position
                            } else {
                                last.Position() - cycle_position
                            };
                        }
                    }
                }
            }
        } else if last.Position() < virtual_position {
            match self.post_loop {
                CurveLoopType::Constant => return last.Value(),
                CurveLoopType::Linear => {
                    return last.Value() - last.TangentOut() * (last.Position() - virtual_position);
                }
                _ => {
                    let cycle = self.calculate_cycle(virtual_position);
                    let (time_range, _) = self.keys.time_range();
                    let cycle_position = virtual_position - (first.Position() + cycle * time_range);
                    match self.post_loop {
                        CurveLoopType::Cycle => {
                            virtual_position = first.Position() + cycle_position
                        }
                        CurveLoopType::CycleOffset => {
                            virtual_position = first.Position() + cycle_position;
                            value_offset = (last.Value() - first.Value()) * cycle;
                        }
                        _ => {
                            virtual_position = if (cycle as i32 & 1) == 0 {
                                first.Position() + cycle_position
                            } else {
                                last.Position() - cycle_position
                            };
                        }
                    }
                }
            }
        }
        let (start, end, amount) = self.find_segment(virtual_position);
        value_offset + hermite(&start, &end, amount)
    }

    fn calculate_cycle(&self, position: f32) -> f32 {
        let first = self.keys.Item(0).Position();
        let (_, inverse) = self.keys.time_range();
        let mut cycle = (position - first) * inverse;
        if cycle < 0.0 {
            cycle -= 1.0;
        }
        (cycle as i32) as f32
    }

    fn find_segment(&self, position: f32) -> (CurveKey, CurveKey, f32) {
        let mut start = self.keys.Item(0);
        for index in 1..self.keys.Count() {
            let end = self.keys.Item(index);
            if end.Position() >= position {
                let span = f64::from(end.Position()) - f64::from(start.Position());
                let amount = if span > 1e-10 {
                    ((f64::from(position) - f64::from(start.Position())) / span) as f32
                } else {
                    0.0
                };
                return (start, end, amount);
            }
            start = end;
        }
        let end = self.keys.Item(self.keys.Count() - 1);
        (start, end, position)
    }
}

fn smooth_tangent(previous: &CurveKey, next: &CurveKey, side_span: f32) -> f32 {
    let value_span = next.Value() - previous.Value();
    if value_span.abs() < f32::EPSILON {
        0.0
    } else {
        value_span * side_span / (next.Position() - previous.Position())
    }
}

fn hermite(start: &CurveKey, end: &CurveKey, amount: f32) -> f32 {
    if start.Continuity() == CurveContinuity::Step {
        return if amount < 1.0 {
            start.Value()
        } else {
            end.Value()
        };
    }
    let squared = amount * amount;
    let cubed = squared * amount;
    start.Value() * (2.0 * cubed - 3.0 * squared + 1.0)
        + end.Value() * (-2.0 * cubed + 3.0 * squared)
        + start.TangentOut() * (cubed - 2.0 * squared + amount)
        + end.TangentIn() * (cubed - squared)
}
