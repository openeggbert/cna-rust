//! Bit-exact observations ported from the neutral CNA-C# compatibility corpus.
//!
//! The expected values in this slice were adjudicated against the pinned XNA
//! decompilation/IL. They are not inferred from FNA or `MonoGame` behavior. A
//! Windows XNA runtime snapshot remains independent future evidence.

#![allow(non_snake_case)]

use std::any::{Any, TypeId};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use cna::Microsoft::Xna::Framework::Content::{
    ContentLoadException, ContentSerializerAttribute, ContentSerializerCollectionItemNameAttribute,
    ContentSerializerRuntimeTypeAttribute, ContentSerializerTypeVersionAttribute,
};
use cna::Microsoft::Xna::Framework::Design::{
    BoundingBoxConverter, BoundingSphereConverter, ColorConverter, MathTypeConverter,
    MatrixConverter, PlaneConverter, PointConverter, QuaternionConverter, RayConverter,
    RectangleConverter, Vector2Converter, Vector3Converter, Vector4Converter,
};
use cna::Microsoft::Xna::Framework::Audio::{
    AudioChannels, AudioEmitter, AudioListener, AudioStopOptions, MicrophoneState, SoundEffect,
    SoundState,
};
use cna::Microsoft::Xna::Framework::Graphics::PackedVector::{
    Alpha8, Bgra5551, Byte4, HalfSingle, NormalizedByte2, Short2,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    Blend, BlendState, BufferUsage, CompareFunction, CullMode, DepthFormat, DepthStencilState,
    GraphicsProfile, IndexElementSize, PresentInterval, PresentationParameters, PrimitiveType,
    RasterizerState, RenderTargetUsage, SamplerState, SetDataOptions, SurfaceFormat,
    TextureAddressMode, TextureFilter, VertexDeclaration, VertexElement, VertexElementFormat,
    VertexElementUsage, VertexPositionColor, VertexPositionColorTexture,
    VertexPositionNormalTexture, VertexPositionTexture, Viewport,
};
use cna::Microsoft::Xna::Framework::Input::Touch::{
    GestureSample, GestureType, TouchCollection, TouchLocation, TouchLocationState,
};
use cna::Microsoft::Xna::Framework::Input::{
    ButtonState, Buttons, GamePadButtons, GamePadDPad, GamePadState, GamePadThumbSticks,
    GamePadTriggers, KeyboardState, Keys, MouseState,
};
use cna::Microsoft::Xna::Framework::Media::{
    MediaSourceType, MediaState, VideoSoundtrackType, VisualizationData,
};
use cna::Microsoft::Xna::Framework::{
    BoundingBox, BoundingFrustum, BoundingSphere, Color, Curve, CurveContinuity, CurveKey,
    CurveLoopType, CurveTangent, DisplayOrientation, DrawableGameComponent, Game, GameComponent,
    GameComponentCollection, GameComponentCollectionEventArgs, GameServiceContainer,
    GraphicsDeviceInformation, IGameComponent, MathHelper, Matrix, Plane, Point,
    PreparingDeviceSettingsEventArgs, Quaternion, Ray, Rectangle, TimeSpan, Vector2, Vector3,
    Vector4,
};
use cna::{
    DesignConstructor, DesignConversion, DesignCulture, DesignPropertyValue, DesignType,
    DesignValue, GameComponentCollectionExt, GameState, GameStateAccess, MathTypeConverterBase,
};

#[derive(Default)]
struct CorpusGame {
    state: Arc<GameState>,
}

impl GameStateAccess for CorpusGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for CorpusGame {}

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn design_properties(converter: &dyn MathTypeConverterBase) -> Vec<(&'static str, DesignType)> {
    converter
        .GetProperties()
        .iter()
        .map(|value| (value.Name(), value.ValueType()))
        .collect()
}

fn design_support(converter: &dyn MathTypeConverterBase) -> (bool, bool, bool, bool, bool, bool) {
    (
        converter.CanConvertFrom(DesignType::String),
        converter.CanConvertFrom(DesignType::Int32),
        converter.CanConvertTo(DesignType::String),
        converter.CanConvertTo(DesignType::InstanceDescriptor),
        converter.GetCreateInstanceSupported(),
        converter.GetPropertiesSupported(),
    )
}

fn design_text(value: DesignConversion) -> String {
    match value {
        DesignConversion::String(value) => value,
        DesignConversion::InstanceDescriptor(_) => panic!("expected Design string conversion"),
    }
}

fn design_constructor(
    converter: &dyn MathTypeConverterBase,
    value: &DesignValue,
) -> DesignConstructor {
    match converter
        .ConvertTo(
            &DesignCulture::Invariant,
            Some(value),
            Some(DesignType::InstanceDescriptor),
        )
        .expect("XNA Design descriptor")
    {
        DesignConversion::InstanceDescriptor(value) => value.Constructor(),
        DesignConversion::String(_) => panic!("expected Design reconstruction descriptor"),
    }
}

fn matrix_bits(value: Matrix) -> [u32; 16] {
    [
        bits(value.M11),
        bits(value.M12),
        bits(value.M13),
        bits(value.M14),
        bits(value.M21),
        bits(value.M22),
        bits(value.M23),
        bits(value.M24),
        bits(value.M31),
        bits(value.M32),
        bits(value.M33),
        bits(value.M34),
        bits(value.M41),
        bits(value.M42),
        bits(value.M43),
        bits(value.M44),
    ]
}

#[test]
#[allow(clippy::excessive_precision, clippy::too_many_lines)]
fn pinned_xna_math_observations() {
    let mut observations = 0_usize;
    macro_rules! observe {
        ($actual:expr, $expected:expr) => {{
            observations += 1;
            assert_eq!($actual, $expected);
        }};
    }

    let v2_zero = Vector2::NormalizeWithValue(Vector2::Zero);
    observe!(
        (bits(v2_zero.X), bits(v2_zero.Y)),
        (0xffc0_0000, 0xffc0_0000)
    );

    let v3_zero = Vector3::NormalizeWithValue(Vector3::Zero);
    observe!(
        (bits(v3_zero.X), bits(v3_zero.Y), bits(v3_zero.Z)),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );

    let v4_zero = Vector4::NormalizeWithVector(Vector4::Zero);
    observe!(
        (
            bits(v4_zero.X),
            bits(v4_zero.Y),
            bits(v4_zero.Z),
            bits(v4_zero.W)
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );

    observe!(
        (
            bits((Vector2::new(3.0) / 7.0).X),
            bits((Vector3::new(7.0) / 3.0).X),
            bits((Vector4::new(12_345.67) / 3.0).X)
        ),
        (0x3edb_6db8, 0x4015_5556, 0x4580_99ca)
    );

    let packed = Color::from_r_and_g_and_b_and_a_as_single_and_single_and_single_and_single(
        0.5,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    );
    observe!(packed.PackedValue(), 0x00ff_0080);
    observe!(
        Color::Lerp(Color::Transparent, Color::White, 0.5).PackedValue(),
        0x7f7f_7f7f
    );
    observe!(
        Color::FromNonPremultipliedWithRAndGAndBAndA(i32::MAX, i32::MAX, i32::MAX, i32::MAX)
            .PackedValue(),
        u32::MAX
    );

    let nan_vector = Vector2::from_x_and_y(f32::NAN, 0.0);
    observe!(
        (nan_vector.Equals(nan_vector), nan_vector == nan_vector),
        (false, false)
    );
    observe!(bits(MathHelper::Clamp(0.0, 2.0, 1.0)), 0x4000_0000);
    observe!(bits(MathHelper::WrapAngle(123_456.789)), 0xbfc2_e06c);
    observe!(
        (
            bits(MathHelper::CatmullRom(-10.0, -10.0, -10.0, -7.0, 0.3)),
            bits(MathHelper::Hermite(-10.0, -10.0, -10.0, -10.0, 1.1))
        ),
        (0xc121_8313, 0xc135_1eba)
    );
    observe!(
        MathHelper::Hermite(1.0, f32::INFINITY, 2.0, 0.0, 0.0).is_nan(),
        true
    );

    let minimum = Vector3::Min(
        Vector3::from_x_and_y_and_z(
            f32::from_bits(0xffc0_0000),
            1.0,
            f32::from_bits(0xffc0_0000),
        ),
        Vector3::from_x_and_y_and_z(
            7.0,
            f32::from_bits(0xffc0_0000),
            f32::from_bits(0xffc0_0000),
        ),
    );
    observe!(
        (bits(minimum.X), bits(minimum.Y), bits(minimum.Z)),
        (0x40e0_0000, 0xffc0_0000, 0xffc0_0000)
    );

    let clamped = Vector3::Clamp(Vector3::Zero, Vector3::new(2.0), Vector3::new(1.0));
    observe!(
        (bits(clamped.X), bits(clamped.Y), bits(clamped.Z)),
        (0x4000_0000, 0x4000_0000, 0x4000_0000)
    );
    observe!(
        Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0).GetHashCode(),
        -1_077_936_128
    );

    let source = [Vector3::Zero];
    let mut destination = [Vector3::One];
    let mut identity = Matrix::Identity;
    let negative_length = catch_unwind(AssertUnwindSafe(|| {
        Vector3::TransformWithSourceArrayAndSourceIndexAndMatrixAndDestinationArrayAndDestinationIndexAndLength(
            &source,
            0,
            &mut identity,
            &mut destination,
            0,
            -1,
        );
    }));
    observe!(negative_length.is_ok(), true);

    let negative_index = catch_unwind(AssertUnwindSafe(|| {
        Vector3::TransformWithSourceArrayAndSourceIndexAndMatrixAndDestinationArrayAndDestinationIndexAndLength(
            &source,
            -1,
            &mut identity,
            &mut destination,
            0,
            1,
        );
    }));
    observe!(negative_index.is_err(), true);
    observe!(bits((-Vector4::Zero).X), 0x8000_0000);

    observe!(bits((Matrix::Identity / 3.0).M11), 0x3eaa_aaab);
    let matrix = Matrix::CreateScale(2.0, 3.0, 4.0)
        * Matrix::CreateRotationY(0.25)
        * Matrix::CreateTranslationWithXPositionAndYPositionAndZPosition(5.0, 6.0, 7.0);
    observe!(
        matrix_bits(matrix * Matrix::Invert(matrix)),
        [
            0x3f80_0000,
            0x0000_0000,
            0xb200_0000,
            0x0000_0000,
            0x0000_0000,
            0x3f80_0000,
            0x0000_0000,
            0x0000_0000,
            0x3300_0000,
            0x0000_0000,
            0x3f80_0000,
            0x0000_0000,
            0x3400_0000,
            0x0000_0000,
            0x0000_0000,
            0x3f80_0000,
        ]
    );
    let singular = Matrix::Invert(Matrix::new(
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ));
    observe!(
        (
            bits(singular.M11),
            bits(singular.M22),
            bits(singular.M33),
            bits(singular.M44)
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );
    let mut nan_matrix = Matrix::Identity;
    nan_matrix.M11 = f32::from_bits(0xffc0_0000);
    observe!(
        (nan_matrix.Equals(nan_matrix), nan_matrix == nan_matrix),
        (false, false)
    );
    observe!(Matrix::Identity.GetHashCode(), -33_554_432);
    let large_rotation = Matrix::CreateRotationY(123_456.789);
    observe!(
        (bits(large_rotation.M11), bits(large_rotation.M31)),
        (0x3d53_e807, 0xbf7f_a83d)
    );
    let infinite_perspective = Matrix::CreatePerspective(4.0, 3.0, 0.1, f32::INFINITY);
    observe!(
        (
            bits(infinite_perspective.M33),
            bits(infinite_perspective.M43)
        ),
        (0xffc0_0000, 0xffc0_0000)
    );

    let mirrored = Matrix::CreateScale(-2.0, 3.0, 4.0)
        * Matrix::CreateRotationY(0.25)
        * Matrix::CreateTranslationWithXPositionAndYPositionAndZPosition(5.0, 6.0, 7.0);
    let mut scale = Vector3::Zero;
    let mut rotation = Quaternion::Identity;
    let mut translation = Vector3::Zero;
    let decomposed = mirrored.Decompose(&mut scale, &mut rotation, &mut translation);
    observe!(
        (
            decomposed,
            bits(scale.X),
            bits(scale.Y),
            bits(scale.Z),
            bits(rotation.X),
            bits(rotation.Y),
            bits(rotation.Z),
            bits(rotation.W),
            bits(translation.X),
            bits(translation.Y),
            bits(translation.Z),
        ),
        (
            true,
            0x4000_0000,
            0x4040_0000,
            0xc080_0000,
            0x0000_0000,
            0x3f7e_00aa,
            0x0000_0000,
            0xbdff_5579,
            0x40a0_0000,
            0x40c0_0000,
            0x40e0_0000,
        )
    );
    let billboard = Matrix::CreateConstrainedBillboard(
        Vector3::from_x_and_y_and_z(0.0, 10.0, 0.0),
        Vector3::Zero,
        Vector3::from_x_and_y_and_z(0.0, 2.0, 0.0),
        None,
        None,
    );
    observe!(
        (
            bits(billboard.M11),
            bits(billboard.M22),
            bits(billboard.M33)
        ),
        (0xbf80_0000, 0x4000_0000, 0xbf80_0000)
    );
    let shadow = Matrix::CreateShadow(
        Vector3::Forward,
        Plane::from_normal_and_d(Vector3::Zero, 0.0),
    );
    observe!((shadow.M11.is_nan(), shadow.M44.is_nan()), (true, true));
    let mut reflection_plane =
        Plane::from_normal_and_d(Vector3::from_x_and_y_and_z(2.0, 0.0, 0.0), 4.0);
    let mut reflection = Matrix::Identity;
    Matrix::CreateReflectionWithValueAndResult(&mut reflection_plane, &mut reflection);
    observe!(
        (
            bits(reflection_plane.Normal.X),
            bits(reflection_plane.D),
            bits(reflection.M11),
            bits(reflection.M41),
        ),
        (0x3f80_0000, 0x4000_0000, 0xbf80_0000, 0xc080_0000)
    );
    let degenerate_look_at = Matrix::CreateLookAt(Vector3::Zero, Vector3::Zero, Vector3::Up);
    observe!(
        matrix_bits(degenerate_look_at),
        [
            0xffc0_0000,
            0xffc0_0000,
            0xffc0_0000,
            0x0000_0000,
            0xffc0_0000,
            0xffc0_0000,
            0xffc0_0000,
            0x0000_0000,
            0xffc0_0000,
            0xffc0_0000,
            0xffc0_0000,
            0x0000_0000,
            0x7fc0_0000,
            0x7fc0_0000,
            0x7fc0_0000,
            0x3f80_0000,
        ]
    );
    observe!(bits((-Matrix::default()).M11), 0x8000_0000);
    observe!(
        Matrix::Identity.ToString(),
        "{ {M11:1 M12:0 M13:0 M14:0} {M21:0 M22:1 M23:0 M24:0} {M31:0 M32:0 M33:1 M34:0} {M41:0 M42:0 M43:0 M44:1} }"
    );

    let mut viewport = Viewport::from_x_and_y_and_width_and_height(11, 13, 640, 360);
    viewport.SetMinDepth(0.2);
    viewport.SetMaxDepth(0.9);
    let viewport_world = Matrix::CreateScale(1.5, 0.75, 2.0)
        * Matrix::CreateRotationY(0.31)
        * Matrix::CreateTranslationWithXPositionAndYPositionAndZPosition(2.0, -1.0, 0.5);
    let viewport_view = Matrix::CreateLookAt(
        Vector3::from_x_and_y_and_z(4.0, 3.0, 8.0),
        Vector3::Zero,
        Vector3::Up,
    );
    let viewport_projection = Matrix::CreatePerspectiveFieldOfView(0.9, 16.0 / 9.0, 0.1, 100.0);
    let projected = viewport.Project(
        Vector3::from_x_and_y_and_z(0.25, -0.5, 1.25),
        viewport_projection,
        viewport_view,
        viewport_world,
    );
    observe!(
        (bits(projected.X), bits(projected.Y), bits(projected.Z)),
        (0x43d4_2808, 0x43ac_9f3c, 0x3f63_aff4)
    );
    let unprojected = viewport.Unproject(
        projected,
        viewport_projection,
        viewport_view,
        viewport_world,
    );
    observe!(
        (
            bits(unprojected.X),
            bits(unprojected.Y),
            bits(unprojected.Z)
        ),
        (0x3e7f_fe10, 0xbeff_f906, 0x3fa0_0111)
    );
    let singular_unprojected = viewport.Unproject(
        Vector3::from_x_and_y_and_z(100.0, 50.0, 0.5),
        Matrix::Identity,
        Matrix::Identity,
        Matrix::default(),
    );
    observe!(
        (
            bits(singular_unprojected.X),
            bits(singular_unprojected.Y),
            bits(singular_unprojected.Z)
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );
    observe!(
        (
            Point::new(1, 2).GetHashCode(),
            Rectangle::new(1, 2, 3, 4).GetHashCode()
        ),
        (3, 10)
    );

    let unit_box = BoundingBox::new(Vector3::new(-1.0), Vector3::new(1.0));
    observe!(unit_box.ContainsWithPoint(Vector3::UnitX) as i32, 1);
    let nan_box = BoundingBox::new(
        Vector3::from_x_and_y_and_z(f32::NAN, -1.0, -1.0),
        Vector3::from_x_and_y_and_z(f32::NAN, 1.0, 1.0),
    );
    observe!(
        (
            unit_box.ContainsWithPoint(Vector3::from_x_and_y_and_z(f32::NAN, 0.0, 0.0)) as i32,
            unit_box.Intersects(nan_box),
        ),
        (0, true)
    );
    let unit_sphere = BoundingSphere::new(Vector3::Zero, 1.0);
    observe!(unit_sphere.ContainsWithPoint(Vector3::UnitX) as i32, 0);
    let points_sphere = BoundingSphere::CreateFromPoints(&[
        Vector3::from_x_and_y_and_z(-4.0, 1.0, 0.0),
        Vector3::from_x_and_y_and_z(6.0, -2.0, 3.0),
        Vector3::from_x_and_y_and_z(0.0, 8.0, -5.0),
        Vector3::from_x_and_y_and_z(2.0, 0.0, 9.0),
    ]);
    observe!(
        (
            bits(points_sphere.Center.X),
            bits(points_sphere.Center.Y),
            bits(points_sphere.Center.Z),
            bits(points_sphere.Radius),
        ),
        (0x3f80_0000, 0x4080_0000, 0x4000_0000, 0x4101_fc10)
    );
    let ray_sphere = Ray::new(Vector3::from_x_and_y_and_z(-5.0, 0.25, 0.0), Vector3::UnitX)
        .IntersectsWithSphere(unit_sphere);
    observe!(ray_sphere.map(bits), Some(0x4081_0421));
    observe!(
        unit_sphere.IntersectsWithSphere(BoundingSphere::new(
            Vector3::from_x_and_y_and_z(2.0, 0.0, 0.0),
            1.0,
        )),
        false
    );
    let near_parallel_box_ray = Ray::new(
        Vector3::from_x_and_y_and_z(2.0, 0.0, 0.0),
        Vector3::from_x_and_y_and_z(-5e-7, 0.0, 0.0),
    );
    observe!(near_parallel_box_ray.Intersects(unit_box), None);
    let near_parallel_plane_ray =
        Ray::new(Vector3::Zero, Vector3::from_x_and_y_and_z(5e-6, 1.0, 0.0));
    observe!(
        near_parallel_plane_ray.IntersectsWithPlane(Plane::from_normal_and_d(Vector3::UnitX, -1.0)),
        None
    );
    let mut behind_ray = Ray::new(Vector3::from_x_and_y_and_z(5e-6, 0.0, 0.0), Vector3::UnitX);
    let mut origin_plane = Plane::from_normal_and_d(Vector3::UnitX, 0.0);
    let by_value = behind_ray.IntersectsWithPlane(origin_plane);
    let mut by_reference = None;
    behind_ray.IntersectsWithPlaneAndResult(&mut origin_plane, &mut by_reference);
    observe!(
        (by_value.map(bits), by_reference.map(bits)),
        (Some(0), Some(0))
    );

    let degenerate_plane =
        Plane::from_point1_and_point2_and_point3(Vector3::Zero, Vector3::Zero, Vector3::Zero);
    observe!(
        (
            bits(degenerate_plane.Normal.X),
            bits(degenerate_plane.Normal.Y),
            bits(degenerate_plane.Normal.Z),
            bits(degenerate_plane.D),
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000, 0x7fc0_0000)
    );
    let near_unit_plane = Plane::NormalizeWithValue(Plane::from_normal_and_d(
        Vector3::from_x_and_y_and_z(0.6, 0.799_999_95, 0.0),
        2.0,
    ));
    observe!(
        (
            bits(near_unit_plane.Normal.X),
            bits(near_unit_plane.Normal.Y),
            bits(near_unit_plane.Normal.Z),
            bits(near_unit_plane.D),
        ),
        (0x3f19_999a, 0x3f4c_cccc, 0x0000_0000, 0x4000_0000)
    );
    observe!(
        Plane::from_normal_and_d(Vector3::Zero, 0.0).Intersects(unit_box) as i32,
        2
    );

    let frustum_projection =
        Matrix::CreatePerspectiveFieldOfView(MathHelper::PiOver4, 4.0 / 3.0, 1.0, 10.0);
    let frustum_matrix = Matrix::CreateLookAt(
        Vector3::from_x_and_y_and_z(0.0, 0.0, 5.0),
        Vector3::Zero,
        Vector3::Up,
    ) * frustum_projection;
    let frustum = BoundingFrustum::new(frustum_matrix);
    observe!(
        (
            bits(frustum.Near().Normal.X),
            bits(frustum.Near().Normal.Y),
            bits(frustum.Near().Normal.Z),
            bits(frustum.Near().D),
        ),
        (0x8000_0000, 0x8000_0000, 0x3f80_0000, 0xc080_0000)
    );
    observe!(
        (
            bits(frustum.Top().Normal.X),
            bits(frustum.Top().Normal.Y),
            bits(frustum.Top().Normal.Z),
            bits(frustum.Top().D),
        ),
        (0x0000_0000, 0x3f6c_835f, 0x3ec3_ef16, 0xbff4_eadb)
    );
    let frustum_corners = frustum.GetCorners();
    observe!(
        (
            bits(frustum_corners[0].X),
            bits(frustum_corners[0].Y),
            bits(frustum_corners[0].Z)
        ),
        (0xbf0d_6289, 0x3ed4_13cb, 0x4080_0000)
    );
    observe!(
        (
            bits(frustum_corners[6].X),
            bits(frustum_corners[6].Y),
            bits(frustum_corners[6].Z)
        ),
        (0x40b0_bb28, 0xc084_8c5d, 0xc09f_fff8)
    );
    observe!(
        (
            frustum.ContainsWithPoint(Vector3::Zero) as i32,
            frustum.ContainsWithPoint(Vector3::from_x_and_y_and_z(0.0, 0.0, 6.0)) as i32,
            frustum.Contains(BoundingBox::new(Vector3::new(-0.5), Vector3::new(0.5))) as i32,
            frustum.ContainsWithSphere(BoundingSphere::new(Vector3::Zero, 0.5)) as i32,
        ),
        (1, 0, 1, 1)
    );
    let distant_frustum = BoundingFrustum::new(
        Matrix::CreateLookAt(
            Vector3::from_x_and_y_and_z(100.0, 0.0, 5.0),
            Vector3::from_x_and_y_and_z(100.0, 0.0, 0.0),
            Vector3::Up,
        ) * frustum_projection,
    );
    observe!(
        (
            frustum.Intersects(BoundingBox::new(Vector3::new(-0.5), Vector3::new(0.5))),
            frustum.Intersects(BoundingBox::new(Vector3::new(100.0), Vector3::new(101.0))),
            frustum.IntersectsWithSphere(BoundingSphere::new(Vector3::Zero, 0.5)),
            frustum.IntersectsWithSphere(BoundingSphere::new(Vector3::new(100.0), 0.5)),
            frustum.IntersectsWithFrustum(&distant_frustum),
        ),
        (true, false, true, false, false)
    );
    observe!(
        frustum
            .IntersectsWithRay(Ray::new(
                Vector3::from_x_and_y_and_z(0.0, 0.0, 20.0),
                Vector3::Forward,
            ))
            .map(bits),
        Some(0x4180_0000)
    );

    let keyboard = KeyboardState::new(&[Keys::Z, Keys::A, Keys::A]);
    observe!(
        keyboard
            .GetPressedKeys()
            .into_iter()
            .map(|key| key as i32)
            .collect::<Vec<_>>(),
        vec![65, 90]
    );
    observe!(keyboard.GetHashCode(), 67_108_866);

    let mouse = MouseState::new(
        12,
        -3,
        120,
        ButtonState::Pressed,
        ButtonState::Released,
        ButtonState::Pressed,
        ButtonState::Pressed,
        ButtonState::Released,
    );
    observe!(
        mouse.ToString(),
        "{X:12 Y:-3 Buttons:Left Right XButton1 Wheel:120}"
    );
    observe!(mouse.GetHashCode(), -120);

    let thumb_sticks = GamePadThumbSticks::new(
        Vector2::from_x_and_y(2.0, -2.0),
        Vector2::from_x_and_y(0.25, -0.5),
    );
    observe!(
        (
            bits(thumb_sticks.Left().X),
            bits(thumb_sticks.Left().Y),
            bits(thumb_sticks.Right().X),
            bits(thumb_sticks.Right().Y),
        ),
        (0x3f80_0000, 0xbf80_0000, 0x3e80_0000, 0xbf00_0000)
    );
    let triggers = GamePadTriggers::new(-0.5, 1.5);
    observe!(
        (bits(triggers.Left()), bits(triggers.Right())),
        (0x0000_0000, 0x3f80_0000)
    );

    let gamepad = GamePadState::from_left_thumb_stick_and_right_thumb_stick_and_left_trigger_and_right_trigger_and_buttons(
        Vector2::from_x_and_y(0.1, -0.3),
        Vector2::from_x_and_y(0.3, -0.3),
        0.1,
        0.2,
        &[],
    );
    observe!(
        (
            gamepad.IsButtonDown(Buttons::LeftThumbstickRight),
            gamepad.IsButtonDown(Buttons::LeftThumbstickDown),
            gamepad.IsButtonDown(Buttons::RightThumbstickRight),
            gamepad.IsButtonDown(Buttons::RightThumbstickDown),
            gamepad.IsButtonDown(Buttons::LeftTrigger),
            gamepad.IsButtonDown(Buttons::RightTrigger),
        ),
        (false, true, true, true, false, true)
    );
    let filtered = GamePadState::from_left_thumb_stick_and_right_thumb_stick_and_left_trigger_and_right_trigger_and_buttons(
        Vector2::Zero,
        Vector2::Zero,
        0.0,
        0.0,
        &[Buttons::A, Buttons::LeftTrigger],
    );
    observe!(
        (
            filtered.IsButtonDown(Buttons::A),
            filtered.IsButtonDown(Buttons::LeftTrigger)
        ),
        (true, false)
    );
    observe!(gamepad.ToString(), "{IsConnected:True}");

    let buttons = GamePadButtons::new(Buttons::A | Buttons::Y | Buttons::Back);
    observe!(buttons.ToString(), "{Buttons:A Y Back}");
    observe!(buttons.GetHashCode(), 1);
    let dpad = GamePadDPad::new(
        ButtonState::Pressed,
        ButtonState::Released,
        ButtonState::Released,
        ButtonState::Pressed,
    );
    observe!(dpad.ToString(), "{DPad:Up Right}");
    observe!(dpad.GetHashCode(), i32::MAX);

    let without_previous = TouchLocation::new(
        7,
        TouchLocationState::Pressed,
        Vector2::from_x_and_y(1.0, 2.0),
    );
    let mut previous = TouchLocation::default();
    let has_previous = without_previous.TryGetPreviousLocation(&mut previous);
    observe!(
        (has_previous, previous.Id(), previous.State() as i32),
        (false, -1, 0)
    );
    let first =
        TouchLocation::from_id_and_state_and_position_and_previous_state_and_previous_position(
            5,
            TouchLocationState::Pressed,
            Vector2::from_x_and_y(1.0, 2.0),
            TouchLocationState::Moved,
            Vector2::from_x_and_y(0.5, 1.5),
        );
    let same_coordinates =
        TouchLocation::from_id_and_state_and_position_and_previous_state_and_previous_position(
            5,
            TouchLocationState::Released,
            Vector2::from_x_and_y(1.0, 2.0),
            TouchLocationState::Released,
            Vector2::from_x_and_y(0.5, 1.5),
        );
    observe!(
        (first.Equals(same_coordinates), first == same_coordinates),
        (true, false)
    );
    observe!(first.GetHashCode(), 2_139_095_045);
    observe!(first.ToString(), "{Position:{X:1 Y:2}}");
    let source = [first];
    let collection = TouchCollection::new(&source);
    observe!(collection.Item(0).Id(), 5);
    observe!(collection.Contains(same_coordinates), false);
    observe!(catch_unwind(|| collection.Item(1)).is_err(), true);

    let quaternion_zero = Quaternion::NormalizeWithQuaternion(Quaternion::default());
    observe!(
        (
            bits(quaternion_zero.X),
            bits(quaternion_zero.Y),
            bits(quaternion_zero.Z),
            bits(quaternion_zero.W)
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );
    let inverse_zero = Quaternion::Inverse(Quaternion::default());
    observe!(
        (
            bits(inverse_zero.X),
            bits(inverse_zero.Y),
            bits(inverse_zero.Z),
            bits(inverse_zero.W)
        ),
        (0xffc0_0000, 0xffc0_0000, 0xffc0_0000, 0xffc0_0000)
    );
    let grouped = Quaternion::from_x_and_y_and_z_and_w(
        45_889.058_593_75,
        -42_412.445_312_5,
        96_034.968_75,
        -76_386.843_75,
    ) * Quaternion::from_x_and_y_and_z_and_w(
        -16_375.435_546_875,
        51_428.187_5,
        -69_603.093_75,
        -2_207.379_882_812_5,
    );
    observe!(
        (
            bits(grouped.X),
            bits(grouped.Y),
            bits(grouped.Z),
            bits(grouped.W)
        ),
        (0xce47_a05e, 0xcf03_edf7, 0x4fc9_c4dd, 0x5011_d115)
    );
    let yaw = Quaternion::CreateFromAxisAngle(Vector3::Up, 0.7);
    let pitch = Quaternion::CreateFromAxisAngle(Vector3::Right, -0.4);
    let slerp = Quaternion::Slerp(yaw, pitch, 0.37);
    observe!(
        (bits(slerp.X), bits(slerp.Y), bits(slerp.Z), bits(slerp.W)),
        (0xbd9a_16ec, 0x3e60_d7e7, 0x0000_0000, 0x3f79_023d)
    );
    let large_axis = Quaternion::CreateFromAxisAngle(Vector3::Up, 123_456.789);
    observe!(
        (
            bits(large_axis.X),
            bits(large_axis.Y),
            bits(large_axis.Z),
            bits(large_axis.W)
        ),
        (0x0000_0000, 0x3f30_464f, 0x0000_0000, 0xbf39_a48f)
    );
    let from_matrix = Quaternion::CreateFromRotationMatrix(Matrix::CreateRotationY(0.7));
    observe!(
        (
            bits(from_matrix.X),
            bits(from_matrix.Y),
            bits(from_matrix.Z),
            bits(from_matrix.W)
        ),
        (0x0000_0000, 0x3eaf_904c, 0x0000_0000, 0x3f70_7abb)
    );
    observe!(bits((-Quaternion::default()).X), 0x8000_0000);

    // Curve observations follow the pinned XNA implementation, including its
    // shallow class-reference cloning and duplicate-position collection rules.
    let empty_curve = Curve::new();
    observe!(bits(empty_curve.Evaluate(5.0)), 0x0000_0000);

    let curve = Curve::new();
    curve.Keys().Add(&CurveKey::new(0.0, 0.0));
    curve.Keys().Add(&CurveKey::new(1.0, 10.0));
    observe!(bits(curve.Evaluate(0.25)), 0x3fc8_0000);

    let asymmetric = Curve::new();
    asymmetric.Keys().Add(
        &CurveKey::from_position_and_value_and_tangent_in_and_tangent_out(0.0, 0.0, 99.0, 4.0),
    );
    asymmetric.Keys().Add(
        &CurveKey::from_position_and_value_and_tangent_in_and_tangent_out(2.0, 10.0, -2.0, 77.0),
    );
    observe!(bits(asymmetric.Evaluate(1.0)), 0x40b8_0000);

    let step = Curve::new();
    step.Keys().Add(
        &CurveKey::from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
            0.0,
            2.0,
            0.0,
            0.0,
            CurveContinuity::Step,
        ),
    );
    step.Keys().Add(&CurveKey::new(1.0, 9.0));
    observe!(
        (bits(step.Evaluate(0.999)), bits(step.Evaluate(1.0))),
        (0x4000_0000, 0x4110_0000)
    );

    let mut looped = Curve::new();
    looped.Keys().Add(&CurveKey::new(5.0, 0.0));
    looped.Keys().Add(&CurveKey::new(7.0, 10.0));
    looped.SetPreLoop(CurveLoopType::Cycle);
    looped.SetPostLoop(CurveLoopType::CycleOffset);
    observe!(
        (bits(looped.Evaluate(4.0)), bits(looped.Evaluate(8.0))),
        (0x40a0_0000, 0x4170_0000)
    );

    let tangent_curve = Curve::new();
    tangent_curve.Keys().Add(&CurveKey::new(0.0, 0.0));
    tangent_curve.Keys().Add(&CurveKey::new(1.0, 10.0));
    tangent_curve.Keys().Add(&CurveKey::new(3.0, 30.0));
    tangent_curve.ComputeTangents(CurveTangent::Smooth);
    observe!(
        (
            bits(tangent_curve.Keys().Item(1).TangentIn()),
            bits(tangent_curve.Keys().Item(1).TangentOut())
        ),
        (0x4120_0000, 0x41a0_0000)
    );

    let duplicates = Curve::new();
    let first_key = CurveKey::new(1.0, 10.0);
    let second_key = CurveKey::new(1.0, 20.0);
    duplicates.Keys().Add(&first_key);
    duplicates.Keys().Add(&second_key);
    observe!(
        (
            duplicates.Keys().Count(),
            bits(duplicates.Keys().Item(0).Value()),
            bits(duplicates.Keys().Item(1).Value())
        ),
        (2, 0x4120_0000, 0x41a0_0000)
    );

    let clone = duplicates.Clone();
    let mut cloned_key = clone.Keys().Item(0);
    cloned_key.SetValue(42.0);
    observe!(bits(duplicates.Keys().Item(0).Value()), 0x4228_0000);

    let a = CurveKey::new(f32::NAN, 1.0);
    let b = CurveKey::new(f32::NAN, 1.0);
    observe!((a.CompareTo(&b), a.Equals(&b)), (1, false));

    let mut linear = Curve::new();
    let mut first = CurveKey::new(0.0, 0.0);
    first.SetTangentIn(2.0);
    let mut last = CurveKey::new(1.0, 10.0);
    last.SetTangentOut(3.0);
    linear.Keys().Add(&first);
    linear.Keys().Add(&last);
    linear.SetPreLoop(CurveLoopType::Linear);
    linear.SetPostLoop(CurveLoopType::Linear);
    observe!(
        (bits(linear.Evaluate(-1.0)), bits(linear.Evaluate(3.0))),
        (0xc000_0000, 0x4180_0000)
    );

    // Packed-vector expectations come from the pinned XNA behavior corpus.
    // They deliberately exercise nearest-even ties and XNA's finite exponent-31 half format.
    observe!(
        (
            Alpha8::new(0.5 / 255.0).PackedValue(),
            Bgra5551::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, 0.5).PackedValue()
        ),
        (0x00, 0x0000)
    );
    observe!(
        Byte4::from_x_and_y_and_z_and_w(0.5, 1.5, 2.5, 3.5).PackedValue(),
        0x0402_0200
    );
    observe!(
        NormalizedByte2::from_x_and_y(0.5 / 127.0, -0.5 / 127.0).PackedValue(),
        0x0000
    );
    let mut minimum_snorm = NormalizedByte2::default();
    minimum_snorm.SetPackedValue(0x8080);
    let minimum_snorm = minimum_snorm.ToVector2();
    observe!(
        (bits(minimum_snorm.X), bits(minimum_snorm.Y)),
        (0xbf80_0000, 0xbf80_0000)
    );
    observe!(Short2::from_x_and_y(0.5, 1.5).PackedValue(), 0x0002_0000);
    let mut exponent31_half = HalfSingle::default();
    exponent31_half.SetPackedValue(0x7c00);
    observe!(
        (
            HalfSingle::new(f32::INFINITY).PackedValue(),
            HalfSingle::new(f32::from_bits(0x7fc0_0000)).PackedValue(),
            bits(exponent31_half.ToSingle())
        ),
        (0x7fff, 0x7fff, 0x4780_0000)
    );
    let mut alpha_string = Alpha8::default();
    alpha_string.SetPackedValue(0x0a);
    let mut bgra_string = Bgra5551::default();
    bgra_string.SetPackedValue(0x000a);
    let mut byte_string = Byte4::default();
    byte_string.SetPackedValue(0x0000_000a);
    observe!(
        (
            alpha_string.ToString(),
            bgra_string.ToString(),
            byte_string.ToString()
        ),
        ("0A".to_owned(), "000A".to_owned(), "0000000A".to_owned())
    );

    // XNA graphics-state construction is pure managed behavior. These values
    // are pinned from the XNA 4 reference source rather than backend output.
    let blend = BlendState::new();
    observe!(
        (
            blend.ColorSourceBlend(),
            blend.ColorDestinationBlend(),
            blend.MultiSampleMask()
        ),
        (Blend::One, Blend::Zero, -1)
    );
    let additive = BlendState::Additive;
    let alpha_blend = BlendState::AlphaBlend;
    observe!(
        (
            additive.ColorSourceBlend(),
            alpha_blend.AlphaDestinationBlend()
        ),
        (Blend::SourceAlpha, Blend::InverseSourceAlpha)
    );
    let depth = DepthStencilState::new();
    observe!(
        (
            depth.DepthBufferEnable(),
            depth.DepthBufferWriteEnable(),
            depth.DepthBufferFunction()
        ),
        (true, true, CompareFunction::LessEqual)
    );
    let no_depth = DepthStencilState::None;
    let depth_read = DepthStencilState::DepthRead;
    observe!(
        (
            no_depth.DepthBufferEnable(),
            depth_read.DepthBufferWriteEnable()
        ),
        (false, false)
    );
    observe!(
        RasterizerState::new().CullMode(),
        CullMode::CullCounterClockwiseFace
    );
    let linear_clamp = SamplerState::LinearClamp;
    let point_wrap = SamplerState::PointWrap;
    observe!(
        (
            SamplerState::new().MaxAnisotropy(),
            linear_clamp.AddressU(),
            point_wrap.Filter()
        ),
        (4, TextureAddressMode::Clamp, TextureFilter::Point)
    );
    let presentation = PresentationParameters::new();
    observe!(
        (
            presentation.BackBufferWidth(),
            presentation.BackBufferHeight(),
            presentation.BackBufferFormat(),
            presentation.DepthStencilFormat(),
            presentation.MultiSampleCount(),
            presentation.DisplayOrientation(),
            presentation.PresentationInterval(),
            presentation.RenderTargetUsage(),
            presentation.IsFullScreen(),
            presentation.Bounds(),
        ),
        (
            0,
            0,
            SurfaceFormat::Color,
            DepthFormat::None,
            0,
            DisplayOrientation::Default,
            PresentInterval::Default,
            RenderTargetUsage::DiscardContents,
            true,
            Rectangle::new(0, 0, 0, 0),
        )
    );
    let cloned_presentation = presentation.Clone();
    cloned_presentation.SetBackBufferWidth(640);
    cloned_presentation.SetIsFullScreen(false);
    observe!(
        (
            presentation.BackBufferWidth(),
            presentation.IsFullScreen(),
            cloned_presentation.BackBufferWidth(),
            cloned_presentation.IsFullScreen(),
        ),
        (0, true, 640, false)
    );

    // Game/component/service expectations are pinned from the XNA 4 Game IL:
    // defaults, change-only notifications, retained service identity and
    // collection duplicate/removal behavior are managed framework semantics.
    let mut game = CorpusGame::default();
    observe!(
        (
            game.IsActive(),
            game.IsFixedTimeStep(),
            game.IsMouseVisible(),
            game.TargetElapsedTime().Ticks(),
            game.InactiveSleepTime().Ticks(),
        ),
        (true, true, false, 166_667, 200_000)
    );
    observe!(
        game.SetTargetElapsedTime(cna::Microsoft::Xna::Framework::TimeSpan::Zero)
            .is_err(),
        true
    );
    observe!(
        game.SetInactiveSleepTime(cna::Microsoft::Xna::Framework::TimeSpan::from_ticks(-1))
            .is_err(),
        true
    );
    observe!(
        DisplayOrientation::LandscapeLeft | DisplayOrientation::Portrait,
        DisplayOrientation::LandscapeLeft | DisplayOrientation::Portrait
    );

    let services = GameServiceContainer::new();
    observe!(services.GetService(TypeId::of::<String>()).is_none(), true);
    let provider: Arc<dyn Any + Send + Sync> = Arc::new(String::from("service"));
    services
        .AddService(TypeId::of::<String>(), Arc::clone(&provider))
        .expect("first service registration");
    observe!(
        Arc::ptr_eq(
            &provider,
            &services
                .GetService(TypeId::of::<String>())
                .expect("retained service")
        ),
        true
    );
    observe!(
        services
            .AddService(TypeId::of::<String>(), Arc::new(String::new()))
            .is_err(),
        true
    );
    services.RemoveService(TypeId::of::<String>());
    observe!(services.GetService(TypeId::of::<String>()).is_none(), true);

    let mut component = GameComponent::new(&game);
    observe!(
        (
            component.Enabled(),
            component.UpdateOrder(),
            Arc::ptr_eq(&component.Game().expect("parent game"), game.game_state())
        ),
        (true, 0, true)
    );
    let enabled_events = Arc::new(AtomicUsize::new(0));
    let enabled_count = Arc::clone(&enabled_events);
    let enabled_registration = Arc::new(AtomicU64::new(0));
    let registration_for_handler = Arc::clone(&enabled_registration);
    let registration = component.AddEnabledChangedHandler(Box::new(move |sender: &dyn Any, _| {
        enabled_count.fetch_add(1, Ordering::SeqCst);
        let component = sender
            .downcast_ref::<GameComponent>()
            .expect("GameComponent event sender");
        assert!(
            component.RemoveEnabledChangedHandler(registration_for_handler.load(Ordering::SeqCst))
        );
    }));
    enabled_registration.store(registration, Ordering::SeqCst);
    component.SetEnabled(false);
    component.SetEnabled(true);
    component.SetEnabled(true);
    observe!(enabled_events.load(Ordering::SeqCst), 1);
    let order_events = Arc::new(AtomicUsize::new(0));
    let order_count = Arc::clone(&order_events);
    component.AddUpdateOrderChangedHandler(Box::new(move |_: &dyn Any, _| {
        order_count.fetch_add(1, Ordering::SeqCst);
    }));
    component.SetUpdateOrder(0);
    component.SetUpdateOrder(7);
    observe!(order_events.load(Ordering::SeqCst), 1);

    let mut drawable = DrawableGameComponent::new(&game);
    observe!((drawable.Visible(), drawable.DrawOrder()), (true, 0));
    let drawable_events = Arc::new(AtomicUsize::new(0));
    let visible_count = Arc::clone(&drawable_events);
    drawable.AddVisibleChangedHandler(Box::new(move |_: &dyn Any, _| {
        visible_count.fetch_add(1, Ordering::SeqCst);
    }));
    let draw_order_count = Arc::clone(&drawable_events);
    drawable.AddDrawOrderChangedHandler(Box::new(move |_: &dyn Any, _| {
        draw_order_count.fetch_add(1, Ordering::SeqCst);
    }));
    drawable.SetVisible(false);
    drawable.SetVisible(false);
    drawable.SetDrawOrder(3);
    drawable.SetDrawOrder(3);
    observe!(drawable_events.load(Ordering::SeqCst), 2);

    let collection = GameComponentCollection::new();
    let collection_events = Arc::new(AtomicUsize::new(0));
    let stored: Arc<dyn IGameComponent> = Arc::new(GameComponent::new(&game));
    let expected = Arc::clone(&stored);
    let added_count = Arc::clone(&collection_events);
    collection.AddComponentAddedHandler(Box::new(
        move |_: &dyn Any, args: GameComponentCollectionEventArgs| {
            assert!(Arc::ptr_eq(&args.GameComponent(), &expected));
            added_count.fetch_add(1, Ordering::SeqCst);
        },
    ));
    collection.Add(Arc::clone(&stored));
    observe!(
        (collection.Count(), collection_events.load(Ordering::SeqCst)),
        (1, 1)
    );
    observe!(
        catch_unwind(AssertUnwindSafe(|| collection.Add(Arc::clone(&stored)))).is_err(),
        true
    );
    let removed_count = Arc::clone(&collection_events);
    collection.AddComponentRemovedHandler(Box::new(move |_: &dyn Any, _| {
        removed_count.fetch_add(10, Ordering::SeqCst);
    }));
    observe!(
        (
            collection.Remove(&stored),
            collection.Count(),
            collection_events.load(Ordering::SeqCst)
        ),
        (true, 0, 11)
    );

    // Content metadata defaults and the per-Game manager identity are pinned
    // to the selected XNA runtime contract. XNB framing/failure paths live in
    // the dedicated managed-reader corpus.
    let content = game.Content();
    observe!(Arc::ptr_eq(&content, &game.Content()), true);
    observe!(content.RootDirectory(), String::new());
    content
        .SetRootDirectory("Content")
        .expect("set empty manager root");
    observe!(content.RootDirectory(), "Content".to_owned());

    let mut serializer = ContentSerializerAttribute::new();
    observe!(
        (
            serializer.ElementName(),
            serializer.FlattenContent(),
            serializer.Optional(),
            serializer.AllowNull(),
            serializer.SharedResource(),
            serializer.CollectionItemName(),
            serializer.HasCollectionItemName(),
        ),
        (
            String::new(),
            false,
            false,
            true,
            false,
            "Item".to_owned(),
            false,
        )
    );
    serializer.SetElementName("Root");
    serializer.SetFlattenContent(true);
    serializer.SetOptional(true);
    serializer.SetAllowNull(false);
    serializer.SetSharedResource(true);
    serializer.SetCollectionItemName("Entry");
    observe!(
        (
            serializer.ElementName(),
            serializer.FlattenContent(),
            serializer.Optional(),
            serializer.AllowNull(),
            serializer.SharedResource(),
            serializer.CollectionItemName(),
            serializer.HasCollectionItemName(),
        ),
        (
            "Root".to_owned(),
            true,
            true,
            false,
            true,
            "Entry".to_owned(),
            true,
        )
    );
    observe!(serializer.Clone(), serializer);
    observe!(
        ContentSerializerCollectionItemNameAttribute::new("Glyph").CollectionItemName(),
        "Glyph".to_owned()
    );
    observe!(
        ContentSerializerRuntimeTypeAttribute::new("Example.Type").RuntimeType(),
        "Example.Type".to_owned()
    );
    observe!(
        ContentSerializerTypeVersionAttribute::new(17).TypeVersion(),
        17
    );
    observe!(
        ContentLoadException::from_message("asset failed").to_string(),
        "asset failed".to_owned()
    );

    // Vertex layout/value identities and validation are managed XNA behavior;
    // native transfer and binding execution is covered by native_stress.
    observe!(
        (
            BufferUsage::None | BufferUsage::WriteOnly,
            SetDataOptions::Discard | SetDataOptions::NoOverwrite,
        ),
        (
            BufferUsage::WriteOnly,
            SetDataOptions::Discard | SetDataOptions::NoOverwrite,
        )
    );
    observe!(
        (
            IndexElementSize::SixteenBits as i32,
            IndexElementSize::ThirtyTwoBits as i32,
            PrimitiveType::TriangleList as i32,
            PrimitiveType::LineStrip as i32,
        ),
        (0, 1, 0, 3)
    );
    let element = VertexElement::new(12, VertexElementFormat::Color, VertexElementUsage::Color, 0);
    observe!(
        (
            element.Offset(),
            element.VertexElementFormat(),
            element.VertexElementUsage(),
            element.UsageIndex(),
        ),
        (12, VertexElementFormat::Color, VertexElementUsage::Color, 0,)
    );
    let declaration = VertexDeclaration::new(&[
        VertexElement::new(
            0,
            VertexElementFormat::Vector3,
            VertexElementUsage::Position,
            0,
        ),
        element,
    ])
    .expect("inferred vertex declaration");
    observe!(
        (declaration.VertexStride(), declaration.GetVertexElements()),
        (
            16,
            vec![
                VertexElement::new(
                    0,
                    VertexElementFormat::Vector3,
                    VertexElementUsage::Position,
                    0,
                ),
                element,
            ]
        )
    );
    observe!(VertexDeclaration::new(&[]).is_err(), true);
    observe!(
        VertexDeclaration::from_vertex_stride_and_elements(12, &[element]).is_err(),
        true
    );
    observe!(
        (
            VertexPositionColor::VertexDeclaration().VertexStride(),
            VertexPositionColorTexture::VertexDeclaration().VertexStride(),
            VertexPositionNormalTexture::VertexDeclaration().VertexStride(),
            VertexPositionTexture::VertexDeclaration().VertexStride(),
        ),
        (16, 24, 32, 20)
    );

    observe!(
        (
            GestureType::None | GestureType::Tap,
            (GestureType::HorizontalDrag | GestureType::VerticalDrag) & GestureType::VerticalDrag,
        ),
        (GestureType::Tap, GestureType::VerticalDrag)
    );
    let gesture = GestureSample::new(
        GestureType::Pinch,
        TimeSpan::from_ticks(123_456),
        Vector2::from_x_and_y(1.0, 2.0),
        Vector2::from_x_and_y(3.0, 4.0),
        Vector2::from_x_and_y(5.0, 6.0),
        Vector2::from_x_and_y(7.0, 8.0),
    );
    observe!(
        (
            gesture.GestureType(),
            gesture.Timestamp().Ticks(),
            gesture.Position(),
            gesture.Position2(),
            gesture.Delta(),
            gesture.Delta2(),
        ),
        (
            GestureType::Pinch,
            123_456,
            Vector2::from_x_and_y(1.0, 2.0),
            Vector2::from_x_and_y(3.0, 4.0),
            Vector2::from_x_and_y(5.0, 6.0),
            Vector2::from_x_and_y(7.0, 8.0),
        )
    );

    let information = GraphicsDeviceInformation::new();
    observe!(
        (
            information.GraphicsProfile(),
            information.PresentationParameters().BackBufferWidth(),
            information.PresentationParameters().BackBufferHeight(),
            information.PresentationParameters().BackBufferFormat(),
            information.PresentationParameters().DepthStencilFormat(),
            information.PresentationParameters().IsFullScreen(),
        ),
        (
            GraphicsProfile::Reach,
            0,
            0,
            SurfaceFormat::Color,
            DepthFormat::None,
            true,
        )
    );
    let shared_information = information.clone();
    shared_information.SetGraphicsProfile(GraphicsProfile::HiDef);
    observe!(information.GraphicsProfile(), GraphicsProfile::HiDef);
    let explicit_clone = information.Clone();
    explicit_clone
        .PresentationParameters()
        .SetBackBufferWidth(321);
    let args = PreparingDeviceSettingsEventArgs::new(Arc::new(information));
    observe!(
        (
            args.GraphicsDeviceInformation()
                .PresentationParameters()
                .BackBufferWidth(),
            explicit_clone.PresentationParameters().BackBufferWidth(),
            explicit_clone.Equals(args.GraphicsDeviceInformation().as_ref() as &dyn Any),
        ),
        (0, 321, false)
    );

    // XNA Design metadata, IL, and Windows TypeConverter behavior are mapped
    // through a closed Rust value/type domain; no CLR designer host is used.
    let math_converter = MathTypeConverter::new();
    observe!(
        (
            math_converter.CanConvertFrom(DesignType::String),
            math_converter.CanConvertTo(DesignType::String),
            math_converter.CanConvertTo(DesignType::InstanceDescriptor),
            math_converter.GetCreateInstanceSupported(),
            math_converter.GetPropertiesSupported(),
        ),
        (true, true, true, true, true)
    );
    observe!(
        design_properties(&PointConverter::new()),
        vec![("X", DesignType::Int32), ("Y", DesignType::Int32)]
    );
    observe!(
        design_properties(&RectangleConverter::new()),
        vec![
            ("X", DesignType::Int32),
            ("Y", DesignType::Int32),
            ("Width", DesignType::Int32),
            ("Height", DesignType::Int32),
        ]
    );
    observe!(
        design_properties(&Vector2Converter::new()),
        vec![("X", DesignType::Single), ("Y", DesignType::Single)]
    );
    observe!(
        design_properties(&Vector3Converter::new()),
        vec![
            ("X", DesignType::Single),
            ("Y", DesignType::Single),
            ("Z", DesignType::Single),
        ]
    );
    observe!(
        design_properties(&Vector4Converter::new()),
        vec![
            ("X", DesignType::Single),
            ("Y", DesignType::Single),
            ("Z", DesignType::Single),
            ("W", DesignType::Single),
        ]
    );
    observe!(
        design_properties(&QuaternionConverter::new()),
        vec![
            ("X", DesignType::Single),
            ("Y", DesignType::Single),
            ("Z", DesignType::Single),
            ("W", DesignType::Single),
        ]
    );
    observe!(
        design_properties(&ColorConverter::new()),
        vec![
            ("R", DesignType::Byte),
            ("G", DesignType::Byte),
            ("B", DesignType::Byte),
            ("A", DesignType::Byte),
        ]
    );
    observe!(
        design_properties(&MatrixConverter::new()),
        vec![
            ("Translation", DesignType::Vector3),
            ("M11", DesignType::Single),
            ("M12", DesignType::Single),
            ("M13", DesignType::Single),
            ("M14", DesignType::Single),
            ("M21", DesignType::Single),
            ("M22", DesignType::Single),
            ("M23", DesignType::Single),
            ("M24", DesignType::Single),
            ("M31", DesignType::Single),
            ("M32", DesignType::Single),
            ("M33", DesignType::Single),
            ("M34", DesignType::Single),
            ("M41", DesignType::Single),
            ("M42", DesignType::Single),
            ("M43", DesignType::Single),
            ("M44", DesignType::Single),
        ]
    );
    observe!(
        design_properties(&BoundingBoxConverter::new()),
        vec![("Min", DesignType::Vector3), ("Max", DesignType::Vector3)]
    );
    observe!(
        design_properties(&BoundingSphereConverter::new()),
        vec![
            ("Center", DesignType::Vector3),
            ("Radius", DesignType::Single)
        ]
    );
    observe!(
        design_properties(&PlaneConverter::new()),
        vec![("Normal", DesignType::Vector3), ("D", DesignType::Single)]
    );
    observe!(
        design_properties(&RayConverter::new()),
        vec![
            ("Position", DesignType::Vector3),
            ("Direction", DesignType::Vector3),
        ]
    );
    observe!(
        design_support(&PointConverter::new()),
        (true, false, true, true, true, true)
    );
    observe!(
        design_support(&RectangleConverter::new()),
        (false, false, true, true, true, true)
    );
    observe!(
        design_support(&Vector3Converter::new()),
        (true, false, true, true, true, true)
    );
    observe!(
        design_support(&BoundingBoxConverter::new()),
        (false, false, true, true, true, true)
    );

    let design_point = DesignValue::Point(Point::new(1, -2));
    observe!(
        design_text(
            PointConverter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&design_point),
                    Some(DesignType::String),
                )
                .expect("Point Design string")
        ),
        "1, -2"
    );
    observe!(
        design_text(
            PointConverter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&design_point),
                    Some(DesignType::String),
                )
                .expect("Point German Design string")
        ),
        "1; -2"
    );
    let design_vector3 = DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.25, -2.5, 3.75));
    observe!(
        design_text(
            Vector3Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&design_vector3),
                    Some(DesignType::String),
                )
                .expect("Vector3 Design string")
        ),
        "1.25, -2.5, 3.75"
    );
    observe!(
        design_text(
            Vector3Converter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&design_vector3),
                    Some(DesignType::String),
                )
                .expect("Vector3 German Design string")
        ),
        "1,25; -2,5; 3,75"
    );
    let design_special = DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -0.0,
    ));
    observe!(
        design_text(
            Vector4Converter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&design_special),
                    Some(DesignType::String),
                )
                .expect("Vector4 special Design string")
        ),
        "NaN, Infinity, -Infinity, 0"
    );
    observe!(
        design_text(
            Vector4Converter::new()
                .ConvertTo(
                    &DesignCulture::DeDe,
                    Some(&design_special),
                    Some(DesignType::String),
                )
                .expect("Vector4 German special Design string")
        ),
        "NaN; +unendlich; -unendlich; 0"
    );
    let design_color = DesignValue::Color(
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(0, 255, 10, 40),
    );
    observe!(
        design_text(
            ColorConverter::new()
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&design_color),
                    Some(DesignType::String),
                )
                .expect("Color Design string")
        ),
        "0, 255, 10, 40"
    );

    observe!(
        (
            design_text(
                RectangleConverter::new()
                    .ConvertTo(
                        &DesignCulture::DeDe,
                        Some(&DesignValue::Rectangle(Rectangle::new(1, 2, 3, 4))),
                        Some(DesignType::String),
                    )
                    .expect("Rectangle fallback string")
            ),
            design_text(
                MatrixConverter::new()
                    .ConvertTo(
                        &DesignCulture::DeDe,
                        Some(&DesignValue::Matrix(Matrix::Identity)),
                        Some(DesignType::String),
                    )
                    .expect("Matrix fallback string")
            ),
        ),
        (
            "{X:1 Y:2 Width:3 Height:4}".to_owned(),
            "{ {M11:1 M12:0 M13:0 M14:0} {M21:0 M22:1 M23:0 M24:0} {M31:0 M32:0 M33:1 M34:0} {M41:0 M42:0 M43:0 M44:1} }".to_owned(),
        )
    );
    let parsed_point = PointConverter::new()
        .ConvertFrom(
            &DesignCulture::Invariant,
            Some(&DesignValue::String("2147483647, -2147483648".to_owned())),
        )
        .expect("Point bounds parse");
    observe!((parsed_point.X, parsed_point.Y), (i32::MAX, i32::MIN));
    let parsed_vector = Vector3Converter::new()
        .ConvertFrom(
            &DesignCulture::Invariant,
            Some(&DesignValue::String("-0, 1e-30, 3.40282347E+38".to_owned())),
        )
        .expect("Vector3 edge parse");
    observe!(
        (
            bits(parsed_vector.X),
            bits(parsed_vector.Y),
            bits(parsed_vector.Z)
        ),
        (0x8000_0000, 0x0da2_4260, 0x7f7f_ffff)
    );
    let parsed_german = Vector3Converter::new()
        .ConvertFrom(
            &DesignCulture::DeDe,
            Some(&DesignValue::String("1,5; -2,25; 3,75".to_owned())),
        )
        .expect("Vector3 German parse");
    observe!(
        (
            bits(parsed_german.X),
            bits(parsed_german.Y),
            bits(parsed_german.Z)
        ),
        (0x3fc0_0000, 0xc010_0000, 0x4070_0000)
    );
    let parsed_special = Vector3Converter::new()
        .ConvertFrom(
            &DesignCulture::DeDe,
            Some(&DesignValue::String(
                "NaN; +unendlich; -unendlich".to_owned(),
            )),
        )
        .expect("Vector3 German special parse");
    observe!(
        (
            bits(parsed_special.X),
            bits(parsed_special.Y),
            bits(parsed_special.Z)
        ),
        (0x7fc0_0000, 0x7f80_0000, 0xff80_0000)
    );
    let parsed_color = ColorConverter::new()
        .ConvertFrom(
            &DesignCulture::Invariant,
            Some(&DesignValue::String("0,255,10,40".to_owned())),
        )
        .expect("Color component parse");
    observe!(
        (
            parsed_color.R(),
            parsed_color.G(),
            parsed_color.B(),
            parsed_color.A()
        ),
        (0, 255, 10, 40)
    );

    let design_vector_converter = Vector3Converter::new();
    observe!(
        (
            design_vector_converter
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String(String::new())),
                )
                .is_err(),
            design_vector_converter
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("1,2".to_owned())),
                )
                .is_err(),
            design_vector_converter
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("1,2,3,4".to_owned())),
                )
                .is_err(),
            design_vector_converter
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("1,,3".to_owned())),
                )
                .is_err(),
        ),
        (true, true, true, true)
    );
    observe!(
        design_vector_converter
            .ConvertFrom(
                &DesignCulture::DeDe,
                Some(&DesignValue::String("1.5; -2.25; 3.75".to_owned())),
            )
            .is_err(),
        true
    );
    observe!(
        (
            design_vector_converter
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("3.5e38,0,0".to_owned())),
                )
                .is_err(),
            PointConverter::new()
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("2147483648,0".to_owned())),
                )
                .is_err(),
            ColorConverter::new()
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("256,0,0,0".to_owned())),
                )
                .is_err(),
        ),
        (true, true, true)
    );

    observe!(
        PointConverter::new()
            .CreateInstance(Some(&[
                DesignPropertyValue::new("X", DesignValue::Int32(1)),
                DesignPropertyValue::new("Y", DesignValue::Int32(2)),
            ]))
            .expect("Point reconstruction"),
        Point::new(1, 2)
    );
    observe!(
        Vector3Converter::new()
            .CreateInstance(Some(&[
                DesignPropertyValue::new("X", DesignValue::Single(1.0)),
                DesignPropertyValue::new("Y", DesignValue::Single(2.0)),
                DesignPropertyValue::new("Z", DesignValue::Single(3.0)),
                DesignPropertyValue::new("Extra", DesignValue::Single(4.0)),
            ]))
            .expect("Vector3 reconstruction with extra property"),
        Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)
    );
    let matrix_values: Vec<_> = (1..=16)
        .enumerate()
        .map(|(index, component)| {
            let row = index / 4 + 1;
            let column = index % 4 + 1;
            DesignPropertyValue::new(
                format!("M{row}{column}"),
                DesignValue::Single(component as f32),
            )
        })
        .chain([DesignPropertyValue::new(
            "Translation",
            DesignValue::Vector3(Vector3::from_x_and_y_and_z(100.0, 200.0, 300.0)),
        )])
        .collect();
    let rebuilt_matrix = MatrixConverter::new()
        .CreateInstance(Some(&matrix_values))
        .expect("Matrix scalar reconstruction");
    observe!(
        (
            rebuilt_matrix.M11,
            rebuilt_matrix.M24,
            rebuilt_matrix.M41,
            rebuilt_matrix.M44
        ),
        (1.0, 8.0, 13.0, 16.0)
    );
    let design_sphere = DesignValue::BoundingSphere(BoundingSphere::new(
        Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0),
        4.0,
    ));
    let mut sphere_values = BoundingSphereConverter::new()
        .GetPropertyValues(Some(&design_sphere))
        .expect("BoundingSphere properties");
    sphere_values[0] = DesignPropertyValue::new(
        "Center",
        DesignValue::Vector3(Vector3::from_x_and_y_and_z(99.0, 2.0, 3.0)),
    );
    observe!(
        (design_sphere, sphere_values[0].Value().clone()),
        (
            DesignValue::BoundingSphere(BoundingSphere::new(
                Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0),
                4.0,
            )),
            DesignValue::Vector3(Vector3::from_x_and_y_and_z(99.0, 2.0, 3.0)),
        )
    );
    observe!(
        [
            design_constructor(
                &PointConverter::new(),
                &DesignValue::Point(Point::new(1, 2))
            ),
            design_constructor(
                &RectangleConverter::new(),
                &DesignValue::Rectangle(Rectangle::new(1, 2, 3, 4)),
            ),
            design_constructor(
                &Vector2Converter::new(),
                &DesignValue::Vector2(Vector2::from_x_and_y(1.0, 2.0)),
            ),
            design_constructor(
                &Vector3Converter::new(),
                &DesignValue::Vector3(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0)),
            ),
            design_constructor(
                &Vector4Converter::new(),
                &DesignValue::Vector4(Vector4::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0)),
            ),
            design_constructor(
                &QuaternionConverter::new(),
                &DesignValue::Quaternion(Quaternion::from_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0,)),
            ),
            design_constructor(
                &ColorConverter::new(),
                &DesignValue::Color(
                    Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                        10, 20, 30, 40,
                    ),
                ),
            ),
            design_constructor(
                &MatrixConverter::new(),
                &DesignValue::Matrix(Matrix::Identity),
            ),
            design_constructor(
                &BoundingBoxConverter::new(),
                &DesignValue::BoundingBox(BoundingBox::new(Vector3::new(1.0), Vector3::new(2.0),)),
            ),
            design_constructor(
                &BoundingSphereConverter::new(),
                &DesignValue::BoundingSphere(BoundingSphere::new(Vector3::new(1.0), 2.0)),
            ),
            design_constructor(
                &PlaneConverter::new(),
                &DesignValue::Plane(Plane::from_normal_and_d(Vector3::new(1.0), 2.0)),
            ),
            design_constructor(
                &RayConverter::new(),
                &DesignValue::Ray(Ray::new(Vector3::new(1.0), Vector3::new(2.0))),
            ),
        ],
        [
            DesignConstructor::PointInt32Int32,
            DesignConstructor::RectangleInt32Int32Int32Int32,
            DesignConstructor::Vector2SingleSingle,
            DesignConstructor::Vector3SingleSingleSingle,
            DesignConstructor::Vector4SingleSingleSingleSingle,
            DesignConstructor::QuaternionSingleSingleSingleSingle,
            DesignConstructor::ColorInt32Int32Int32Int32,
            DesignConstructor::MatrixSixteenSingles,
            DesignConstructor::BoundingBoxVector3Vector3,
            DesignConstructor::BoundingSphereVector3Single,
            DesignConstructor::PlaneVector3Single,
            DesignConstructor::RayVector3Vector3,
        ]
    );
    observe!(
        (
            design_vector_converter.CreateInstance(None).is_err(),
            design_vector_converter
                .CreateInstance(Some(&[
                    DesignPropertyValue::new("X", DesignValue::Single(1.0)),
                    DesignPropertyValue::new("Y", DesignValue::Single(2.0)),
                ]))
                .is_err(),
            design_vector_converter
                .CreateInstance(Some(&[
                    DesignPropertyValue::new("X", DesignValue::Int32(1)),
                    DesignPropertyValue::new("Y", DesignValue::Single(2.0)),
                    DesignPropertyValue::new("Z", DesignValue::Single(3.0)),
                ]))
                .is_err(),
            design_vector_converter
                .CreateInstance(Some(&[
                    DesignPropertyValue::new("X", DesignValue::Single(1.0)),
                    DesignPropertyValue::new("Y", DesignValue::Null),
                    DesignPropertyValue::new("Z", DesignValue::Single(3.0)),
                ]))
                .is_err(),
        ),
        (true, true, true, true)
    );
    observe!(
        (
            design_text(
                design_vector_converter
                    .ConvertTo(
                        &DesignCulture::Invariant,
                        Some(&DesignValue::Point(Point::Zero)),
                        Some(DesignType::String),
                    )
                    .expect("base Point string fallback")
            ),
            design_vector_converter
                .ConvertTo(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::Vector3(Vector3::Zero)),
                    Some(DesignType::Int32),
                )
                .is_err(),
            BoundingBoxConverter::new()
                .ConvertFrom(
                    &DesignCulture::Invariant,
                    Some(&DesignValue::String("1,2".to_owned())),
                )
                .is_err(),
        ),
        ("{X:0 Y:0}".to_owned(), true, true)
    );

    observe!(
        (
            AudioChannels::Mono as i32,
            AudioChannels::Stereo as i32,
            AudioStopOptions::AsAuthored as i32,
            AudioStopOptions::Immediate as i32,
            SoundState::Playing as i32,
            SoundState::Paused as i32,
            SoundState::Stopped as i32,
            MicrophoneState::Started as i32,
            MicrophoneState::Stopped as i32,
        ),
        (1, 2, 0, 1, 0, 1, 2, 0, 1)
    );
    let listener = AudioListener::new();
    observe!(listener.Position(), Vector3::Zero);
    observe!(listener.Velocity(), Vector3::Zero);
    observe!(listener.Forward(), Vector3::Forward);
    observe!(listener.Up(), Vector3::Up);
    let mut emitter = AudioEmitter::new();
    observe!(emitter.Position(), Vector3::Zero);
    observe!(emitter.Velocity(), Vector3::Zero);
    observe!(emitter.Forward(), Vector3::Forward);
    observe!(emitter.Up(), Vector3::Up);
    observe!(bits(emitter.DopplerScale()), 0x3f80_0000);
    observe!(
        catch_unwind(AssertUnwindSafe(|| emitter.SetDopplerScale(-1.0))).is_err(),
        true
    );
    emitter.SetDopplerScale(f32::NAN);
    observe!(emitter.DopplerScale().is_nan(), true);
    emitter.SetDopplerScale(-0.0);
    observe!(bits(emitter.DopplerScale()), 0x8000_0000);
    observe!(
        SoundEffect::GetSampleDuration(88_200, 44_100, AudioChannels::Mono),
        TimeSpan::FromSeconds(1.0)
    );
    observe!(
        SoundEffect::GetSampleSizeInBytes(
            TimeSpan::FromSeconds(1.0),
            44_100,
            AudioChannels::Mono,
        ),
        88_198
    );
    observe!(
        SoundEffect::GetSampleSizeInBytes(
            TimeSpan::FromSeconds(1.0),
            44_100,
            AudioChannels::Stereo,
        ),
        176_400
    );
    observe!(
        SoundEffect::GetSampleDuration(1, 44_100, AudioChannels::Mono),
        TimeSpan::Zero
    );
    observe!(
        SoundEffect::GetSampleSizeInBytes(TimeSpan::Zero, 8_000, AudioChannels::Mono),
        0
    );
    observe!(
        SoundEffect::GetSampleSizeInBytes(
            TimeSpan::FromMilliseconds(10.0),
            8_000,
            AudioChannels::Mono,
        ),
        160
    );
    observe!(
        (
            catch_unwind(|| {
                SoundEffect::GetSampleSizeInBytes(
                    TimeSpan::from_ticks(-1),
                    8_000,
                    AudioChannels::Mono,
                )
            })
            .is_err(),
            catch_unwind(|| SoundEffect::GetSampleDuration(2, 7_999, AudioChannels::Mono))
                .is_err(),
        ),
        (true, true)
    );

    // XNA Media value identities and fixed visualization shape. These are
    // reference-runtime facts, not HEADLESS/native-backend observations.
    observe!(MediaSourceType::LocalDevice as i32, 0);
    observe!(MediaSourceType::WindowsMediaConnect as i32, 4);
    observe!(MediaState::Stopped as i32, 0);
    observe!(MediaState::Playing as i32, 1);
    observe!(MediaState::Paused as i32, 2);
    observe!(VideoSoundtrackType::Music as i32, 0);
    observe!(VideoSoundtrackType::Dialog as i32, 1);
    observe!(VideoSoundtrackType::MusicAndDialog as i32, 2);
    let visualization = VisualizationData::new();
    observe!(visualization.Frequencies().len(), 256);
    observe!(visualization.Samples().len(), 256);

    assert_eq!(observations, 215);
}
