//! CNB's primitive writer, cursor and chunk navigation, against the live library.
//!
//! The point of this file is the **round trip**: a payload written with
//! [`CnbByteWriter`] and read back with [`CnbReader`] must produce the values
//! that went in, in order, and the reader must then say the region is
//! exhausted. That is a stronger check than either half alone, because a
//! matching pair of bugs in one implementation would still fail it -- the
//! reader is upstream's, and so is the writer, so what is being measured is
//! that this crate marshals both correctly.
//!
//! What is deliberately *not* tested here is that Rust could do the same with
//! `Vec<u8>`. It could, for the scalars; it could not for the string framing,
//! the keyframe layout or the limit checks, and a second encoder of a format
//! stored in files is the thing this binding exists to avoid.

use cna::extensions::cnb::{
    checked_add, checked_multiply, compression_is_supported, crc32c, crc32c_continue,
    crc32c_portable, crc32c_uses_hardware, format_magic, has_magic, is_well_formed_utf8,
    logical_name_problem, make_chunk_id, CnbByteWriter, CnbReader, ChunkId, Compression,
};
use cna::extensions::cnb::{
    encode_animation_clip, encode_curve, encode_song, encode_texture_cube, encode_video, VideoInfo,
};
use cna::extensions::content::{
    AssetTypeId, CnbDocument, CnbTextureData, CnbTextureFormat, CnbWriter, ReadLimits,
};
use cna::extensions::models::{AnimationClip, BoneTrack, ClipTargetSpace, Keyframe};
use cna::extensions::object_dictionary::ObjectValueKind;
use cna::Microsoft::Xna::Framework::Graphics::SurfaceFormat;
use cna::Microsoft::Xna::Framework::{Curve, CurveContinuity, CurveKey, CurveLoopType, Quaternion, Vector3};

fn native_enabled() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_some()
}

fn sample_keyframe() -> Keyframe {
    Keyframe {
        time_seconds: 1.25,
        translation: Vector3 {
            X: 1.0,
            Y: 2.0,
            Z: 3.0,
        },
        rotation: Quaternion {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
            W: 1.0,
        },
        scale: Vector3 {
            X: 2.0,
            Y: 2.0,
            Z: 2.0,
        },
    }
}

#[test]
fn a_payload_round_trips_through_the_writer_and_the_cursor() {
    if !native_enabled() {
        return;
    }
    let writer = CnbByteWriter::new().expect("a byte writer");
    assert!(writer.is_empty().expect("a fresh writer is empty"));

    writer.write_u8(0x2A).expect("u8");
    writer.write_u16(0xBEEF).expect("u16");
    writer.write_u32(0xDEAD_BEEF).expect("u32");
    writer.write_u64(0x0123_4567_89AB_CDEF).expect("u64");
    writer.write_i32(-42).expect("i32");
    writer.write_f32(0.5).expect("f32");
    writer.write_f64(-0.25).expect("f64");
    writer.write_string("a name with åccents").expect("string");
    writer.write_keyframe(sample_keyframe()).expect("keyframe");
    writer.write_bytes(&[1, 2, 3, 4]).expect("raw bytes");
    writer.write_zeros(3).expect("padding");

    let bytes = writer.to_bytes().expect("the written bytes");
    assert_eq!(
        bytes.len() as u64,
        writer.len().expect("the reported size"),
        "the copy and the size must agree"
    );

    let reader = CnbReader::new(&bytes, "round-trip", ReadLimits::default()).expect("a cursor");
    assert_eq!(reader.len().expect("size"), bytes.len() as u64);
    assert_eq!(reader.position().expect("position"), 0);
    assert_eq!(reader.read_u8().expect("u8"), 0x2A);
    assert_eq!(reader.read_u16().expect("u16"), 0xBEEF);
    assert_eq!(reader.read_u32().expect("u32"), 0xDEAD_BEEF);
    assert_eq!(reader.read_u64().expect("u64"), 0x0123_4567_89AB_CDEF);
    assert_eq!(reader.read_i32().expect("i32"), -42);
    assert_eq!(reader.read_f32().expect("f32"), 0.5);
    assert_eq!(reader.read_f64().expect("f64"), -0.25);
    assert_eq!(
        reader.read_string().expect("string"),
        "a name with åccents",
        "a non-ASCII string must survive the length prefix and the UTF-8 check"
    );

    let keyframe = reader.read_keyframe().expect("keyframe");
    assert_eq!(keyframe, sample_keyframe(), "the 48-byte layout round-trips");

    assert_eq!(reader.read_bytes(4).expect("raw bytes"), vec![1, 2, 3, 4]);
    reader.skip(3).expect("the padding");

    // Nothing left, and the cursor agrees. A schema that has read everything
    // it expects and finds bytes over has misread something.
    assert_eq!(reader.remaining().expect("remaining"), 0);
    reader
        .require_exhausted()
        .expect("the cursor consumed exactly what was written");
}

#[test]
fn a_cursor_refuses_to_read_past_its_region() {
    if !native_enabled() {
        return;
    }
    let writer = CnbByteWriter::new().expect("a byte writer");
    writer.write_u32(7).expect("u32");
    let bytes = writer.to_bytes().expect("bytes");

    let reader = CnbReader::new(&bytes, "short", ReadLimits::default()).expect("a cursor");
    assert_eq!(reader.read_u32().expect("the one value"), 7);
    assert!(
        reader.read_u32().is_err(),
        "reading past the region must fail rather than answer a zero"
    );
    assert!(
        reader.skip(1).is_err(),
        "skipping past the region must fail too"
    );
}

#[test]
fn a_cursor_names_its_context_in_a_failure() {
    if !native_enabled() {
        return;
    }
    let reader = CnbReader::new(&[0_u8; 4], "player.cnb", ReadLimits::default())
        .expect("a cursor");
    assert_eq!(reader.context().expect("context"), "player.cnb");

    // A caller's own schema check, reported the way CNA's are. Always an Err,
    // and the message must reach the caller rather than being swallowed.
    let error = reader
        .fail::<()>("the bone count exceeds the skeleton")
        .expect_err("fail() is documented never to succeed");
    let text = error.to_string();
    assert!(
        text.contains("the bone count exceeds the skeleton"),
        "the caller's message should survive, got: {text}"
    );
}

#[test]
fn a_string_longer_than_the_limit_is_refused_before_it_is_allocated() {
    if !native_enabled() {
        return;
    }
    let writer = CnbByteWriter::new().expect("a byte writer");
    writer.write_string("0123456789").expect("a ten-byte string");
    let bytes = writer.to_bytes().expect("bytes");

    let limits = ReadLimits {
        max_string_bytes: Some(4),
        ..ReadLimits::default()
    };
    let reader = CnbReader::new(&bytes, "limited", limits).expect("a cursor");
    assert!(
        reader.read_string().is_err(),
        "a string longer than the limit must be refused, not truncated"
    );
}

#[test]
fn a_take_empties_the_writer_and_a_copy_does_not() {
    if !native_enabled() {
        return;
    }
    let writer = CnbByteWriter::new().expect("a byte writer");
    writer.write_u32(1).expect("u32");
    let copied = writer.to_bytes().expect("copy");
    assert_eq!(
        writer.len().expect("size after a copy"),
        copied.len() as u64,
        "copying must leave the writer as it was"
    );

    let taken = writer.take().expect("take");
    assert_eq!(taken, copied, "taking must hand over the same bytes");
    assert!(
        writer.is_empty().expect("size after a take"),
        "taking must leave the writer empty"
    );
}

#[test]
fn a_document_s_chunks_are_reachable_by_index_and_by_identity() {
    if !native_enabled() {
        return;
    }
    let asset_type = AssetTypeId::custom("CnaRust.Test.ChunkNavigation").expect("a custom type");
    let writer = CnbWriter::new(asset_type, 1).expect("a writer");
    writer
        .set_metadata("CnaRust.Test.ChunkNavigation", "navigation")
        .expect("metadata");

    let payload = CnbByteWriter::new().expect("a byte writer");
    payload.write_u32(0xC0FF_EE00).expect("u32");
    payload.write_string("payload").expect("string");
    let payload_bytes = payload.to_bytes().expect("bytes");

    let chunk = make_chunk_id(b'T', b'E', b'S', b'T').expect("a chunk identity");
    assert!(
        chunk.is_well_formed().expect("well-formed"),
        "four printable characters make a well-formed identity"
    );
    assert_eq!(chunk.to_text().expect("text"), "TEST");

    writer
        .add_chunk(chunk.0, &payload_bytes, 0, 1)
        .expect("the payload chunk");
    let file = writer.build().expect("build");

    assert!(
        has_magic(&file).expect("magic"),
        "a built document begins with CNB's magic"
    );
    assert!(
        file.starts_with(&format_magic().expect("the magic bytes")),
        "and the magic is the same bytes format_magic answers"
    );

    let document =
        CnbDocument::parse(&file, "navigation.cnb", ReadLimits::default()).expect("parse");
    let index = document
        .require_single(chunk)
        .expect("exactly one TEST chunk");
    assert_eq!(
        document.find_all(chunk).expect("find_all"),
        vec![index],
        "find_all and require_single must agree"
    );
    assert_eq!(document.find_single(chunk).expect("find_single"), Some(index));
    assert_eq!(
        document
            .find_single(ChunkId(0xFFFF_FFFF))
            .expect("an identity the document does not carry"),
        None,
        "an absent chunk is None, not a failure"
    );

    let entry = document.chunk_at(index).expect("the entry");
    assert_eq!(entry.chunk, chunk);
    assert_eq!(entry.uncompressed_size, payload_bytes.len() as u64);
    assert_eq!(
        document.chunk_data(index).expect("the bytes"),
        payload_bytes,
        "a chunk's bytes must come back as they went in"
    );

    // The checksum in the table of contents is a CRC32C over the stored bytes,
    // and this crate can compute the same number.
    if entry.compression == Compression(0) {
        assert_eq!(
            entry.checksum,
            crc32c(&payload_bytes).expect("crc32c"),
            "the entry's checksum should be the CRC32C of the stored bytes"
        );
    }

    // The reading counterpart of building the payload.
    let reader = document.open_chunk(index).expect("a cursor over the chunk");
    assert_eq!(reader.read_u32().expect("u32"), 0xC0FF_EE00);
    assert_eq!(reader.read_string().expect("string"), "payload");
    reader.require_exhausted().expect("nothing left over");

    document
        .require_asset(asset_type.value(), 1)
        .expect("the document is what it claims to be");
    // The second argument is the highest version the decoder understands, not
    // the one it expects. A decoder that reads up to version 3 must accept a
    // version-1 file -- getting this backwards would make every decoder refuse
    // every file older than itself.
    document
        .require_asset(asset_type.value(), 3)
        .expect("a newer decoder reads an older file");
    assert!(
        document.require_asset(asset_type.value(), 0).is_err(),
        "version 1 is always the lowest accepted, so a maximum of 0 admits nothing"
    );
    assert!(
        document.require_asset(asset_type.value() ^ 1, 1).is_err(),
        "a different asset type must be refused on identity"
    );
}

#[test]
fn the_two_crc32c_paths_agree() {
    if !native_enabled() {
        return;
    }
    let bytes: Vec<u8> = (0..=255_u8).cycle().take(4096).collect();
    let hardware = crc32c(&bytes).expect("crc32c");
    let portable = crc32c_portable(&bytes).expect("portable crc32c");
    assert_eq!(
        hardware, portable,
        "the hardware and software CRC32C must agree, or a file written on one \
         machine fails its checksum on another"
    );
    println!(
        "NOTE: crc32c uses hardware here = {}",
        crc32c_uses_hardware().expect("which path")
    );

    // Continuing over two halves must equal one pass over the whole.
    let (first, second) = bytes.split_at(1000);
    let seed = crc32c(first).expect("the first half");
    assert_eq!(
        crc32c_continue(seed, second).expect("continued"),
        hardware,
        "a streamed CRC32C must equal the one-pass answer"
    );
}

#[test]
fn the_small_helpers_answer_what_they_claim() {
    if !native_enabled() {
        return;
    }
    // Overflow is an answer here, not a failure: a schema computing an offset
    // wants the verdict, and `None` is how Rust spells it.
    assert_eq!(checked_add(2, 3).expect("a sum"), Some(5));
    assert_eq!(
        checked_add(u64::MAX, 1).expect("an overflowing sum"),
        None,
        "an overflow must be reported, not wrapped"
    );
    assert_eq!(checked_multiply(6, 7).expect("a product"), Some(42));
    assert_eq!(checked_multiply(u64::MAX, 2).expect("an overflow"), None);

    assert!(is_well_formed_utf8("plain text".as_bytes()).expect("valid"));
    assert!(
        !is_well_formed_utf8(&[0xFF, 0xFE]).expect("invalid"),
        "bytes no Rust `str` could hold must be reported as malformed"
    );

    assert_eq!(
        logical_name_problem("textures/hero").expect("a usable name"),
        None,
        "an ordinary logical name has no problem to report"
    );
    let problem = logical_name_problem("").expect("an empty name");
    println!("NOTE: the empty logical name reports: {problem:?}");

    // Whether a codec is present is a build-time fact, and the honest answer
    // for one CNA knows but was not built with is `false`.
    for raw in 0..4_u32 {
        let compression = Compression(raw);
        println!(
            "NOTE: compression {raw} ({:?}) supported = {:?}",
            compression.name(),
            compression_is_supported(compression)
        );
    }
}

#[test]
fn a_curve_round_trips_through_the_cnb_codec() {
    if !native_enabled() {
        return;
    }
    // The Curve arithmetic is Rust's; only the marshalling is CNA's. What this
    // measures is that a curve survives the crossing unchanged -- the keys, the
    // tangents, the continuity and both loop modes.
    let mut curve = Curve::new();
    curve.SetPreLoop(CurveLoopType::Oscillate);
    curve.SetPostLoop(CurveLoopType::Linear);
    curve.Keys().Add(
        &CurveKey::from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
            0.0,
            1.0,
            0.25,
            -0.25,
            CurveContinuity::Smooth,
        ),
    );
    curve.Keys().Add(
        &CurveKey::from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
            2.5,
            -3.0,
            0.5,
            0.75,
            CurveContinuity::Step,
        ),
    );

    let bytes = encode_curve(&curve, "test-curve").expect("encode a curve");
    assert!(
        has_magic(&bytes).expect("magic"),
        "an encoded curve is a whole .cnb document"
    );

    let document = CnbDocument::parse(&bytes, "curve.cnb", ReadLimits::default())
        .expect("the encoded curve parses");
    let decoded = document.decode_curve().expect("decode a curve");

    assert_eq!(decoded.PreLoop(), CurveLoopType::Oscillate);
    assert_eq!(decoded.PostLoop(), CurveLoopType::Linear);
    assert_eq!(decoded.Keys().Count(), curve.Keys().Count());
    for index in 0..curve.Keys().Count() {
        let before = curve.Keys().Item(index);
        let after = decoded.Keys().Item(index);
        assert_eq!(after.Position(), before.Position(), "key {index} position");
        assert_eq!(after.Value(), before.Value(), "key {index} value");
        assert_eq!(after.TangentIn(), before.TangentIn(), "key {index} in tangent");
        assert_eq!(
            after.TangentOut(),
            before.TangentOut(),
            "key {index} out tangent"
        );
        assert_eq!(
            after.Continuity(),
            before.Continuity(),
            "key {index} continuity"
        );
    }
}

#[test]
fn an_animation_clip_round_trips_through_the_cnb_codec() {
    if !native_enabled() {
        return;
    }
    let clip = AnimationClip {
        duration_seconds: 2.0,
        tracks: vec![BoneTrack {
            bone_index: 3,
            keyframes: vec![
                Keyframe {
                    time_seconds: 0.0,
                    translation: Vector3 { X: 0.0, Y: 0.0, Z: 0.0 },
                    rotation: Quaternion { X: 0.0, Y: 0.0, Z: 0.0, W: 1.0 },
                    scale: Vector3 { X: 1.0, Y: 1.0, Z: 1.0 },
                },
                Keyframe {
                    time_seconds: 2.0,
                    translation: Vector3 { X: 4.0, Y: 5.0, Z: 6.0 },
                    rotation: Quaternion { X: 0.0, Y: 0.0, Z: 0.0, W: 1.0 },
                    scale: Vector3 { X: 2.0, Y: 2.0, Z: 2.0 },
                },
            ],
        }],
    };

    let bytes = encode_animation_clip(&clip, ClipTargetSpace::SceneNode, "test-clip")
        .expect("encode a clip");
    let document = CnbDocument::parse(&bytes, "clip.cnb", ReadLimits::default())
        .expect("the encoded clip parses");
    let decoded = document.decode_animation_clip().expect("decode a clip");

    let (duration, tracks, space) = decoded.info().expect("clip info");
    assert_eq!(duration, 2.0);
    assert_eq!(tracks, 1);
    assert_eq!(
        space,
        ClipTargetSpace::SceneNode,
        "the target space must survive: a joint-palette clip applied to scene \
         nodes poses the wrong bones without saying so"
    );

    let (bone, keyframe_count) = decoded.track(0).expect("track 0");
    assert_eq!(bone, 3);
    assert_eq!(keyframe_count, 2);
    assert_eq!(
        decoded.keyframes(0).expect("keyframes"),
        clip.tracks[0].keyframes,
        "the keyframes must come back exactly as they went in"
    );
}

#[test]
fn a_song_and_a_video_round_trip_as_references() {
    if !native_enabled() {
        return;
    }
    // Neither carries media: both record a stream reference the player opens.
    let bytes = encode_song("audio/title.ogg", "Title", 90_000, "title-song")
        .expect("encode a song");
    let document = CnbDocument::parse(&bytes, "song.cnb", ReadLimits::default())
        .expect("the encoded song parses");
    let (name, milliseconds, stream) = document.decode_song().expect("decode a song");
    assert_eq!(name, "Title");
    assert_eq!(milliseconds, 90_000);
    assert_eq!(stream, "audio/title.ogg");

    let mut info = VideoInfo::default();
    info.duration_milliseconds = 5_000;
    info.width = 640;
    info.height = 360;
    info.frames_per_second = 30.0;
    let bytes = encode_video("video/intro.mp4", info, "intro").expect("encode a video");
    let document = CnbDocument::parse(&bytes, "video.cnb", ReadLimits::default())
        .expect("the encoded video parses");
    let (decoded, stream) = document.decode_video().expect("decode a video");
    assert_eq!(decoded.width, 640);
    assert_eq!(decoded.height, 360);
    assert_eq!(decoded.frames_per_second, 30.0);
    assert_eq!(decoded.duration_milliseconds, 5_000);
    assert_eq!(stream, "video/intro.mp4");
}

#[test]
fn a_texture_carries_several_representations_and_the_caller_chooses() {
    if !native_enabled() {
        return;
    }
    // A `.cnb` texture may hold the same image several times over, so one file
    // serves machines with different capabilities. The caller's predicate picks
    // in file order, which makes the authored order the preference order.
    let texture = CnbTextureData::new(4, 4, 1, 6, 1).expect("cube-map texture data");
    let raw = CnbTextureFormat::from_surface_format(SurfaceFormat::Color)
        .expect("the format that stores SurfaceFormat::Color");
    assert!(
        CnbTextureFormat::is_known(raw.value()).expect("known"),
        "a format this build produced must be one it knows"
    );
    let representation = texture
        .add_representation(raw)
        .expect("an uncompressed representation");

    let level_bytes = raw
        .level_byte_size(4, 4, 1)
        .expect("one level's byte count");
    // Six faces of one mip level each.
    for face in 0..6 {
        texture
            .set_level(representation, face, &vec![0_u8; level_bytes as usize])
            .expect("a face's pixels");
    }

    let chosen = texture
        .select_representation(|_| true)
        .expect("a selection")
        .expect("a caller that accepts everything gets the first representation");
    assert_eq!(chosen, representation);
    assert_eq!(
        texture
            .select_representation(|_| false)
            .expect("a selection"),
        None,
        "a caller that can upload nothing is told so rather than handed one anyway"
    );

    let bytes = encode_texture_cube(&texture, "cube").expect("encode the cube map");
    let document = CnbDocument::parse(&bytes, "cube.cnb", ReadLimits::default())
        .expect("the encoded cube map parses");
    let decoded = document.decode_texture_cube().expect("decode the cube map");
    assert_eq!(
        decoded.info().expect("shape").face_count,
        6,
        "a cube map keeps its six faces through the round trip"
    );
}

#[test]
fn the_dictionary_value_kinds_are_distinct() {
    // No library needed. `ObjectDictionary` reads a value by handing CNA the
    // kind it just reported and a destination of exactly that type's size, so
    // two kinds that collided would read each other's bytes -- a `Vector3` read
    // as a `Matrix` writes 64 bytes into a 12-byte destination. This is the
    // cheap guard on that: every kind this crate names is its own.
    let kinds = [
        ObjectValueKind::Unknown,
        ObjectValueKind::Boolean,
        ObjectValueKind::Int32,
        ObjectValueKind::Single,
        ObjectValueKind::Double,
        ObjectValueKind::Text,
        ObjectValueKind::Vector2,
        ObjectValueKind::Vector3,
        ObjectValueKind::Vector4,
        ObjectValueKind::Matrix,
        ObjectValueKind::Quaternion,
        ObjectValueKind::Color,
        ObjectValueKind::BoundingSphere,
        ObjectValueKind::BoundingBox,
        ObjectValueKind::ForeignObject,
    ];
    let distinct: std::collections::BTreeSet<_> = kinds.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        kinds.len(),
        "every dictionary value kind must be its own"
    );
    assert_eq!(
        kinds.len(),
        15,
        "the ABI names fifteen kinds; a new one needs an arm in `value` and `array` \
         before it is added here"
    );
}
