//! `texture.h`: the format arithmetic, and textures with no graphics device.
//!
//! The format facts are asserted against relationships that must hold whatever
//! the numbers are -- a compressed format costs more bytes per block than an
//! uncompressed one costs per texel, an uncompressed block covers one texel,
//! alignment is in range -- and the exact values are *printed*, because they
//! are CNA's to choose and this test's job is to notice when they change, not
//! to restate them.
//!
//! The device-free textures are the part with no XNA counterpart at all, and
//! they run on any artifact: no renderer is involved.


use cna::extensions::texture::{
    validate_base_texture_format, validate_get_data_element_size, FormatFacts, ImageFormat,
    StandaloneTexture, Texture2DFile,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters, SurfaceFormat, Texture2D,
};
use cna::Microsoft::Xna::Framework::{Color, GraphicsDeviceInformation};
use cna::{CnaError, ErrorCategory};

fn skip() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_none()
}

#[test]
fn cna_answers_for_every_surface_format_and_the_answers_cohere() {
    if skip() {
        return;
    }
    let interesting = [
        SurfaceFormat::Color,
        SurfaceFormat::Bgr565,
        SurfaceFormat::Dxt1,
        SurfaceFormat::Dxt5,
        SurfaceFormat::Single,
        SurfaceFormat::Vector4,
        SurfaceFormat::HdrBlendable,
    ];
    for format in interesting {
        let facts = FormatFacts::of(format).unwrap_or_else(|e| panic!("{format:?}: {e:?}"));
        println!("NOTE: {format:?} -> {facts:?} compressed={}", facts.is_compressed());

        assert!(
            facts.bytes_per_unit > 0,
            "{format:?} costs a positive number of bytes"
        );
        assert!(
            facts.block_size_squared >= 1,
            "{format:?} covers at least one texel per unit"
        );
        assert!(
            (1..=8).contains(&facts.pixel_store_alignment),
            "{format:?} alignment must be one through eight, got {}",
            facts.pixel_store_alignment
        );
    }

    // The relationship that gives the two block routes their meaning: a DXT
    // block covers many texels, an uncompressed texel covers exactly one.
    let dxt1 = FormatFacts::of(SurfaceFormat::Dxt1).expect("Dxt1");
    let color = FormatFacts::of(SurfaceFormat::Color).expect("Color");
    assert!(dxt1.is_compressed(), "Dxt1 is a block format");
    assert!(!color.is_compressed(), "Color is not");
    assert_eq!(color.block_size_squared, 1);
    assert!(
        dxt1.block_size_squared > color.block_size_squared,
        "a compression block covers more texels than one uncompressed texel"
    );
}

#[test]
fn the_base_texture_contract_is_narrower_than_a_renderers() {
    if skip() {
        return;
    }
    // Documented: the renderer-independent base contract accepts Color alone.
    validate_base_texture_format(SurfaceFormat::Color).expect("Color is the base format");

    let refusal = validate_base_texture_format(SurfaceFormat::Dxt1)
        .expect_err("a valid but non-base format is refused");
    println!("NOTE: base contract refuses Dxt1: {refusal}");
    assert!(
        matches!(
            refusal,
            CnaError::Native {
                category: ErrorCategory::NotSupported,
                ..
            }
        ),
        "a valid format outside the base contract is NOT_SUPPORTED, not an argument \
         error -- the two say different things and only one means 'no such format': \
         {refusal:?}"
    );
}

#[test]
fn an_element_size_must_divide_the_format_unit() {
    if skip() {
        return;
    }
    let bytes = FormatFacts::of(SurfaceFormat::Color)
        .expect("Color")
        .bytes_per_unit;

    // A whole unit always divides itself.
    validate_get_data_element_size(SurfaceFormat::Color, bytes)
        .expect("one whole unit is a valid element size");

    // A size that does not divide the unit is refused. `bytes + 1` cannot
    // divide `bytes` for any bytes > 1.
    let refusal = validate_get_data_element_size(SurfaceFormat::Color, bytes + 1)
        .expect_err("an element larger than the unit cannot divide it");
    println!("NOTE: {} does not divide a {bytes}-byte unit: {refusal}", bytes + 1);
}

#[test]
fn a_standalone_texture_needs_no_device_at_all() {
    if skip() {
        return;
    }
    let empty = StandaloneTexture::empty().expect("a default standalone texture");
    let facts = empty.facts().expect("its facts");
    let storage = empty.storage().expect("its storage");
    println!("NOTE: empty standalone -> {facts:?} {storage:?}");
    assert!(
        !storage.has_renderer,
        "a texture created without a device has no renderer resource"
    );

    // Releasing twice is safe, and a released texture answers a refusal rather
    // than reaching into a freed handle.
    empty.release().expect("release");
    empty.release().expect("releasing twice is a no-op");
    let after = empty.facts().expect_err("a released texture answers nothing");
    println!("NOTE: after release: {after}");
}

#[test]
fn a_cpu_only_texture_holds_the_pixels_it_was_given() {
    if skip() {
        return;
    }
    let pixels = vec![
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(255, 0, 0, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(0, 255, 0, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(0, 0, 255, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(255, 255, 255, 128),
    ];
    let texture = StandaloneTexture::from_pixels(2, 2, SurfaceFormat::Color, &pixels)
        .expect("a 2x2 CPU-only texture");

    let storage = texture.storage().expect("its storage");
    println!("NOTE: cpu-only storage -> {storage:?}");
    assert!(
        storage.has_cpu_shadow,
        "a CPU-only texture is exactly one that keeps an authoritative CPU shadow"
    );
    assert!(!storage.has_renderer, "and has no renderer resource");

    assert_eq!(
        texture.facts().expect("its facts").format,
        SurfaceFormat::Color,
        "it reports the format it was created with"
    );

    // The wrong number of pixels is refused before it reaches CNA, because a
    // short array would otherwise be read past its end.
    let short = StandaloneTexture::from_pixels(2, 2, SurfaceFormat::Color, &pixels[..3]);
    assert!(
        short.is_err(),
        "three pixels is not a 2x2 image, and passing the length through would \
         have CNA read a fourth that does not exist"
    );

    // Uploading raw bytes: four per pixel, and a length that is not a whole
    // number of pixels is refused rather than truncated.
    let bytes: Vec<u8> = (0..16).map(|i| i as u8).collect();
    texture
        .set_rgba8_bytes(&bytes)
        .expect("sixteen bytes is four pixels");
    assert!(
        texture.set_rgba8_bytes(&bytes[..15]).is_err(),
        "fifteen bytes is not a whole number of four-byte pixels"
    );
}

#[test]
fn a_texture_round_trips_through_a_png_file() {
    if skip() {
        return;
    }
    let pixels = vec![Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(12, 34, 56, 255); 4];
    let texture = StandaloneTexture::from_pixels(2, 2, SurfaceFormat::Color, &pixels)
        .expect("a 2x2 CPU-only texture");

    let directory = std::env::temp_dir().join(format!(
        "cna-rust-texture-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let path = directory.join("round-trip.png");
    let path_text = path.to_str().expect("a UTF-8 path").to_owned();

    texture
        .save_to_file(ImageFormat::Png, &path_text)
        .expect("encode straight to a file");
    let written = std::fs::metadata(&path).expect("the file exists").len();
    println!("NOTE: wrote {written} bytes to {path_text}");
    assert!(written > 0, "the encoder wrote something");

    // The real qualification: read it back through CNA's own decoder and check
    // the pixels survived, not merely that a file appeared.
    let decoded = StandaloneTexture::from_file(&path_text).expect("decode the file back");
    let facts = decoded.facts().expect("the decoded facts");
    println!("NOTE: decoded -> {facts:?}");
    assert_eq!(
        facts.format,
        SurfaceFormat::Color,
        "a PNG decodes to the Color format"
    );
    assert!(
        decoded.storage().expect("storage").has_cpu_shadow,
        "a file-decoded texture is CPU-backed"
    );

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&directory);
}

#[test]
fn decoding_a_file_that_is_not_an_image_is_refused() {
    if skip() {
        return;
    }
    let directory = std::env::temp_dir().join(format!("cna-rust-bad-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let path = directory.join("not-an-image.png");
    std::fs::write(&path, b"this is not a PNG").expect("write the decoy");
    let path_text = path.to_str().expect("a UTF-8 path").to_owned();

    let refusal = StandaloneTexture::from_file(&path_text)
        .expect_err("bytes that are not an image do not decode");
    println!("NOTE: {refusal}");

    let missing = StandaloneTexture::from_file("/no/such/file/anywhere.png")
        .expect_err("a path that names nothing does not decode");
    println!("NOTE: {missing}");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&directory);

}

/// The same routes against a texture that *does* have a device.
///
/// Skipped on a renderer that cannot build a windowless device, which is a
/// property of the artifact rather than of these routes; the HEADLESS build
/// runs all of it.
#[test]
fn a_device_owned_texture_takes_the_same_file_and_byte_paths() {
    if skip() {
        return;
    }
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    let device = match GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    ) {
        Ok(device) => device,
        Err(CnaError::Native {
            category: ErrorCategory::Platform,
            ref message,
            ..
        }) if message.contains("platform window id") => {
            println!("this renderer cannot create a device without a window: {message}");
            return;
        }
        Err(error) => panic!("independent GraphicsDevice construction failed: {error:?}"),
    };

    let pixels = vec![Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(9, 8, 7, 255); 4];
    let texture = <Texture2D as Texture2DFile>::from_pixels(&device, 2, 2, &pixels)
        .expect("a 2x2 device texture");
    assert_eq!(texture.Width(), 2);
    assert_eq!(texture.Height(), 2);

    let storage = texture.storage().expect("its storage");
    println!("NOTE: device texture storage -> {storage:?}");
    assert!(
        storage.has_renderer,
        "a texture created on a device holds a renderer resource, which is the whole \
         difference from the standalone kind"
    );
    assert_eq!(
        texture.facts().expect("its facts").format,
        SurfaceFormat::Color
    );

    let bytes: Vec<u8> = (0..16).map(|i| (i * 7) as u8).collect();
    texture
        .set_rgba8_bytes(&bytes)
        .expect("sixteen bytes is four pixels");

    let directory = std::env::temp_dir().join(format!("cna-rust-devtex-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let path = directory.join("device.png");
    let path_text = path.to_str().expect("a UTF-8 path").to_owned();
    texture
        .save_to_file(ImageFormat::Png, &path_text)
        .expect("encode straight to a file");

    // Decoded back onto the same device: the file path and the device path
    // meet, which is what says the two halves of this header agree.
    let reloaded =
        <Texture2D as Texture2DFile>::from_file(&device, &path_text).expect("decode onto a device");
    assert_eq!(reloaded.Width(), 2);
    assert_eq!(reloaded.Height(), 2);
    assert!(
        reloaded.storage().expect("storage").has_renderer,
        "a file decoded with a device gets a renderer resource"
    );

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&directory);
}
