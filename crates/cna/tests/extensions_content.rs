//! CNA's `.cnb` content container, end to end.
//!
//! The fixture is built here rather than checked in: encoding texture data and
//! parsing the result back is the round trip that proves both directions, and
//! it needs no asset the repository is not allowed to carry.

use cna::extensions::content::{AssetTypeId, CnbDocument, CnbTextureData, ReadLimits};
use cna::CnaError;

/// A 4x2 image whose every pixel is distinguishable, so a byte that moves is
/// visible rather than averaged away.
fn rgba_fixture() -> (u32, u32, Vec<u8>) {
    let (width, height) = (4_u32, 2_u32);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for index in 0..width * height {
        let value = (index * 17) as u8;
        rgba.extend_from_slice(&[value, value.wrapping_add(1), value.wrapping_add(2), 0xFF]);
    }
    (width, height, rgba)
}

#[test]
fn a_texture_survives_a_cnb_round_trip() {
    let (width, height, rgba) = rgba_fixture();
    let source = CnbTextureData::from_rgba8(width, height, &rgba).expect("build texture data");

    let info = source.info().expect("texture info");
    assert_eq!((info.width, info.height), (width, height));
    assert_eq!(info.face_count, 1, "a Texture2D has one face");
    assert!(info.representation_count >= 1);

    let bytes = source
        .encode_texture2d("cna-rust round trip")
        .expect("encode a .cnb document");
    assert!(!bytes.is_empty());

    let document =
        CnbDocument::parse(&bytes, "in-memory fixture", ReadLimits::default()).expect("parse");
    assert_eq!(document.origin().as_deref(), Ok("in-memory fixture"));
    let (major, minor) = document.container_version().expect("container version");
    assert!(major >= 1, "a container always declares a major version");
    let _ = minor;
    assert!(document.chunk_count().expect("chunk count") >= 1);

    // The document knows what it holds: a built-in Texture2D identity, which
    // is not a custom one.
    let asset_type = document.asset_type().expect("asset type");
    assert_eq!(asset_type, AssetTypeId::TEXTURE2D);
    assert_eq!(asset_type.is_custom(), Ok(false));
    let name = asset_type.name().expect("asset type name");
    assert!(!name.is_empty());

    // Minting a custom identity is deliberately not the inverse of naming a
    // built-in one: it hashes into the custom range, so feeding it a built-in
    // name yields a custom identity rather than that built-in.
    let minted = AssetTypeId::custom(&name).expect("mint a custom identity");
    assert_ne!(minted, asset_type);
    assert_eq!(minted.is_custom(), Ok(true));
    assert_eq!(
        AssetTypeId::custom(&name),
        Ok(minted),
        "minting is deterministic for one name",
    );
    assert!(AssetTypeId::custom("").is_err(), "an empty name is refused");

    let metadata = document.metadata().expect("metadata");
    if metadata.present {
        assert_eq!(metadata.content_name, "cna-rust round trip");
    }

    let decoded = document.decode_texture2d().expect("decode Texture2D");
    let decoded_info = decoded.info().expect("decoded info");
    assert_eq!((decoded_info.width, decoded_info.height), (width, height));
    assert_eq!(
        decoded.level_dimensions(0).expect("level dimensions"),
        (width, height, 1)
    );

    // The pixels come back byte for byte. This is the assertion the whole
    // round trip exists for.
    let representations = decoded.representation_count().expect("representations");
    assert!(representations >= 1);
    let format = decoded.representation_format(0).expect("format");
    assert!(!format.name().expect("format name").is_empty());
    assert!(format.unit_bytes().expect("unit bytes") >= 1);
    assert_eq!(decoded.level_count(0).expect("level count"), 1);
    assert_eq!(decoded.level_bytes(0, 0).expect("level bytes"), rgba);

    // A .cnb format that has an XNA counterpart maps to it; one that does not
    // reports so rather than being forced onto the nearest XNA format.
    match format.surface_format() {
        Ok(_) => {}
        Err(CnaError::UnsupportedRuntime(_) | CnaError::Native { .. }) => {}
        Err(error) => panic!("unexpected surface-format failure: {error}"),
    }
}

#[test]
fn a_malformed_document_is_refused_rather_than_guessed_at() {
    // Not a container at all.
    assert!(CnbDocument::parse(b"not a cnb file", "junk", ReadLimits::default()).is_err());
    assert!(CnbDocument::parse(&[], "empty", ReadLimits::default()).is_err());

    // A truncated real document is refused too: the parser is bounded by the
    // container's own invariants, not by the bytes it happens to have.
    let (width, height, rgba) = rgba_fixture();
    let bytes = CnbTextureData::from_rgba8(width, height, &rgba)
        .expect("build texture data")
        .encode_texture2d("truncated")
        .expect("encode");
    let truncated = &bytes[..bytes.len() / 2];
    assert!(CnbDocument::parse(truncated, "truncated", ReadLimits::default()).is_err());
}

#[test]
fn read_limits_bound_an_untrusted_document() {
    let (width, height, rgba) = rgba_fixture();
    let bytes = CnbTextureData::from_rgba8(width, height, &rgba)
        .expect("build texture data")
        .encode_texture2d("bounded")
        .expect("encode");

    // The defaults accept the document this test just produced.
    CnbDocument::parse(&bytes, "bounded", ReadLimits::default()).expect("default limits accept it");

    // A file-size bound below the document's own size refuses it. A `.cnb` is
    // untrusted input, and the point of the bounds is that the file does not
    // get to decide how much the parser will read.
    let limits = ReadLimits {
        max_file_size: Some(1),
        ..ReadLimits::default()
    };
    assert!(CnbDocument::parse(&bytes, "bounded", limits).is_err());
}
