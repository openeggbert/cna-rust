//! Managed projection of XNA's mathematical design-time converters.

#![allow(non_snake_case, non_upper_case_globals, clippy::missing_errors_doc)]

use crate::value::{
    BoundingBox, BoundingSphere, Color, Matrix, Plane, Point, Quaternion, Ray, Rectangle, Vector2,
    Vector3, Vector4,
};
use crate::{CnaError, Result};

/// Stable type identity used by the closed XNA Design conversion domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignType {
    String,
    InstanceDescriptor,
    Int32,
    Byte,
    Single,
    Point,
    Rectangle,
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Color,
    Matrix,
    BoundingBox,
    BoundingSphere,
    Plane,
    Ray,
}

/// Explicit formatting context for XNA Design component strings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignCulture {
    decimal_separator: char,
    list_separator: &'static str,
    nan_symbol: &'static str,
    positive_infinity_symbol: &'static str,
    negative_infinity_symbol: &'static str,
}

impl DesignCulture {
    pub const Invariant: Self = Self::new('.', ",", "NaN", "Infinity", "-Infinity");
    pub const EnUs: Self = Self::Invariant;
    pub const DeDe: Self = Self::new(',', ";", "NaN", "+unendlich", "-unendlich");

    #[must_use]
    pub const fn new(
        decimal_separator: char,
        list_separator: &'static str,
        nan_symbol: &'static str,
        positive_infinity_symbol: &'static str,
        negative_infinity_symbol: &'static str,
    ) -> Self {
        Self {
            decimal_separator,
            list_separator,
            nan_symbol,
            positive_infinity_symbol,
            negative_infinity_symbol,
        }
    }

    #[must_use]
    pub const fn decimal_separator(&self) -> char {
        self.decimal_separator
    }

    #[must_use]
    pub const fn list_separator(&self) -> &'static str {
        self.list_separator
    }
}

/// Closed value union replacing arbitrary CLR `object` values in Design APIs.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignValue {
    Null,
    String(String),
    Int32(i32),
    Byte(u8),
    Single(f32),
    Point(Point),
    Rectangle(Rectangle),
    Vector2(Vector2),
    Vector3(Vector3),
    Vector4(Vector4),
    Quaternion(Quaternion),
    Color(Color),
    Matrix(Matrix),
    BoundingBox(BoundingBox),
    BoundingSphere(BoundingSphere),
    Plane(Plane),
    Ray(Ray),
}

impl DesignValue {
    #[must_use]
    pub const fn ValueType(&self) -> Option<DesignType> {
        match self {
            Self::Null => None,
            Self::String(_) => Some(DesignType::String),
            Self::Int32(_) => Some(DesignType::Int32),
            Self::Byte(_) => Some(DesignType::Byte),
            Self::Single(_) => Some(DesignType::Single),
            Self::Point(_) => Some(DesignType::Point),
            Self::Rectangle(_) => Some(DesignType::Rectangle),
            Self::Vector2(_) => Some(DesignType::Vector2),
            Self::Vector3(_) => Some(DesignType::Vector3),
            Self::Vector4(_) => Some(DesignType::Vector4),
            Self::Quaternion(_) => Some(DesignType::Quaternion),
            Self::Color(_) => Some(DesignType::Color),
            Self::Matrix(_) => Some(DesignType::Matrix),
            Self::BoundingBox(_) => Some(DesignType::BoundingBox),
            Self::BoundingSphere(_) => Some(DesignType::BoundingSphere),
            Self::Plane(_) => Some(DesignType::Plane),
            Self::Ray(_) => Some(DesignType::Ray),
        }
    }
}

/// Immutable metadata for one ordered XNA Design property.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignPropertyDescriptor {
    name: &'static str,
    value_type: DesignType,
}

impl DesignPropertyDescriptor {
    const fn new(name: &'static str, value_type: DesignType) -> Self {
        Self { name, value_type }
    }

    #[must_use]
    pub const fn Name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn ValueType(&self) -> DesignType {
        self.value_type
    }
}

/// One named entry in the deterministic ordered property-value representation.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignPropertyValue {
    name: String,
    value: DesignValue,
}

impl DesignPropertyValue {
    #[must_use]
    pub fn new(name: impl Into<String>, value: DesignValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    #[must_use]
    pub fn Name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn Value(&self) -> &DesignValue {
        &self.value
    }
}

/// Stable identity for the constructors selected by XNA's converters.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignConstructor {
    PointInt32Int32,
    RectangleInt32Int32Int32Int32,
    Vector2SingleSingle,
    Vector3SingleSingleSingle,
    Vector4SingleSingleSingleSingle,
    QuaternionSingleSingleSingleSingle,
    ColorInt32Int32Int32Int32,
    MatrixSixteenSingles,
    BoundingBoxVector3Vector3,
    BoundingSphereVector3Single,
    PlaneVector3Single,
    RayVector3Vector3,
}

/// Small executable counterpart of CLR's `InstanceDescriptor`.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignInstanceDescriptor {
    constructor: DesignConstructor,
    arguments: Vec<DesignValue>,
    is_complete: bool,
}

impl DesignInstanceDescriptor {
    fn complete(constructor: DesignConstructor, arguments: Vec<DesignValue>) -> Self {
        Self {
            constructor,
            arguments,
            is_complete: true,
        }
    }

    #[must_use]
    pub const fn Constructor(&self) -> DesignConstructor {
        self.constructor
    }

    #[must_use]
    pub fn Arguments(&self) -> &[DesignValue] {
        &self.arguments
    }

    #[must_use]
    pub const fn IsComplete(&self) -> bool {
        self.is_complete
    }

    pub fn Invoke(&self) -> Result<DesignValue> {
        use DesignConstructor as C;
        require_argument_count(
            &self.arguments,
            constructor_argument_count(self.constructor),
        )?;
        match self.constructor {
            C::PointInt32Int32 => Ok(DesignValue::Point(Point::new(
                argument_i32(&self.arguments, 0)?,
                argument_i32(&self.arguments, 1)?,
            ))),
            C::RectangleInt32Int32Int32Int32 => Ok(DesignValue::Rectangle(Rectangle::new(
                argument_i32(&self.arguments, 0)?,
                argument_i32(&self.arguments, 1)?,
                argument_i32(&self.arguments, 2)?,
                argument_i32(&self.arguments, 3)?,
            ))),
            C::Vector2SingleSingle => Ok(DesignValue::Vector2(Vector2::from_x_and_y(
                argument_f32(&self.arguments, 0)?,
                argument_f32(&self.arguments, 1)?,
            ))),
            C::Vector3SingleSingleSingle => Ok(DesignValue::Vector3(Vector3::from_x_and_y_and_z(
                argument_f32(&self.arguments, 0)?,
                argument_f32(&self.arguments, 1)?,
                argument_f32(&self.arguments, 2)?,
            ))),
            C::Vector4SingleSingleSingleSingle => {
                Ok(DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(
                    argument_f32(&self.arguments, 0)?,
                    argument_f32(&self.arguments, 1)?,
                    argument_f32(&self.arguments, 2)?,
                    argument_f32(&self.arguments, 3)?,
                )))
            }
            C::QuaternionSingleSingleSingleSingle => Ok(DesignValue::Quaternion(
                Quaternion::from_x_and_y_and_z_and_w(
                    argument_f32(&self.arguments, 0)?,
                    argument_f32(&self.arguments, 1)?,
                    argument_f32(&self.arguments, 2)?,
                    argument_f32(&self.arguments, 3)?,
                ),
            )),
            C::ColorInt32Int32Int32Int32 => Ok(DesignValue::Color(
                Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                    i32::from(argument_u8(&self.arguments, 0)?),
                    i32::from(argument_u8(&self.arguments, 1)?),
                    i32::from(argument_u8(&self.arguments, 2)?),
                    i32::from(argument_u8(&self.arguments, 3)?),
                ),
            )),
            C::MatrixSixteenSingles => Ok(DesignValue::Matrix(Matrix::new(
                argument_f32(&self.arguments, 0)?,
                argument_f32(&self.arguments, 1)?,
                argument_f32(&self.arguments, 2)?,
                argument_f32(&self.arguments, 3)?,
                argument_f32(&self.arguments, 4)?,
                argument_f32(&self.arguments, 5)?,
                argument_f32(&self.arguments, 6)?,
                argument_f32(&self.arguments, 7)?,
                argument_f32(&self.arguments, 8)?,
                argument_f32(&self.arguments, 9)?,
                argument_f32(&self.arguments, 10)?,
                argument_f32(&self.arguments, 11)?,
                argument_f32(&self.arguments, 12)?,
                argument_f32(&self.arguments, 13)?,
                argument_f32(&self.arguments, 14)?,
                argument_f32(&self.arguments, 15)?,
            ))),
            C::BoundingBoxVector3Vector3 => Ok(DesignValue::BoundingBox(BoundingBox::new(
                argument_vector3(&self.arguments, 0)?,
                argument_vector3(&self.arguments, 1)?,
            ))),
            C::BoundingSphereVector3Single => {
                let radius = argument_f32(&self.arguments, 1)?;
                validate_radius(radius)?;
                Ok(DesignValue::BoundingSphere(BoundingSphere::new(
                    argument_vector3(&self.arguments, 0)?,
                    radius,
                )))
            }
            C::PlaneVector3Single => Ok(DesignValue::Plane(Plane::from_normal_and_d(
                argument_vector3(&self.arguments, 0)?,
                argument_f32(&self.arguments, 1)?,
            ))),
            C::RayVector3Vector3 => Ok(DesignValue::Ray(Ray::new(
                argument_vector3(&self.arguments, 0)?,
                argument_vector3(&self.arguments, 1)?,
            ))),
        }
    }
}

/// Result of converting a selected Design value to a supported destination.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignConversion {
    String(String),
    InstanceDescriptor(DesignInstanceDescriptor),
}

/// Common mapped contract for XNA's `MathTypeConverter` inheritance family.
pub trait MathTypeConverterBase {
    fn CanConvertFrom(&self, sourceType: DesignType) -> bool;
    fn CanConvertTo(&self, destinationType: DesignType) -> bool;
    fn GetCreateInstanceSupported(&self) -> bool;
    fn GetPropertiesSupported(&self) -> bool;
    fn GetProperties(&self) -> &'static [DesignPropertyDescriptor];
    fn GetPropertyValues(&self, value: Option<&DesignValue>) -> Result<Vec<DesignPropertyValue>>;
    fn ConvertFrom(
        &self,
        culture: &DesignCulture,
        value: Option<&DesignValue>,
    ) -> Result<DesignValue>;
    fn ConvertTo(
        &self,
        culture: &DesignCulture,
        value: Option<&DesignValue>,
        destinationType: Option<DesignType>,
    ) -> Result<DesignConversion>;
    fn CreateInstance(&self, propertyValues: Option<&[DesignPropertyValue]>)
        -> Result<DesignValue>;
}

/// Concrete, directly constructible foundation declared by XNA.
#[derive(Clone, Copy, Debug, Default)]
pub struct MathTypeConverter;

impl MathTypeConverter {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub const fn CanConvertFrom(&self, sourceType: DesignType) -> bool {
        matches!(sourceType, DesignType::String)
    }

    #[must_use]
    pub const fn CanConvertTo(&self, destinationType: DesignType) -> bool {
        matches!(
            destinationType,
            DesignType::String | DesignType::InstanceDescriptor
        )
    }

    #[must_use]
    pub const fn GetCreateInstanceSupported(&self) -> bool {
        true
    }

    #[must_use]
    pub const fn GetPropertiesSupported(&self) -> bool {
        true
    }

    #[must_use]
    pub const fn GetProperties(&self) -> &'static [DesignPropertyDescriptor] {
        &[]
    }
}

impl MathTypeConverterBase for MathTypeConverter {
    fn CanConvertFrom(&self, sourceType: DesignType) -> bool {
        Self::CanConvertFrom(self, sourceType)
    }

    fn CanConvertTo(&self, destinationType: DesignType) -> bool {
        Self::CanConvertTo(self, destinationType)
    }

    fn GetCreateInstanceSupported(&self) -> bool {
        Self::GetCreateInstanceSupported(self)
    }

    fn GetPropertiesSupported(&self) -> bool {
        Self::GetPropertiesSupported(self)
    }

    fn GetProperties(&self) -> &'static [DesignPropertyDescriptor] {
        Self::GetProperties(self)
    }

    fn GetPropertyValues(&self, _value: Option<&DesignValue>) -> Result<Vec<DesignPropertyValue>> {
        Ok(Vec::new())
    }

    fn ConvertFrom(
        &self,
        _culture: &DesignCulture,
        _value: Option<&DesignValue>,
    ) -> Result<DesignValue> {
        Err(unsupported_conversion())
    }

    fn ConvertTo(
        &self,
        _culture: &DesignCulture,
        value: Option<&DesignValue>,
        destinationType: Option<DesignType>,
    ) -> Result<DesignConversion> {
        match (value, destinationType) {
            (Some(value), Some(DesignType::String)) => {
                Ok(DesignConversion::String(fallback_string(value)?))
            }
            _ => Err(unsupported_conversion()),
        }
    }

    fn CreateInstance(
        &self,
        _propertyValues: Option<&[DesignPropertyValue]>,
    ) -> Result<DesignValue> {
        Err(unsupported_conversion())
    }
}

const POINT_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("X", DesignType::Int32),
    DesignPropertyDescriptor::new("Y", DesignType::Int32),
];
const RECTANGLE_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("X", DesignType::Int32),
    DesignPropertyDescriptor::new("Y", DesignType::Int32),
    DesignPropertyDescriptor::new("Width", DesignType::Int32),
    DesignPropertyDescriptor::new("Height", DesignType::Int32),
];
const VECTOR2_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("X", DesignType::Single),
    DesignPropertyDescriptor::new("Y", DesignType::Single),
];
const VECTOR3_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("X", DesignType::Single),
    DesignPropertyDescriptor::new("Y", DesignType::Single),
    DesignPropertyDescriptor::new("Z", DesignType::Single),
];
const VECTOR4_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("X", DesignType::Single),
    DesignPropertyDescriptor::new("Y", DesignType::Single),
    DesignPropertyDescriptor::new("Z", DesignType::Single),
    DesignPropertyDescriptor::new("W", DesignType::Single),
];
const COLOR_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("R", DesignType::Byte),
    DesignPropertyDescriptor::new("G", DesignType::Byte),
    DesignPropertyDescriptor::new("B", DesignType::Byte),
    DesignPropertyDescriptor::new("A", DesignType::Byte),
];
const MATRIX_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("Translation", DesignType::Vector3),
    DesignPropertyDescriptor::new("M11", DesignType::Single),
    DesignPropertyDescriptor::new("M12", DesignType::Single),
    DesignPropertyDescriptor::new("M13", DesignType::Single),
    DesignPropertyDescriptor::new("M14", DesignType::Single),
    DesignPropertyDescriptor::new("M21", DesignType::Single),
    DesignPropertyDescriptor::new("M22", DesignType::Single),
    DesignPropertyDescriptor::new("M23", DesignType::Single),
    DesignPropertyDescriptor::new("M24", DesignType::Single),
    DesignPropertyDescriptor::new("M31", DesignType::Single),
    DesignPropertyDescriptor::new("M32", DesignType::Single),
    DesignPropertyDescriptor::new("M33", DesignType::Single),
    DesignPropertyDescriptor::new("M34", DesignType::Single),
    DesignPropertyDescriptor::new("M41", DesignType::Single),
    DesignPropertyDescriptor::new("M42", DesignType::Single),
    DesignPropertyDescriptor::new("M43", DesignType::Single),
    DesignPropertyDescriptor::new("M44", DesignType::Single),
];
const BOUNDING_BOX_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("Min", DesignType::Vector3),
    DesignPropertyDescriptor::new("Max", DesignType::Vector3),
];
const BOUNDING_SPHERE_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("Center", DesignType::Vector3),
    DesignPropertyDescriptor::new("Radius", DesignType::Single),
];
const PLANE_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("Normal", DesignType::Vector3),
    DesignPropertyDescriptor::new("D", DesignType::Single),
];
const RAY_PROPERTIES: &[DesignPropertyDescriptor] = &[
    DesignPropertyDescriptor::new("Position", DesignType::Vector3),
    DesignPropertyDescriptor::new("Direction", DesignType::Vector3),
];

#[derive(Clone, Copy, Debug, Default)]
pub struct PointConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct RectangleConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector2Converter;
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3Converter;
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector4Converter;
#[derive(Clone, Copy, Debug, Default)]
pub struct QuaternionConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct ColorConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct MatrixConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingBoxConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingSphereConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct PlaneConverter;
#[derive(Clone, Copy, Debug, Default)]
pub struct RayConverter;

macro_rules! converter_impl {
    ($converter:ty, $variant:ident, $target:ty, $type_id:expr, $string:expr,
     $properties:expr, $parse:path, $format:path, $decompose:path, $create:path, $descriptor:path) => {
        impl MathTypeConverterBase for $converter {
            fn CanConvertFrom(&self, sourceType: DesignType) -> bool {
                $string && sourceType == DesignType::String
            }

            fn CanConvertTo(&self, destinationType: DesignType) -> bool {
                matches!(
                    destinationType,
                    DesignType::String | DesignType::InstanceDescriptor
                )
            }

            fn GetCreateInstanceSupported(&self) -> bool {
                true
            }

            fn GetPropertiesSupported(&self) -> bool {
                true
            }

            fn GetProperties(&self) -> &'static [DesignPropertyDescriptor] {
                $properties
            }

            fn GetPropertyValues(
                &self,
                value: Option<&DesignValue>,
            ) -> Result<Vec<DesignPropertyValue>> {
                let value = extract_value::<$target>(value, |value| match value {
                    DesignValue::$variant(value) => Some(*value),
                    _ => None,
                })?;
                let values = $decompose(&value);
                Ok($properties
                    .iter()
                    .zip(values)
                    .map(|(property, value)| DesignPropertyValue::new(property.Name(), value))
                    .collect())
            }

            fn ConvertFrom(
                &self,
                culture: &DesignCulture,
                value: Option<&DesignValue>,
            ) -> Result<DesignValue> {
                if !$string {
                    return Err(unsupported_conversion());
                }
                let text = match value {
                    Some(DesignValue::String(value)) => value,
                    _ => return Err(wrong_converter_value()),
                };
                $parse(culture, text).map(DesignValue::$variant)
            }

            fn ConvertTo(
                &self,
                culture: &DesignCulture,
                value: Option<&DesignValue>,
                destinationType: Option<DesignType>,
            ) -> Result<DesignConversion> {
                let destination = destinationType.ok_or_else(null_destination_type)?;
                match destination {
                    DesignType::String => match value {
                        Some(DesignValue::$variant(value)) => {
                            Ok(DesignConversion::String($format(value, culture)))
                        }
                        Some(value) => Ok(DesignConversion::String(fallback_string(value)?)),
                        None => Err(wrong_converter_value()),
                    },
                    DesignType::InstanceDescriptor => {
                        let value = extract_value::<$target>(value, |value| match value {
                            DesignValue::$variant(value) => Some(*value),
                            _ => None,
                        })?;
                        Ok(DesignConversion::InstanceDescriptor($descriptor(&value)))
                    }
                    _ => Err(unsupported_conversion()),
                }
            }

            fn CreateInstance(
                &self,
                propertyValues: Option<&[DesignPropertyValue]>,
            ) -> Result<DesignValue> {
                $create(propertyValues).map(DesignValue::$variant)
            }
        }

        impl $converter {
            #[must_use]
            pub const fn new() -> Self {
                Self
            }

            pub fn ConvertTo(
                &self,
                culture: &DesignCulture,
                value: Option<&DesignValue>,
                destinationType: Option<DesignType>,
            ) -> Result<DesignConversion> {
                <Self as MathTypeConverterBase>::ConvertTo(self, culture, value, destinationType)
            }

            pub fn CreateInstance(
                &self,
                propertyValues: Option<&[DesignPropertyValue]>,
            ) -> Result<$target> {
                $create(propertyValues)
            }
        }
    };
}

macro_rules! declared_convert_from {
    ($converter:ty, $variant:ident, $target:ty) => {
        impl $converter {
            pub fn ConvertFrom(
                &self,
                culture: &DesignCulture,
                value: Option<&DesignValue>,
            ) -> Result<$target> {
                match <Self as MathTypeConverterBase>::ConvertFrom(self, culture, value)? {
                    DesignValue::$variant(value) => Ok(value),
                    _ => Err(wrong_converter_value()),
                }
            }
        }
    };
}

converter_impl!(
    PointConverter,
    Point,
    Point,
    DesignType::Point,
    true,
    POINT_PROPERTIES,
    parse_point,
    format_point,
    decompose_point,
    create_point,
    descriptor_point
);
converter_impl!(
    RectangleConverter,
    Rectangle,
    Rectangle,
    DesignType::Rectangle,
    false,
    RECTANGLE_PROPERTIES,
    parse_rectangle,
    format_rectangle,
    decompose_rectangle,
    create_rectangle,
    descriptor_rectangle
);
converter_impl!(
    Vector2Converter,
    Vector2,
    Vector2,
    DesignType::Vector2,
    true,
    VECTOR2_PROPERTIES,
    parse_vector2,
    format_vector2,
    decompose_vector2,
    create_vector2,
    descriptor_vector2
);
converter_impl!(
    Vector3Converter,
    Vector3,
    Vector3,
    DesignType::Vector3,
    true,
    VECTOR3_PROPERTIES,
    parse_vector3,
    format_vector3,
    decompose_vector3,
    create_vector3,
    descriptor_vector3
);
converter_impl!(
    Vector4Converter,
    Vector4,
    Vector4,
    DesignType::Vector4,
    true,
    VECTOR4_PROPERTIES,
    parse_vector4,
    format_vector4,
    decompose_vector4,
    create_vector4,
    descriptor_vector4
);
converter_impl!(
    QuaternionConverter,
    Quaternion,
    Quaternion,
    DesignType::Quaternion,
    true,
    VECTOR4_PROPERTIES,
    parse_quaternion,
    format_quaternion,
    decompose_quaternion,
    create_quaternion,
    descriptor_quaternion
);
converter_impl!(
    ColorConverter,
    Color,
    Color,
    DesignType::Color,
    true,
    COLOR_PROPERTIES,
    parse_color,
    format_color,
    decompose_color,
    create_color,
    descriptor_color
);
converter_impl!(
    MatrixConverter,
    Matrix,
    Matrix,
    DesignType::Matrix,
    false,
    MATRIX_PROPERTIES,
    parse_matrix,
    format_matrix,
    decompose_matrix,
    create_matrix,
    descriptor_matrix
);
converter_impl!(
    BoundingBoxConverter,
    BoundingBox,
    BoundingBox,
    DesignType::BoundingBox,
    false,
    BOUNDING_BOX_PROPERTIES,
    parse_bounding_box,
    format_bounding_box,
    decompose_bounding_box,
    create_bounding_box,
    descriptor_bounding_box
);
converter_impl!(
    BoundingSphereConverter,
    BoundingSphere,
    BoundingSphere,
    DesignType::BoundingSphere,
    false,
    BOUNDING_SPHERE_PROPERTIES,
    parse_bounding_sphere,
    format_bounding_sphere,
    decompose_bounding_sphere,
    create_bounding_sphere,
    descriptor_bounding_sphere
);
converter_impl!(
    PlaneConverter,
    Plane,
    Plane,
    DesignType::Plane,
    false,
    PLANE_PROPERTIES,
    parse_plane,
    format_plane,
    decompose_plane,
    create_plane,
    descriptor_plane
);
converter_impl!(
    RayConverter,
    Ray,
    Ray,
    DesignType::Ray,
    false,
    RAY_PROPERTIES,
    parse_ray,
    format_ray,
    decompose_ray,
    create_ray,
    descriptor_ray
);

declared_convert_from!(PointConverter, Point, Point);
declared_convert_from!(Vector2Converter, Vector2, Vector2);
declared_convert_from!(Vector3Converter, Vector3, Vector3);
declared_convert_from!(Vector4Converter, Vector4, Vector4);
declared_convert_from!(QuaternionConverter, Quaternion, Quaternion);
declared_convert_from!(ColorConverter, Color, Color);
declared_convert_from!(BoundingBoxConverter, BoundingBox, BoundingBox);
declared_convert_from!(BoundingSphereConverter, BoundingSphere, BoundingSphere);
declared_convert_from!(RayConverter, Ray, Ray);

fn extract_value<T>(
    value: Option<&DesignValue>,
    extract: impl FnOnce(&DesignValue) -> Option<T>,
) -> Result<T> {
    let value = value.ok_or_else(wrong_converter_value)?;
    extract(value).ok_or_else(wrong_converter_value)
}

fn components<'a>(
    culture: &DesignCulture,
    value: &'a str,
    expected: usize,
) -> Result<Vec<&'a str>> {
    let values: Vec<_> = value.trim().split(culture.list_separator).collect();
    if values.len() != expected || values.iter().any(|value| value.trim().is_empty()) {
        return Err(invalid_component_string());
    }
    Ok(values)
}

fn parse_i32(value: &str) -> Result<i32> {
    let value = value.trim();
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value).as_bytes();
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err(invalid_component_string());
    }
    value.parse().map_err(|_| invalid_component_string())
}

fn parse_u8(value: &str) -> Result<u8> {
    let value = parse_i32(value)?;
    u8::try_from(value).map_err(|_| invalid_component_string())
}

fn parse_f32(culture: &DesignCulture, value: &str) -> Result<f32> {
    let value = value.trim();
    if value == culture.nan_symbol || value == "NaN" {
        return Ok(f32::NAN);
    }
    if value == culture.positive_infinity_symbol || value == "Infinity" || value == "+Infinity" {
        return Ok(f32::INFINITY);
    }
    if value == culture.negative_infinity_symbol || value == "-Infinity" {
        return Ok(f32::NEG_INFINITY);
    }
    if !valid_decimal(value, culture.decimal_separator) {
        return Err(invalid_component_string());
    }
    let invariant = if culture.decimal_separator == '.' {
        value.to_owned()
    } else {
        value.replace(culture.decimal_separator, ".")
    };
    let parsed: f32 = invariant.parse().map_err(|_| invalid_component_string())?;
    if parsed.is_infinite() {
        return Err(invalid_component_string());
    }
    Ok(parsed)
}

fn valid_decimal(value: &str, decimal: char) -> bool {
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (mantissa, exponent) = match value.find(['e', 'E']) {
        Some(index) => (&value[..index], Some(&value[index + 1..])),
        None => (value, None),
    };
    if let Some(exponent) = exponent {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if exponent.is_empty() || !exponent.as_bytes().iter().all(u8::is_ascii_digit) {
            return false;
        }
    }
    let mut decimal_count = 0;
    let mut digit_count = 0;
    for character in mantissa.chars() {
        if character == decimal {
            decimal_count += 1;
        } else if character.is_ascii_digit() {
            digit_count += 1;
        } else {
            return false;
        }
    }
    digit_count != 0 && decimal_count <= 1
}

fn format_values(culture: &DesignCulture, values: &[String]) -> String {
    values.join(&format!("{} ", culture.list_separator))
}

fn format_f32(value: f32, culture: &DesignCulture) -> String {
    if value.is_nan() {
        return culture.nan_symbol.to_owned();
    }
    if value == f32::INFINITY {
        return culture.positive_infinity_symbol.to_owned();
    }
    if value == f32::NEG_INFINITY {
        return culture.negative_infinity_symbol.to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }

    let negative = value.is_sign_negative();
    let absolute = f64::from(value.abs());
    let mut exponent = absolute.log10().floor() as i32;
    let mantissa = absolute / 10.0_f64.powi(exponent);
    let mut digits = (mantissa.mul_add(1_000_000.0, 0.5).floor()) as u32;
    if digits == 10_000_000 {
        digits = 1_000_000;
        exponent += 1;
    }
    let digits = format!("{digits:07}");
    let mut text = if exponent < -4 || exponent >= 7 {
        let fraction = digits[1..].trim_end_matches('0');
        let mantissa = if fraction.is_empty() {
            digits[..1].to_owned()
        } else {
            format!("{}.{}", &digits[..1], fraction)
        };
        format!(
            "{mantissa}E{}{:02}",
            if exponent >= 0 { "+" } else { "-" },
            exponent.abs()
        )
    } else if exponent >= 0 {
        let split = usize::try_from(exponent + 1).expect("nonnegative decimal split");
        if split == digits.len() {
            digits
        } else {
            trim_decimal(&format!("{}.{}", &digits[..split], &digits[split..]))
        }
    } else {
        let zeroes = usize::try_from(-exponent - 1).expect("nonnegative leading zero count");
        trim_decimal(&format!("0.{}{digits}", "0".repeat(zeroes)))
    };
    if negative {
        text.insert(0, '-');
    }
    if culture.decimal_separator != '.' {
        text = text.replace('.', &culture.decimal_separator.to_string());
    }
    text
}

fn trim_decimal(value: &str) -> String {
    if !value.contains('.') {
        return value.to_owned();
    }
    let trimmed = value.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn parse_point(culture: &DesignCulture, value: &str) -> Result<Point> {
    let values = components(culture, value, 2)?;
    Ok(Point::new(parse_i32(values[0])?, parse_i32(values[1])?))
}

fn parse_vector2(culture: &DesignCulture, value: &str) -> Result<Vector2> {
    let values = components(culture, value, 2)?;
    Ok(Vector2::from_x_and_y(
        parse_f32(culture, values[0])?,
        parse_f32(culture, values[1])?,
    ))
}

fn parse_vector3(culture: &DesignCulture, value: &str) -> Result<Vector3> {
    let values = components(culture, value, 3)?;
    Ok(Vector3::from_x_and_y_and_z(
        parse_f32(culture, values[0])?,
        parse_f32(culture, values[1])?,
        parse_f32(culture, values[2])?,
    ))
}

fn parse_vector4(culture: &DesignCulture, value: &str) -> Result<Vector4> {
    let values = components(culture, value, 4)?;
    Ok(Vector4::from_x_and_y_and_z_and_w(
        parse_f32(culture, values[0])?,
        parse_f32(culture, values[1])?,
        parse_f32(culture, values[2])?,
        parse_f32(culture, values[3])?,
    ))
}

fn parse_quaternion(culture: &DesignCulture, value: &str) -> Result<Quaternion> {
    let value = parse_vector4(culture, value)?;
    Ok(Quaternion::from_x_and_y_and_z_and_w(
        value.X, value.Y, value.Z, value.W,
    ))
}

fn parse_color(culture: &DesignCulture, value: &str) -> Result<Color> {
    let values = components(culture, value, 4)?;
    Ok(
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
            i32::from(parse_u8(values[0])?),
            i32::from(parse_u8(values[1])?),
            i32::from(parse_u8(values[2])?),
            i32::from(parse_u8(values[3])?),
        ),
    )
}

macro_rules! unsupported_parser {
    ($($name:ident => $target:ty),+ $(,)?) => {$ (
        fn $name(_culture: &DesignCulture, _value: &str) -> Result<$target> {
            Err(unsupported_conversion())
        }
    )+ };
}

unsupported_parser!(
    parse_rectangle => Rectangle,
    parse_matrix => Matrix,
    parse_bounding_box => BoundingBox,
    parse_bounding_sphere => BoundingSphere,
    parse_plane => Plane,
    parse_ray => Ray,
);

fn format_point(value: &Point, culture: &DesignCulture) -> String {
    format_values(culture, &[value.X.to_string(), value.Y.to_string()])
}

fn format_vector2(value: &Vector2, culture: &DesignCulture) -> String {
    format_values(
        culture,
        &[format_f32(value.X, culture), format_f32(value.Y, culture)],
    )
}

fn format_vector3(value: &Vector3, culture: &DesignCulture) -> String {
    format_values(
        culture,
        &[
            format_f32(value.X, culture),
            format_f32(value.Y, culture),
            format_f32(value.Z, culture),
        ],
    )
}

fn format_vector4(value: &Vector4, culture: &DesignCulture) -> String {
    format_values(
        culture,
        &[
            format_f32(value.X, culture),
            format_f32(value.Y, culture),
            format_f32(value.Z, culture),
            format_f32(value.W, culture),
        ],
    )
}

fn format_quaternion(value: &Quaternion, culture: &DesignCulture) -> String {
    format_values(
        culture,
        &[
            format_f32(value.X, culture),
            format_f32(value.Y, culture),
            format_f32(value.Z, culture),
            format_f32(value.W, culture),
        ],
    )
}

fn format_color(value: &Color, culture: &DesignCulture) -> String {
    format_values(
        culture,
        &[
            value.R().to_string(),
            value.G().to_string(),
            value.B().to_string(),
            value.A().to_string(),
        ],
    )
}

macro_rules! fallback_formatter {
    ($($name:ident => $target:ty),+ $(,)?) => {$ (
        fn $name(value: &$target, _culture: &DesignCulture) -> String {
            value.ToString()
        }
    )+ };
}

fallback_formatter!(
    format_rectangle => Rectangle,
    format_matrix => Matrix,
    format_bounding_box => BoundingBox,
    format_bounding_sphere => BoundingSphere,
    format_plane => Plane,
    format_ray => Ray,
);

fn decompose_point(value: &Point) -> Vec<DesignValue> {
    vec![DesignValue::Int32(value.X), DesignValue::Int32(value.Y)]
}

fn decompose_rectangle(value: &Rectangle) -> Vec<DesignValue> {
    vec![
        DesignValue::Int32(value.X),
        DesignValue::Int32(value.Y),
        DesignValue::Int32(value.Width),
        DesignValue::Int32(value.Height),
    ]
}

fn decompose_vector2(value: &Vector2) -> Vec<DesignValue> {
    vec![DesignValue::Single(value.X), DesignValue::Single(value.Y)]
}

fn decompose_vector3(value: &Vector3) -> Vec<DesignValue> {
    vec![
        DesignValue::Single(value.X),
        DesignValue::Single(value.Y),
        DesignValue::Single(value.Z),
    ]
}

fn decompose_vector4(value: &Vector4) -> Vec<DesignValue> {
    vec![
        DesignValue::Single(value.X),
        DesignValue::Single(value.Y),
        DesignValue::Single(value.Z),
        DesignValue::Single(value.W),
    ]
}

fn decompose_quaternion(value: &Quaternion) -> Vec<DesignValue> {
    vec![
        DesignValue::Single(value.X),
        DesignValue::Single(value.Y),
        DesignValue::Single(value.Z),
        DesignValue::Single(value.W),
    ]
}

fn decompose_color(value: &Color) -> Vec<DesignValue> {
    vec![
        DesignValue::Byte(value.R()),
        DesignValue::Byte(value.G()),
        DesignValue::Byte(value.B()),
        DesignValue::Byte(value.A()),
    ]
}

fn decompose_matrix(value: &Matrix) -> Vec<DesignValue> {
    vec![
        DesignValue::Vector3(value.Translation()),
        DesignValue::Single(value.M11),
        DesignValue::Single(value.M12),
        DesignValue::Single(value.M13),
        DesignValue::Single(value.M14),
        DesignValue::Single(value.M21),
        DesignValue::Single(value.M22),
        DesignValue::Single(value.M23),
        DesignValue::Single(value.M24),
        DesignValue::Single(value.M31),
        DesignValue::Single(value.M32),
        DesignValue::Single(value.M33),
        DesignValue::Single(value.M34),
        DesignValue::Single(value.M41),
        DesignValue::Single(value.M42),
        DesignValue::Single(value.M43),
        DesignValue::Single(value.M44),
    ]
}

fn decompose_bounding_box(value: &BoundingBox) -> Vec<DesignValue> {
    vec![
        DesignValue::Vector3(value.Min),
        DesignValue::Vector3(value.Max),
    ]
}

fn decompose_bounding_sphere(value: &BoundingSphere) -> Vec<DesignValue> {
    vec![
        DesignValue::Vector3(value.Center),
        DesignValue::Single(value.Radius),
    ]
}

fn decompose_plane(value: &Plane) -> Vec<DesignValue> {
    vec![
        DesignValue::Vector3(value.Normal),
        DesignValue::Single(value.D),
    ]
}

fn decompose_ray(value: &Ray) -> Vec<DesignValue> {
    vec![
        DesignValue::Vector3(value.Position),
        DesignValue::Vector3(value.Direction),
    ]
}

fn properties(values: Option<&[DesignPropertyValue]>) -> Result<&[DesignPropertyValue]> {
    values.ok_or(CnaError::InvalidInput(
        "XNA Design property values must not be null",
    ))
}

fn required<'a>(values: &'a [DesignPropertyValue], name: &str) -> Result<&'a DesignValue> {
    let mut matches = values.iter().filter(|value| value.Name() == name);
    let value = matches.next().ok_or(CnaError::InvalidInput(
        "a required XNA Design property is missing",
    ))?;
    if matches.next().is_some() {
        return Err(CnaError::InvalidInput(
            "an XNA Design property name must not occur more than once",
        ));
    }
    if matches!(value.Value(), DesignValue::Null) {
        return Err(CnaError::InvalidInput(
            "a required XNA Design property must not be null",
        ));
    }
    Ok(value.Value())
}

fn required_i32(values: &[DesignPropertyValue], name: &str) -> Result<i32> {
    match required(values, name)? {
        DesignValue::Int32(value) => Ok(*value),
        _ => Err(wrong_property_type()),
    }
}

fn required_u8(values: &[DesignPropertyValue], name: &str) -> Result<u8> {
    match required(values, name)? {
        DesignValue::Byte(value) => Ok(*value),
        _ => Err(wrong_property_type()),
    }
}

fn required_f32(values: &[DesignPropertyValue], name: &str) -> Result<f32> {
    match required(values, name)? {
        DesignValue::Single(value) => Ok(*value),
        _ => Err(wrong_property_type()),
    }
}

fn required_vector3(values: &[DesignPropertyValue], name: &str) -> Result<Vector3> {
    match required(values, name)? {
        DesignValue::Vector3(value) => Ok(*value),
        _ => Err(wrong_property_type()),
    }
}

fn create_point(values: Option<&[DesignPropertyValue]>) -> Result<Point> {
    let values = properties(values)?;
    Ok(Point::new(
        required_i32(values, "X")?,
        required_i32(values, "Y")?,
    ))
}

fn create_rectangle(values: Option<&[DesignPropertyValue]>) -> Result<Rectangle> {
    let values = properties(values)?;
    Ok(Rectangle::new(
        required_i32(values, "X")?,
        required_i32(values, "Y")?,
        required_i32(values, "Width")?,
        required_i32(values, "Height")?,
    ))
}

fn create_vector2(values: Option<&[DesignPropertyValue]>) -> Result<Vector2> {
    let values = properties(values)?;
    Ok(Vector2::from_x_and_y(
        required_f32(values, "X")?,
        required_f32(values, "Y")?,
    ))
}

fn create_vector3(values: Option<&[DesignPropertyValue]>) -> Result<Vector3> {
    let values = properties(values)?;
    Ok(Vector3::from_x_and_y_and_z(
        required_f32(values, "X")?,
        required_f32(values, "Y")?,
        required_f32(values, "Z")?,
    ))
}

fn create_vector4(values: Option<&[DesignPropertyValue]>) -> Result<Vector4> {
    let values = properties(values)?;
    Ok(Vector4::from_x_and_y_and_z_and_w(
        required_f32(values, "X")?,
        required_f32(values, "Y")?,
        required_f32(values, "Z")?,
        required_f32(values, "W")?,
    ))
}

fn create_quaternion(values: Option<&[DesignPropertyValue]>) -> Result<Quaternion> {
    let values = properties(values)?;
    Ok(Quaternion::from_x_and_y_and_z_and_w(
        required_f32(values, "X")?,
        required_f32(values, "Y")?,
        required_f32(values, "Z")?,
        required_f32(values, "W")?,
    ))
}

fn create_color(values: Option<&[DesignPropertyValue]>) -> Result<Color> {
    let values = properties(values)?;
    Ok(
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
            i32::from(required_u8(values, "R")?),
            i32::from(required_u8(values, "G")?),
            i32::from(required_u8(values, "B")?),
            i32::from(required_u8(values, "A")?),
        ),
    )
}

fn create_matrix(values: Option<&[DesignPropertyValue]>) -> Result<Matrix> {
    let values = properties(values)?;
    Ok(Matrix::new(
        required_f32(values, "M11")?,
        required_f32(values, "M12")?,
        required_f32(values, "M13")?,
        required_f32(values, "M14")?,
        required_f32(values, "M21")?,
        required_f32(values, "M22")?,
        required_f32(values, "M23")?,
        required_f32(values, "M24")?,
        required_f32(values, "M31")?,
        required_f32(values, "M32")?,
        required_f32(values, "M33")?,
        required_f32(values, "M34")?,
        required_f32(values, "M41")?,
        required_f32(values, "M42")?,
        required_f32(values, "M43")?,
        required_f32(values, "M44")?,
    ))
}

fn create_bounding_box(values: Option<&[DesignPropertyValue]>) -> Result<BoundingBox> {
    let values = properties(values)?;
    Ok(BoundingBox::new(
        required_vector3(values, "Min")?,
        required_vector3(values, "Max")?,
    ))
}

fn create_bounding_sphere(values: Option<&[DesignPropertyValue]>) -> Result<BoundingSphere> {
    let values = properties(values)?;
    let radius = required_f32(values, "Radius")?;
    validate_radius(radius)?;
    Ok(BoundingSphere::new(
        required_vector3(values, "Center")?,
        radius,
    ))
}

fn create_plane(values: Option<&[DesignPropertyValue]>) -> Result<Plane> {
    let values = properties(values)?;
    Ok(Plane::from_normal_and_d(
        required_vector3(values, "Normal")?,
        required_f32(values, "D")?,
    ))
}

fn create_ray(values: Option<&[DesignPropertyValue]>) -> Result<Ray> {
    let values = properties(values)?;
    Ok(Ray::new(
        required_vector3(values, "Position")?,
        required_vector3(values, "Direction")?,
    ))
}

macro_rules! descriptor {
    ($name:ident, $target:ty, $constructor:expr, $decompose:path) => {
        fn $name(value: &$target) -> DesignInstanceDescriptor {
            DesignInstanceDescriptor::complete($constructor, $decompose(value))
        }
    };
}

descriptor!(
    descriptor_point,
    Point,
    DesignConstructor::PointInt32Int32,
    decompose_point
);
descriptor!(
    descriptor_rectangle,
    Rectangle,
    DesignConstructor::RectangleInt32Int32Int32Int32,
    decompose_rectangle
);
descriptor!(
    descriptor_vector2,
    Vector2,
    DesignConstructor::Vector2SingleSingle,
    decompose_vector2
);
descriptor!(
    descriptor_vector3,
    Vector3,
    DesignConstructor::Vector3SingleSingleSingle,
    decompose_vector3
);
descriptor!(
    descriptor_vector4,
    Vector4,
    DesignConstructor::Vector4SingleSingleSingleSingle,
    decompose_vector4
);
descriptor!(
    descriptor_quaternion,
    Quaternion,
    DesignConstructor::QuaternionSingleSingleSingleSingle,
    decompose_quaternion
);
descriptor!(
    descriptor_color,
    Color,
    DesignConstructor::ColorInt32Int32Int32Int32,
    decompose_color
);
descriptor!(
    descriptor_bounding_box,
    BoundingBox,
    DesignConstructor::BoundingBoxVector3Vector3,
    decompose_bounding_box
);
descriptor!(
    descriptor_bounding_sphere,
    BoundingSphere,
    DesignConstructor::BoundingSphereVector3Single,
    decompose_bounding_sphere
);
descriptor!(
    descriptor_plane,
    Plane,
    DesignConstructor::PlaneVector3Single,
    decompose_plane
);
descriptor!(
    descriptor_ray,
    Ray,
    DesignConstructor::RayVector3Vector3,
    decompose_ray
);

fn descriptor_matrix(value: &Matrix) -> DesignInstanceDescriptor {
    DesignInstanceDescriptor::complete(
        DesignConstructor::MatrixSixteenSingles,
        decompose_matrix(value).into_iter().skip(1).collect(),
    )
}

fn fallback_string(value: &DesignValue) -> Result<String> {
    match value {
        DesignValue::Null => Err(wrong_converter_value()),
        DesignValue::String(value) => Ok(value.clone()),
        DesignValue::Int32(value) => Ok(value.to_string()),
        DesignValue::Byte(value) => Ok(value.to_string()),
        DesignValue::Single(value) => Ok(value.to_string()),
        DesignValue::Point(value) => Ok(value.ToString()),
        DesignValue::Rectangle(value) => Ok(value.ToString()),
        DesignValue::Vector2(value) => Ok(value.ToString()),
        DesignValue::Vector3(value) => Ok(value.ToString()),
        DesignValue::Vector4(value) => Ok(value.ToString()),
        DesignValue::Quaternion(value) => Ok(value.ToString()),
        DesignValue::Color(value) => Ok(value.ToString()),
        DesignValue::Matrix(value) => Ok(value.ToString()),
        DesignValue::BoundingBox(value) => Ok(value.ToString()),
        DesignValue::BoundingSphere(value) => Ok(value.ToString()),
        DesignValue::Plane(value) => Ok(value.ToString()),
        DesignValue::Ray(value) => Ok(value.ToString()),
    }
}

fn argument_i32(arguments: &[DesignValue], index: usize) -> Result<i32> {
    match arguments.get(index) {
        Some(DesignValue::Int32(value)) => Ok(*value),
        _ => Err(invalid_descriptor()),
    }
}

fn argument_u8(arguments: &[DesignValue], index: usize) -> Result<u8> {
    match arguments.get(index) {
        Some(DesignValue::Byte(value)) => Ok(*value),
        _ => Err(invalid_descriptor()),
    }
}

fn argument_f32(arguments: &[DesignValue], index: usize) -> Result<f32> {
    match arguments.get(index) {
        Some(DesignValue::Single(value)) => Ok(*value),
        _ => Err(invalid_descriptor()),
    }
}

fn argument_vector3(arguments: &[DesignValue], index: usize) -> Result<Vector3> {
    match arguments.get(index) {
        Some(DesignValue::Vector3(value)) => Ok(*value),
        _ => Err(invalid_descriptor()),
    }
}

const fn constructor_argument_count(constructor: DesignConstructor) -> usize {
    match constructor {
        DesignConstructor::PointInt32Int32
        | DesignConstructor::Vector2SingleSingle
        | DesignConstructor::BoundingBoxVector3Vector3
        | DesignConstructor::BoundingSphereVector3Single
        | DesignConstructor::PlaneVector3Single
        | DesignConstructor::RayVector3Vector3 => 2,
        DesignConstructor::Vector3SingleSingleSingle => 3,
        DesignConstructor::RectangleInt32Int32Int32Int32
        | DesignConstructor::Vector4SingleSingleSingleSingle
        | DesignConstructor::QuaternionSingleSingleSingleSingle
        | DesignConstructor::ColorInt32Int32Int32Int32 => 4,
        DesignConstructor::MatrixSixteenSingles => 16,
    }
}

fn require_argument_count(arguments: &[DesignValue], expected: usize) -> Result<()> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(invalid_descriptor())
    }
}

fn validate_radius(radius: f32) -> Result<()> {
    if radius < 0.0 {
        Err(CnaError::InvalidInput(
            "an XNA BoundingSphere radius must be nonnegative",
        ))
    } else {
        Ok(())
    }
}

const fn invalid_component_string() -> CnaError {
    CnaError::InvalidInput("invalid XNA Design component string")
}

const fn unsupported_conversion() -> CnaError {
    CnaError::InvalidInput("the requested XNA Design conversion is not supported")
}

const fn wrong_converter_value() -> CnaError {
    CnaError::InvalidInput("the XNA Design converter received a null or incompatible value")
}

const fn null_destination_type() -> CnaError {
    CnaError::InvalidInput("the XNA Design destination type must not be null")
}

const fn wrong_property_type() -> CnaError {
    CnaError::InvalidInput("an XNA Design property has an incompatible value type")
}

const fn invalid_descriptor() -> CnaError {
    CnaError::InvalidInput("the XNA Design reconstruction descriptor is invalid")
}
