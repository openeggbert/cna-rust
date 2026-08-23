#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ContainmentType {
    Disjoint = 0,
    Contains = 1,
    Intersects = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum PlaneIntersectionType {
    Front = 0,
    Back = 1,
    Intersecting = 2,
}
