#![allow(non_snake_case)]

use cna::Microsoft::Xna::Framework::Design::{
    BoundingBoxConverter, BoundingSphereConverter, ColorConverter, MathTypeConverter,
    MatrixConverter, PlaneConverter, PointConverter, QuaternionConverter, RayConverter,
    RectangleConverter, Vector2Converter, Vector3Converter, Vector4Converter,
};
use cna::Microsoft::Xna::Framework::{
    BoundingBox, BoundingSphere, Color, Matrix, Plane, Point, Quaternion, Ray, Rectangle, Vector2,
    Vector3, Vector4,
};
use cna::{
    DesignConstructor, DesignConversion, DesignCulture, DesignPropertyDescriptor,
    DesignPropertyValue, DesignType, DesignValue, MathTypeConverterBase,
};

fn property(name: &str, value: DesignValue) -> DesignPropertyValue {
    DesignPropertyValue::new(name, value)
}

fn names(properties: &[DesignPropertyDescriptor]) -> Vec<&str> {
    properties
        .iter()
        .map(DesignPropertyDescriptor::Name)
        .collect()
}

fn types(properties: &[DesignPropertyDescriptor]) -> Vec<DesignType> {
    properties
        .iter()
        .map(DesignPropertyDescriptor::ValueType)
        .collect()
}

fn text(conversion: DesignConversion) -> String {
    match conversion {
        DesignConversion::String(value) => value,
        DesignConversion::InstanceDescriptor(_) => panic!("expected a string conversion"),
    }
}

fn round_trip(converter: &dyn MathTypeConverterBase, value: DesignValue) {
    let conversion = converter
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(&value),
            Some(DesignType::InstanceDescriptor),
        )
        .unwrap();
    let descriptor = match conversion {
        DesignConversion::InstanceDescriptor(value) => value,
        DesignConversion::String(_) => panic!("expected a reconstruction descriptor"),
    };
    assert!(descriptor.IsComplete());
    assert_eq!(descriptor.Invoke().unwrap(), value);
}

#[test]
fn mapped_base_contract_and_string_support_match_xna() {
    let base = MathTypeConverter::new();
    assert!(base.CanConvertFrom(DesignType::String));
    assert!(!base.CanConvertFrom(DesignType::Int32));
    assert!(base.CanConvertTo(DesignType::String));
    assert!(base.CanConvertTo(DesignType::InstanceDescriptor));
    assert!(!base.CanConvertTo(DesignType::Int32));
    assert!(base.GetCreateInstanceSupported());
    assert!(base.GetPropertiesSupported());
    assert!(base.GetProperties().is_empty());

    let string_converters: [&dyn MathTypeConverterBase; 6] = [
        &PointConverter::new(),
        &Vector2Converter::new(),
        &Vector3Converter::new(),
        &Vector4Converter::new(),
        &QuaternionConverter::new(),
        &ColorConverter::new(),
    ];
    for converter in string_converters {
        assert!(converter.CanConvertFrom(DesignType::String));
        assert!(!converter.CanConvertFrom(DesignType::Int32));
        assert!(converter.CanConvertTo(DesignType::String));
        assert!(converter.CanConvertTo(DesignType::InstanceDescriptor));
        assert!(converter.GetCreateInstanceSupported());
        assert!(converter.GetPropertiesSupported());
    }

    let non_string_converters: [&dyn MathTypeConverterBase; 6] = [
        &RectangleConverter::new(),
        &MatrixConverter::new(),
        &BoundingBoxConverter::new(),
        &BoundingSphereConverter::new(),
        &PlaneConverter::new(),
        &RayConverter::new(),
    ];
    for converter in non_string_converters {
        assert!(!converter.CanConvertFrom(DesignType::String));
        assert!(converter.CanConvertTo(DesignType::String));
        assert!(converter.CanConvertTo(DesignType::InstanceDescriptor));
        assert!(converter
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String("1, 2".to_owned())),
            )
            .is_err());
    }
}

#[test]
fn ordered_property_metadata_matches_xna_descriptors() {
    let point = PointConverter::new();
    assert_eq!(names(point.GetProperties()), ["X", "Y"]);
    assert_eq!(types(point.GetProperties()), [DesignType::Int32; 2]);

    let rectangle = RectangleConverter::new();
    assert_eq!(
        names(rectangle.GetProperties()),
        ["X", "Y", "Width", "Height"]
    );
    assert_eq!(types(rectangle.GetProperties()), [DesignType::Int32; 4]);

    let vector2 = Vector2Converter::new();
    assert_eq!(names(vector2.GetProperties()), ["X", "Y"]);
    assert_eq!(types(vector2.GetProperties()), [DesignType::Single; 2]);
    let vector3 = Vector3Converter::new();
    assert_eq!(names(vector3.GetProperties()), ["X", "Y", "Z"]);
    assert_eq!(types(vector3.GetProperties()), [DesignType::Single; 3]);
    let vector4 = Vector4Converter::new();
    assert_eq!(names(vector4.GetProperties()), ["X", "Y", "Z", "W"]);
    assert_eq!(types(vector4.GetProperties()), [DesignType::Single; 4]);
    let quaternion = QuaternionConverter::new();
    assert_eq!(names(quaternion.GetProperties()), ["X", "Y", "Z", "W"]);
    assert_eq!(types(quaternion.GetProperties()), [DesignType::Single; 4]);

    let color = ColorConverter::new();
    assert_eq!(names(color.GetProperties()), ["R", "G", "B", "A"]);
    assert_eq!(types(color.GetProperties()), [DesignType::Byte; 4]);

    let matrix = MatrixConverter::new();
    assert_eq!(
        names(matrix.GetProperties()),
        [
            "Translation",
            "M11",
            "M12",
            "M13",
            "M14",
            "M21",
            "M22",
            "M23",
            "M24",
            "M31",
            "M32",
            "M33",
            "M34",
            "M41",
            "M42",
            "M43",
            "M44",
        ]
    );
    assert_eq!(matrix.GetProperties()[0].ValueType(), DesignType::Vector3);
    assert!(matrix.GetProperties()[1..]
        .iter()
        .all(|value| value.ValueType() == DesignType::Single));

    let box_converter = BoundingBoxConverter::new();
    assert_eq!(names(box_converter.GetProperties()), ["Min", "Max"]);
    assert_eq!(
        types(box_converter.GetProperties()),
        [DesignType::Vector3; 2]
    );
    let sphere = BoundingSphereConverter::new();
    assert_eq!(names(sphere.GetProperties()), ["Center", "Radius"]);
    assert_eq!(
        types(sphere.GetProperties()),
        [DesignType::Vector3, DesignType::Single]
    );
    let plane = PlaneConverter::new();
    assert_eq!(names(plane.GetProperties()), ["Normal", "D"]);
    assert_eq!(
        types(plane.GetProperties()),
        [DesignType::Vector3, DesignType::Single]
    );
    let ray = RayConverter::new();
    assert_eq!(names(ray.GetProperties()), ["Position", "Direction"]);
    assert_eq!(types(ray.GetProperties()), [DesignType::Vector3; 2]);
}

#[test]
fn strings_preserve_xna_culture_and_binary32_behavior() {
    let point = PointConverter::new();
    assert_eq!(
        text(
            point
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::Point(Point::new(1, -2))),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1, -2"
    );
    assert_eq!(
        text(
            point
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&DesignValue::Point(Point::new(1, -2))),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1; -2"
    );
    assert_eq!(
        point
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String(
                    " 2147483647 , -2147483648 ".to_owned(),
                )),
            )
            .unwrap(),
        Point::new(i32::MAX, i32::MIN)
    );

    let vector = Vector3Converter::new();
    let ordinary = DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.25, -2.5, 3.75));
    assert_eq!(
        text(
            vector
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&ordinary),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1.25, -2.5, 3.75"
    );
    assert_eq!(
        text(
            vector
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&ordinary),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1,25; -2,5; 3,75"
    );

    let parsed = vector
        .ConvertFrom(
            &DesignCulture::Invariant,
            Some(&DesignValue::String("-0, 1e-30, 3.40282347E+38".to_owned())),
        )
        .unwrap();
    assert_eq!(
        [parsed.X.to_bits(), parsed.Y.to_bits(), parsed.Z.to_bits()],
        [0x8000_0000, 0x0da2_4260, 0x7f7f_ffff]
    );
    let german = vector
        .ConvertFrom(
            &DesignCulture::DeDe,
            Some(&DesignValue::String(
                "NaN; +unendlich; -unendlich".to_owned(),
            )),
        )
        .unwrap();
    assert!(german.X.is_nan());
    assert_eq!(german.Y, f32::INFINITY);
    assert_eq!(german.Z, f32::NEG_INFINITY);

    let special = DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -0.0,
    ));
    let vector4 = Vector4Converter::new();
    assert_eq!(
        text(
            vector4
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&special),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "NaN, Infinity, -Infinity, 0"
    );
    assert_eq!(
        text(
            vector4
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&special),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "NaN; +unendlich; -unendlich; 0"
    );

    let extremes = DesignValue::Vector2(Vector2::from_x_and_y(1.0e-30, f32::MAX));
    assert_eq!(
        text(
            Vector2Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&extremes),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1E-30, 3.402823E+38"
    );
    assert_eq!(
        text(
            Vector2Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::Vector2(Vector2::from_x_and_y(
                        f32::from_bits(1),
                        f32::MIN_POSITIVE,
                    ))),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1.401298E-45, 1.175494E-38"
    );
    assert_eq!(
        text(
            Vector4Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(
                        1.234_567_2,
                        1.234_567_8,
                        9_999_999.0,
                        10_000_000.0,
                    ))),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "1.234567, 1.234568, 9999999, 1E+07"
    );

    let color = ColorConverter::new();
    let color_value = DesignValue::Color(
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(0, 255, 10, 40),
    );
    assert_eq!(
        text(
            color
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&color_value),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "0, 255, 10, 40"
    );
    assert_eq!(
        color
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String("0,255,10,40".to_owned())),
            )
            .unwrap(),
        match color_value {
            DesignValue::Color(value) => value,
            _ => unreachable!(),
        }
    );
}

#[test]
fn malformed_strings_nulls_wrong_types_and_overflow_fail() {
    let vector = Vector3Converter::new();
    for value in ["", "1,2", "1,2,3,4", "1,,3", "3.5e38,0,0", "NaN,0"] {
        assert!(vector
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String(value.to_owned())),
            )
            .is_err());
    }
    assert!(vector
        .ConvertFrom(
            &DesignCulture::DeDe,
            Some(&DesignValue::String("1.5; -2.25; 3.75".to_owned())),
        )
        .is_err());
    assert!(vector.ConvertFrom(&DesignCulture::Invariant, None).is_err());
    assert!(vector
        .ConvertFrom(&DesignCulture::Invariant, Some(&DesignValue::Int32(3)),)
        .is_err());
    assert!(vector
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(&DesignValue::Vector3(Vector3::Zero)),
            None,
        )
        .is_err());
    assert!(vector
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(&DesignValue::Vector3(Vector3::Zero)),
            Some(DesignType::Int32),
        )
        .is_err());

    for value in ["2147483648,0", "1.0,2"] {
        assert!(PointConverter::new()
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String(value.to_owned())),
            )
            .is_err());
    }
    for value in ["-1,0,0,0", "256,0,0,0", "Red", ""] {
        assert!(ColorConverter::new()
            .ConvertFrom(
                &DesignCulture::Invariant,
                Some(&DesignValue::String(value.to_owned())),
            )
            .is_err());
    }
}

#[test]
fn unsupported_string_input_uses_xna_fallback_output() {
    let rectangle = Rectangle::new(1, 2, 3, 4);
    assert_eq!(
        text(
            RectangleConverter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&DesignValue::Rectangle(rectangle)),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "{X:1 Y:2 Width:3 Height:4}"
    );
    assert_eq!(
        text(
            MatrixConverter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&DesignValue::Matrix(Matrix::Identity)),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        Matrix::Identity.ToString()
    );
    assert_eq!(
        text(
            BoundingBoxConverter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&DesignValue::BoundingBox(BoundingBox::new(
                        Vector3::new(1.0),
                        Vector3::new(2.0),
                    ))),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "{Min:{X:1 Y:1 Z:1} Max:{X:2 Y:2 Z:2}}"
    );

    assert_eq!(
        text(
            Vector3Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::Point(Point::Zero)),
                    Some(DesignType::String),
                )
                .unwrap(),
        ),
        "{X:0 Y:0}"
    );
}

#[test]
fn create_instance_reconstructs_every_shape_and_validates_properties() {
    assert_eq!(
        PointConverter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Int32(1)),
                property("Y", DesignValue::Int32(2)),
                property("Extra", DesignValue::Int32(3)),
            ]))
            .unwrap(),
        Point::new(1, 2)
    );
    assert_eq!(
        RectangleConverter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Int32(1)),
                property("Y", DesignValue::Int32(2)),
                property("Width", DesignValue::Int32(3)),
                property("Height", DesignValue::Int32(4)),
            ]))
            .unwrap(),
        Rectangle::new(1, 2, 3, 4)
    );
    assert_eq!(
        Vector2Converter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Single(1.0)),
                property("Y", DesignValue::Single(2.0)),
            ]))
            .unwrap(),
        Vector2::from_x_and_y(1.0, 2.0)
    );
    assert_eq!(
        Vector3Converter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Single(1.0)),
                property("Y", DesignValue::Single(2.0)),
                property("Z", DesignValue::Single(3.0)),
                property("Extra", DesignValue::Single(4.0)),
            ]))
            .unwrap(),
        Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)
    );
    assert_eq!(
        Vector4Converter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Single(1.0)),
                property("Y", DesignValue::Single(2.0)),
                property("Z", DesignValue::Single(3.0)),
                property("W", DesignValue::Single(4.0)),
            ]))
            .unwrap(),
        Vector4::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0)
    );
    assert_eq!(
        QuaternionConverter::new()
            .CreateInstance(Some(&[
                property("X", DesignValue::Single(1.0)),
                property("Y", DesignValue::Single(2.0)),
                property("Z", DesignValue::Single(3.0)),
                property("W", DesignValue::Single(4.0)),
            ]))
            .unwrap(),
        Quaternion::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0)
    );
    let rgba =
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(10, 20, 30, 40);
    assert_eq!(
        ColorConverter::new()
            .CreateInstance(Some(&[
                property("R", DesignValue::Byte(10)),
                property("G", DesignValue::Byte(20)),
                property("B", DesignValue::Byte(30)),
                property("A", DesignValue::Byte(40)),
            ]))
            .unwrap(),
        rgba
    );
    assert_eq!(
        BoundingBoxConverter::new()
            .CreateInstance(Some(&[
                property("Min", DesignValue::Vector3(Vector3::new(1.0))),
                property("Max", DesignValue::Vector3(Vector3::new(2.0))),
            ]))
            .unwrap(),
        BoundingBox::new(Vector3::new(1.0), Vector3::new(2.0))
    );
    assert_eq!(
        BoundingSphereConverter::new()
            .CreateInstance(Some(&[
                property(
                    "Center",
                    DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)),
                ),
                property("Radius", DesignValue::Single(4.0)),
            ]))
            .unwrap(),
        BoundingSphere::new(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0), 4.0)
    );
    assert_eq!(
        PlaneConverter::new()
            .CreateInstance(Some(&[
                property(
                    "Normal",
                    DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)),
                ),
                property("D", DesignValue::Single(4.0)),
            ]))
            .unwrap(),
        Plane::from_normal_and_d(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0), 4.0)
    );
    assert_eq!(
        RayConverter::new()
            .CreateInstance(Some(&[
                property(
                    "Position",
                    DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)),
                ),
                property(
                    "Direction",
                    DesignValue::Vector3(Vector3::from_x_and_y_and_z(4.0, 5.0, 6.0)),
                ),
            ]))
            .unwrap(),
        Ray::new(
            Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0),
            Vector3::from_x_and_y_and_z(4.0, 5.0, 6.0),
        )
    );

    let vector = Vector3Converter::new();
    assert!(vector.CreateInstance(None).is_err());
    assert!(vector
        .CreateInstance(Some(&[
            property("X", DesignValue::Single(1.0)),
            property("Y", DesignValue::Single(2.0)),
        ]))
        .is_err());
    assert!(vector
        .CreateInstance(Some(&[
            property("X", DesignValue::Int32(1)),
            property("Y", DesignValue::Single(2.0)),
            property("Z", DesignValue::Single(3.0)),
        ]))
        .is_err());
    assert!(vector
        .CreateInstance(Some(&[
            property("X", DesignValue::Single(1.0)),
            property("Y", DesignValue::Null),
            property("Z", DesignValue::Single(3.0)),
        ]))
        .is_err());
    assert!(vector
        .CreateInstance(Some(&[
            property("X", DesignValue::Single(1.0)),
            property("X", DesignValue::Single(1.0)),
            property("Y", DesignValue::Single(2.0)),
            property("Z", DesignValue::Single(3.0)),
        ]))
        .is_err());
}

#[test]
fn matrix_uses_translation_for_properties_but_not_reconstruction() {
    let values: Vec<_> = (1..=16)
        .enumerate()
        .map(|(index, value)| {
            let row = index / 4 + 1;
            let column = index % 4 + 1;
            property(
                &format!("M{row}{column}"),
                DesignValue::Single(value as f32),
            )
        })
        .chain([property(
            "Translation",
            DesignValue::Vector3(Vector3::from_x_and_y_and_z(100.0, 200.0, 300.0)),
        )])
        .collect();
    let matrix = MatrixConverter::new()
        .CreateInstance(Some(&values))
        .unwrap();
    assert_eq!(
        (matrix.M11, matrix.M24, matrix.M41, matrix.M44),
        (1.0, 8.0, 13.0, 16.0)
    );
    assert_eq!(
        matrix.Translation(),
        Vector3::from_x_and_y_and_z(13.0, 14.0, 15.0)
    );

    let decomposed = MatrixConverter::new()
        .GetPropertyValues(Some(&DesignValue::Matrix(matrix)))
        .unwrap();
    assert_eq!(decomposed[0].Name(), "Translation");
    assert_eq!(
        decomposed[0].Value(),
        &DesignValue::Vector3(Vector3::from_x_and_y_and_z(13.0, 14.0, 15.0))
    );
    assert_eq!(decomposed.len(), 17);
}

#[test]
fn property_decomposition_snapshots_nested_value_types() {
    let sphere = BoundingSphere::new(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0), 4.0);
    let source = DesignValue::BoundingSphere(sphere);
    let mut values = BoundingSphereConverter::new()
        .GetPropertyValues(Some(&source))
        .unwrap();
    values[0] = property(
        "Center",
        DesignValue::Vector3(Vector3::from_x_and_y_and_z(99.0, 2.0, 3.0)),
    );
    assert_eq!(
        source,
        DesignValue::BoundingSphere(BoundingSphere::new(
            Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0),
            4.0,
        ))
    );

    assert!(BoundingBoxConverter::new()
        .GetPropertyValues(Some(&DesignValue::Point(Point::Zero)))
        .is_err());
    assert!(RayConverter::new().GetPropertyValues(None).is_err());
}

#[test]
fn executable_descriptors_round_trip_all_concrete_converters() {
    round_trip(&PointConverter::new(), DesignValue::Point(Point::new(1, 2)));
    round_trip(
        &RectangleConverter::new(),
        DesignValue::Rectangle(Rectangle::new(1, 2, 3, 4)),
    );
    round_trip(
        &Vector2Converter::new(),
        DesignValue::Vector2(Vector2::from_x_and_y(1.0, 2.0)),
    );
    round_trip(
        &Vector3Converter::new(),
        DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)),
    );
    round_trip(
        &Vector4Converter::new(),
        DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0)),
    );
    round_trip(
        &QuaternionConverter::new(),
        DesignValue::Quaternion(Quaternion::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0)),
    );
    round_trip(
        &ColorConverter::new(),
        DesignValue::Color(
            Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(10, 20, 30, 40),
        ),
    );
    round_trip(
        &MatrixConverter::new(),
        DesignValue::Matrix(Matrix::Identity),
    );
    round_trip(
        &BoundingBoxConverter::new(),
        DesignValue::BoundingBox(BoundingBox::new(Vector3::new(1.0), Vector3::new(2.0))),
    );
    round_trip(
        &BoundingSphereConverter::new(),
        DesignValue::BoundingSphere(BoundingSphere::new(Vector3::new(1.0), 2.0)),
    );
    round_trip(
        &PlaneConverter::new(),
        DesignValue::Plane(Plane::from_normal_and_d(Vector3::new(1.0), 2.0)),
    );
    round_trip(
        &RayConverter::new(),
        DesignValue::Ray(Ray::new(Vector3::new(1.0), Vector3::new(2.0))),
    );

    let color = DesignValue::Color(
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(10, 20, 30, 40),
    );
    let descriptor = match ColorConverter::new()
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(&color),
            Some(DesignType::InstanceDescriptor),
        )
        .unwrap()
    {
        DesignConversion::InstanceDescriptor(value) => value,
        DesignConversion::String(_) => unreachable!(),
    };
    assert_eq!(
        descriptor.Constructor(),
        DesignConstructor::ColorInt32Int32Int32Int32
    );
    assert!(descriptor
        .Arguments()
        .iter()
        .all(|value| matches!(value, DesignValue::Byte(_))));

    let matrix = DesignValue::Matrix(Matrix::Identity);
    let descriptor = match MatrixConverter::new()
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(&matrix),
            Some(DesignType::InstanceDescriptor),
        )
        .unwrap()
    {
        DesignConversion::InstanceDescriptor(value) => value,
        DesignConversion::String(_) => unreachable!(),
    };
    assert_eq!(
        descriptor.Constructor(),
        DesignConstructor::MatrixSixteenSingles
    );
    assert_eq!(descriptor.Arguments().len(), 16);
}
