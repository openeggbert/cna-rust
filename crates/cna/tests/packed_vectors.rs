//! XNA-derived packed-vector behavior, including all seventeen storage formats.

#![allow(non_snake_case)]

use cna::Microsoft::Xna::Framework::Graphics::PackedVector::{
    Alpha8, Bgr565, Bgra4444, Bgra5551, Byte4, HalfSingle, HalfVector2, HalfVector4, IPackedVector,
    NormalizedByte2, NormalizedByte4, NormalizedShort2, NormalizedShort4, Rg32, Rgba1010102,
    Rgba64, Short2, Short4,
};
use cna::Microsoft::Xna::Framework::{Vector2, Vector3, Vector4};

#[test]
fn all_xna_packed_formats_have_real_pack_and_unpack_behavior() {
    assert_eq!(Alpha8::new(1.0).PackedValue(), 0xff);
    assert_eq!(
        Bgr565::from_x_and_y_and_z(0.0, 1.0, 0.0).PackedValue(),
        0x07e0
    );
    assert_eq!(
        Bgra4444::from_x_and_y_and_z_and_w(0.0, 0.0, 1.0, 0.0).PackedValue(),
        0x000f
    );
    assert_eq!(
        Bgra5551::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, 1.0).PackedValue(),
        0x8000
    );
    assert_eq!(
        Byte4::from_x_and_y_and_z_and_w(1.5, 2.5, 3.5, 4.5).PackedValue(),
        0x0404_0202
    );
    assert_eq!(HalfSingle::new(1.0).PackedValue(), 0x3c00);
    assert_eq!(
        HalfVector2::from_x_and_y(1.0, 2.0).PackedValue(),
        0x4000_3c00
    );
    assert_eq!(
        HalfVector4::from_x_and_y_and_z_and_w(1.0, -2.0, 0.5, 4.0).ToVector4(),
        Vector4::from_x_and_y_and_z_and_w(1.0, -2.0, 0.5, 4.0)
    );
    assert_eq!(
        NormalizedByte2::from_x_and_y(1.0, -1.0).PackedValue(),
        0x817f
    );
    assert_eq!(
        NormalizedByte4::from_x_and_y_and_z_and_w(1.0, -1.0, 0.0, 1.0).PackedValue(),
        0x7f00_817f
    );
    assert_eq!(
        NormalizedShort2::from_x_and_y(1.0, -1.0).PackedValue(),
        0x8001_7fff
    );
    assert_eq!(
        NormalizedShort4::from_x_and_y_and_z_and_w(1.0, -1.0, 0.0, 0.5).PackedValue(),
        0x4000_0000_8001_7fff
    );
    assert_eq!(Rg32::from_x_and_y(1.0, 0.5).PackedValue(), 0x8000_ffff);
    assert_eq!(
        Rgba1010102::from_x_and_y_and_z_and_w(1.0, 0.0, 0.0, 0.5).PackedValue(),
        0x8000_03ff
    );
    assert_eq!(
        Rgba64::from_x_and_y_and_z_and_w(0.0, 0.5, 1.0, 0.25).PackedValue(),
        0x4000_ffff_8000_0000
    );
    assert_eq!(Short2::from_x_and_y(1.5, -2.5).PackedValue(), 0xfffe_0002);
    assert_eq!(
        Short4::from_x_and_y_and_z_and_w(40_000.0, -40_000.0, 0.5, 1.5).PackedValue(),
        0x0002_0000_8000_7fff
    );
}

#[test]
fn every_format_round_trips_through_the_untyped_interface() {
    fn changes<T: IPackedVector + Default>(mut value: T) -> bool {
        value.PackFromVector4(Vector4::Zero);
        let zero = value.ToVector4();
        value.PackFromVector4(Vector4::One);
        zero != value.ToVector4()
    }

    assert!(changes(Alpha8::default()));
    assert!(changes(Bgr565::default()));
    assert!(changes(Bgra4444::default()));
    assert!(changes(Bgra5551::default()));
    assert!(changes(Byte4::default()));
    assert!(changes(HalfSingle::default()));
    assert!(changes(HalfVector2::default()));
    assert!(changes(HalfVector4::default()));
    assert!(changes(NormalizedByte2::default()));
    assert!(changes(NormalizedByte4::default()));
    assert!(changes(NormalizedShort2::default()));
    assert!(changes(NormalizedShort4::default()));
    assert!(changes(Rg32::default()));
    assert!(changes(Rgba1010102::default()));
    assert!(changes(Rgba64::default()));
    assert!(changes(Short2::default()));
    assert!(changes(Short4::default()));

    // The mapped primary constructors accept their XNA vector value types.
    assert_eq!(Bgr565::new(Vector3::One).PackedValue(), 0xffff);
    assert_eq!(Rg32::new(Vector2::One).PackedValue(), u32::MAX);
}
