use super::color::XNA_NAMED_COLORS;
use super::{Color, MathHelper, Matrix, Rectangle, Vector2, Vector3};

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1.0e-5, "{actual} != {expected}");
}

#[test]
fn named_values_match_xna() {
    assert_eq!(Vector2::Zero, Vector2::from_x_and_y(0.0, 0.0));
    assert_eq!(
        Color::CornflowerBlue,
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(100, 149, 237, 255)
    );
    assert_eq!(XNA_NAMED_COLORS.len(), 141);
    for &(_, color, packed) in XNA_NAMED_COLORS {
        assert_eq!(color.PackedValue(), packed);
    }
}

#[test]
fn color_packing_matches_xna_edge_behavior() {
    let extreme = Color::from_r_and_g_and_b_and_a_as_single_and_single_and_single_and_single(
        0.5,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    );
    assert_eq!(extreme.PackedValue(), 0x00ff_0080);
    assert_eq!(
        Color::Lerp(Color::Transparent, Color::White, 0.5).PackedValue(),
        0x7f7f_7f7f
    );
    assert_eq!((Color::White * 0.5).PackedValue(), 0x7f7f_7f7f);
    assert_eq!(
        Color::FromNonPremultipliedWithRAndGAndBAndA(i32::MAX, i32::MAX, i32::MAX, i32::MAX,)
            .PackedValue(),
        u32::MAX
    );
    assert_eq!(Color::Transparent.PackedValue(), 0);
    assert_eq!(Color::Red.ToString(), "{R:255 G:0 B:0 A:255}");
}
#[test]
fn matrix_multiplication_is_real() {
    let m = Matrix::CreateScaleWithScale(2.0)
        * Matrix::CreateTranslation(Vector3::from_x_and_y_and_z(3.0, 4.0, 5.0));
    let p = Vector3::Transform(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0), m);
    assert_eq!(p, Vector3::from_x_and_y_and_z(5.0, 8.0, 11.0));
}
#[test]
fn rotations_follow_xna_row_vector_convention() {
    let p = Vector3::Transform(
        Vector3::Up,
        Matrix::CreateRotationX(core::f32::consts::FRAC_PI_2),
    );
    close(p.Y, 0.0);
    close(p.Z, 1.0);
}
#[test]
fn perspective_has_expected_xna_terms() {
    let p = Matrix::CreatePerspectiveFieldOfView(MathHelper::PiOver2, 2.0, 1.0, 10.0);
    close(p.M11, 0.5);
    close(p.M22, 1.0);
    close(p.M34, -1.0);
    close(p.M43, -10.0 / 9.0);
}
#[test]
fn rectangle_edges_are_half_open() {
    let r = Rectangle::new(2, 3, 4, 5);
    assert!(r.Contains(2, 3));
    assert!(!r.Contains(6, 8));
}
