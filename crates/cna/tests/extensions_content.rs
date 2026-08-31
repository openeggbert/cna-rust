//! CNA's `.cnb` content container, end to end.
//!
//! The fixture is built here rather than checked in: encoding texture data and
//! parsing the result back is the round trip that proves both directions, and
//! it needs no asset the repository is not allowed to carry.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cna::extensions::content::{
    AssetTypeId, CnbDocument, CnbEffectKind, CnbLoader, CnbLoaderRegistry, CnbMaterial,
    CnbAudioFormat, CnbGlyph, CnbMaterialTexture, CnbModel, CnbModelPart, CnbSoundEffect,
    CnbSoundEffectInfo, CnbSpriteFont, CnbSpriteFontInfo, CnbTextureData, CnbWriter,
    NativeContentManager, ReadLimits,
};
use cna::Microsoft::Xna::Framework::{Rectangle, Vector3};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::GraphicsDeviceInformation;
use cna::{CnaError, ErrorCategory, Result};

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

/// Index bytes that really address the part's vertices.
///
/// CNA validates every index against the part's vertex count, so an arbitrary
/// byte pattern is rejected rather than stored -- which is the correct
/// behaviour, and the reason this builds indices instead of noise.
fn encode_indices(element_size: u32, index_count: u32, vertex_count: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((element_size * index_count) as usize);
    for ordinal in 0..index_count {
        let vertex = ordinal % vertex_count;
        if element_size == 2 {
            bytes.extend_from_slice(&(vertex as u16).to_le_bytes());
        } else {
            bytes.extend_from_slice(&vertex.to_le_bytes());
        }
    }
    bytes
}

/// A model with a real hierarchy: a root, two children, two meshes over three
/// parts, distinct geometry per part, and one textured PBR material.
///
/// Every value is chosen to be distinguishable, so a field that is dropped,
/// reordered or defaulted shows up as a wrong value rather than as a plausible
/// one.
fn authored_model() -> CnbModel {
    let model = CnbModel::new().expect("start a model");

    // A scene graph, not a flat list: the root and two children with different
    // transforms, so parentage and per-bone transforms are both observable.
    let identity = [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0_f32,
    ];
    let mut translated = identity;
    translated[12] = 3.5;
    translated[13] = -2.25;
    translated[14] = 7.0;
    let mut scaled = identity;
    scaled[0] = 2.0;
    scaled[5] = 4.0;
    scaled[10] = 8.0;

    let root = model.add_bone("root", None, &identity).expect("root bone");
    assert_eq!(root, 0, "the first bone is index zero");
    let torso = model
        .add_bone("torso", Some(root), &translated)
        .expect("torso bone");
    let arm = model.add_bone("arm", Some(torso), &scaled).expect("arm bone");

    // Both policy facts are authored rather than inferred, so the round trip
    // can show they survive as written and are not recomputed from the shape.
    model.set_flags(true, true).expect("model flags");

    // Three parts with different vertex strides and index widths, so a part
    // read back with another part's geometry is visible.
    let mut parts = Vec::new();
    for (ordinal, (stride, vertices, indices, element_size, kind)) in [
        (12_u32, 3_u32, 3_u32, 2_u32, CnbEffectKind::Basic),
        (16_u32, 4_u32, 6_u32, 2_u32, CnbEffectKind::Pbr),
        (24_u32, 6_u32, 6_u32, 4_u32, CnbEffectKind::DualTexture),
    ]
    .into_iter()
    .enumerate()
    {
        let index = model
            .add_part(
                CnbModelPart {
                    vertex_stride: stride,
                    vertex_count: vertices,
                    index_count: indices,
                    index_element_size: element_size,
                    primitive_topology: 4,
                    primitive_count: indices / 3,
                    effect_kind: kind,
                    vertex_color_enabled: ordinal == 1,
                    unlit: ordinal == 2,
                },
                &format!("part{ordinal}"),
                "",
            )
            .expect("add part");
        let vertex_bytes: Vec<u8> = (0..stride * vertices)
            .map(|byte| (byte as u8).wrapping_mul(3).wrapping_add(ordinal as u8))
            .collect();
        let index_bytes = encode_indices(element_size, indices, vertices);
        model
            .set_part_vertex_bytes(index, &vertex_bytes)
            .expect("part vertex bytes");
        model
            .set_part_index_bytes(index, &index_bytes)
            .expect("part index bytes");
        parts.push((index, vertex_bytes, index_bytes));
    }

    // One part carries a full PBR material with two named textures, so the
    // material's per-part association and the eight-slot name space are both
    // exercised.
    model
        .set_material(
            parts[1].0,
            CnbMaterial {
                base_color_factor: [0.25, 0.5, 0.75, 1.0],
                emissive_factor: [0.1, 0.2, 0.3],
                specular_color_factor: [0.9, 0.8, 0.7],
                metallic_factor: 0.125,
                roughness_factor: 0.875,
                ior: 1.45,
                specular_factor: 0.5,
                normal_scale: 2.0,
                occlusion_strength: 0.25,
                alpha_cutoff: 0.375,
                alpha_mode: 1,
                double_sided: true,
            },
        )
        .expect("set material");
    model
        .set_material_texture(parts[1].0, CnbMaterialTexture::BaseColor, "textures/base")
        .expect("base colour texture");
    model
        .set_material_texture(parts[1].0, CnbMaterialTexture::Normal, "textures/normal")
        .expect("normal texture");

    // Two meshes over the three parts, in a deliberate draw order that is not
    // the parts' own order.
    model
        .add_mesh("body", Some(torso), &[parts[1].0 as u32, parts[0].0 as u32])
        .expect("body mesh");
    model
        .add_mesh("limb", Some(arm), &[parts[2].0 as u32])
        .expect("limb mesh");
    model
}

#[test]
fn a_model_survives_a_cnb_round_trip_with_its_exact_graph() {
    let authored = authored_model();
    let bytes = authored.encode("cna-rust model round trip").expect("encode");
    assert!(!bytes.is_empty());

    let document = CnbDocument::parse(&bytes, "model.cnb", ReadLimits::default()).expect("parse");
    assert_eq!(
        document.asset_type().expect("asset type"),
        AssetTypeId::MODEL,
        "the document names itself a model"
    );
    let model = document.decode_model().expect("decode model");

    // Counts and the two policy facts that travel with the content.
    let info = model.info().expect("model info");
    assert_eq!(info.bone_count, 3);
    assert_eq!(info.part_count, 3);
    assert_eq!(info.mesh_count, 2);
    assert_eq!(info.animation_count, 0);
    assert_eq!(info.light_count, 0);
    assert!(!info.has_skeleton, "no skinning skeleton was authored");
    assert!(
        info.has_bone_hierarchy,
        "the authored hierarchy flag survives the round trip"
    );
    assert!(
        info.applies_gltf_lighting_policy,
        "the authored lighting policy survives the round trip"
    );

    // The exact hierarchy, by name and parent, not merely the right count.
    assert_eq!(model.bone_name(0).expect("root name"), "root");
    assert_eq!(model.bone_name(1).expect("torso name"), "torso");
    assert_eq!(model.bone_name(2).expect("arm name"), "arm");
    assert_eq!(model.bone(0).expect("root").parent, None);
    assert_eq!(model.bone(1).expect("torso").parent, Some(0));
    assert_eq!(model.bone(2).expect("arm").parent, Some(1));

    // The exact transforms, element by element.
    let mut expected_root = [0.0_f32; 16];
    for i in 0..4 {
        expected_root[i * 5] = 1.0;
    }
    assert_eq!(model.bone(0).expect("root").transform, expected_root);
    let torso = model.bone(1).expect("torso").transform;
    assert_eq!((torso[12], torso[13], torso[14]), (3.5, -2.25, 7.0));
    let arm = model.bone(2).expect("arm").transform;
    assert_eq!((arm[0], arm[5], arm[10]), (2.0, 4.0, 8.0));

    // The exact mesh-to-part relationship, including the draw order, which is
    // deliberately not the parts' own order.
    assert_eq!(model.mesh_name(0).expect("mesh 0 name"), "body");
    assert_eq!(model.mesh_name(1).expect("mesh 1 name"), "limb");
    assert_eq!(model.mesh(0).expect("mesh 0").parent_bone, Some(1));
    assert_eq!(model.mesh(1).expect("mesh 1").parent_bone, Some(2));
    assert_eq!(
        model.mesh_part_indices(0).expect("body parts"),
        vec![1_u32, 0],
        "draw order is preserved, not sorted"
    );
    assert_eq!(model.mesh_part_indices(1).expect("limb parts"), vec![2_u32]);

    // Each part's own numeric state and its own geometry bytes.
    for (ordinal, (stride, vertices, indices, element_size, kind)) in [
        (12_u32, 3_u32, 3_u32, 2_u32, CnbEffectKind::Basic),
        (16_u32, 4_u32, 6_u32, 2_u32, CnbEffectKind::Pbr),
        (24_u32, 6_u32, 6_u32, 4_u32, CnbEffectKind::DualTexture),
    ]
    .into_iter()
    .enumerate()
    {
        let index = ordinal as u64;
        let part = model.part(index).expect("part");
        assert_eq!(part.vertex_stride, stride);
        assert_eq!(part.vertex_count, vertices);
        assert_eq!(part.index_count, indices);
        assert_eq!(part.index_element_size, element_size);
        assert_eq!(part.primitive_topology, 4);
        assert_eq!(part.primitive_count, indices / 3);
        assert_eq!(part.effect_kind, kind);
        assert_eq!(part.vertex_color_enabled, ordinal == 1);
        assert_eq!(part.unlit, ordinal == 2);
        assert_eq!(model.part_name(index).expect("part name"), format!("part{ordinal}"));

        let expected_vertices: Vec<u8> = (0..stride * vertices)
            .map(|byte| (byte as u8).wrapping_mul(3).wrapping_add(ordinal as u8))
            .collect();
        let expected_indices = encode_indices(element_size, indices, vertices);
        assert_eq!(
            model.part_vertex_bytes(index).expect("vertex bytes"),
            expected_vertices,
            "part {ordinal} kept its own vertices"
        );
        assert_eq!(
            model.part_index_bytes(index).expect("index bytes"),
            expected_indices,
            "part {ordinal} kept its own indices"
        );
    }

    // The material belongs to the part that was given it, with every factor.
    let material = model.material(1).expect("material");
    assert_eq!(material.base_color_factor, [0.25, 0.5, 0.75, 1.0]);
    assert_eq!(material.emissive_factor, [0.1, 0.2, 0.3]);
    assert_eq!(material.specular_color_factor, [0.9, 0.8, 0.7]);
    assert_eq!(material.metallic_factor, 0.125);
    assert_eq!(material.roughness_factor, 0.875);
    assert_eq!(material.ior, 1.45);
    assert_eq!(material.specular_factor, 0.5);
    assert_eq!(material.normal_scale, 2.0);
    assert_eq!(material.occlusion_strength, 0.25);
    assert_eq!(material.alpha_cutoff, 0.375);
    assert_eq!(material.alpha_mode, 1);
    assert!(material.double_sided);

    // Texture names come back in their own slots, and an unset slot is empty
    // rather than carrying a neighbour's name.
    assert_eq!(
        model
            .material_texture(1, CnbMaterialTexture::BaseColor)
            .expect("base colour"),
        "textures/base"
    );
    assert_eq!(
        model
            .material_texture(1, CnbMaterialTexture::Normal)
            .expect("normal"),
        "textures/normal"
    );
    assert_eq!(
        model
            .material_texture(1, CnbMaterialTexture::Emissive)
            .expect("emissive"),
        "",
        "an unset slot names nothing"
    );
    assert_eq!(
        model
            .material_texture(1, CnbMaterialTexture::Second)
            .expect("second layer"),
        "",
        "the DualTexture slot is not the base-colour slot"
    );
}

#[test]
fn a_model_document_is_not_a_texture_document() {
    let bytes = authored_model().encode("kind check").expect("encode");
    let document = CnbDocument::parse(&bytes, "model.cnb", ReadLimits::default()).expect("parse");
    // Asking a model document for a texture must be CNA's refusal, not a
    // reinterpretation of the same bytes as pixels.
    assert!(
        document.decode_texture2d().is_err(),
        "a model document must not decode as a texture"
    );

    let (width, height, rgba) = rgba_fixture();
    let texture_bytes = CnbTextureData::from_rgba8(width, height, &rgba)
        .expect("texture data")
        .encode_texture2d("kind check")
        .expect("encode texture");
    let texture_document =
        CnbDocument::parse(&texture_bytes, "texture.cnb", ReadLimits::default()).expect("parse");
    assert!(
        texture_document.decode_model().is_err(),
        "a texture document must not decode as a model"
    );
}

#[test]
fn a_truncated_model_document_is_refused_rather_than_half_decoded() {
    let bytes = authored_model().encode("truncation").expect("encode");
    for keep in [0_usize, 1, bytes.len() / 3, bytes.len() / 2, bytes.len() - 1] {
        let result = CnbDocument::parse(&bytes[..keep], "truncated.cnb", ReadLimits::default());
        assert!(
            result.is_err(),
            "a document truncated to {keep} of {} bytes must be refused",
            bytes.len()
        );
    }
}

#[test]
fn one_tightened_bound_does_not_zero_every_other_bound() {
    // CNA's contract is "start from the defaults, then lower what you want
    // tighter", so an unset field must arrive as CNA's own default and not as
    // zero. Sending zero would make every field a hard zero limit, and
    // tightening one bound would refuse every document.
    let bytes = authored_model().encode("one bound").expect("encode");
    let document = CnbDocument::parse(
        &bytes,
        "one-bound.cnb",
        ReadLimits {
            max_file_size: Some(bytes.len() as u64),
            ..ReadLimits::default()
        },
    )
    .expect("tightening only max_file_size leaves every other bound at CNA's default");
    assert_eq!(
        document.asset_type().expect("asset type"),
        AssetTypeId::MODEL
    );
}

#[test]
fn model_parsing_honours_a_caller_supplied_bound() {
    let bytes = authored_model().encode("bounded").expect("encode");
    let refused = CnbDocument::parse(
        &bytes,
        "bounded.cnb",
        ReadLimits {
            max_file_size: Some(8),
            ..ReadLimits::default()
        },
    );
    assert!(
        matches!(refused, Err(CnaError::Native { .. })),
        "a file-size bound below the document must refuse it, got {refused:?}"
    );
    // The same bytes parse when the bound admits them, so the refusal above is
    // the bound and not the content.
    CnbDocument::parse(
        &bytes,
        "bounded.cnb",
        ReadLimits {
            max_file_size: Some(bytes.len() as u64),
            ..ReadLimits::default()
        },
    )
    .expect("the same document parses within a sufficient bound");
}

/// A game's own asset type, decoded by a Rust loader.
#[derive(Debug, Eq, PartialEq)]
struct ProbeAsset {
    origin: String,
    asset_name: String,
    container: (u16, u16),
}

/// Counts how often the loader ran, so a load that never reached Rust is
/// distinguishable from one that did.
struct CountingLoader {
    calls: Arc<AtomicUsize>,
    fail: bool,
    panic: bool,
}

impl CnbLoader for CountingLoader {
    fn load(
        &self,
        document: &CnbDocument,
        asset_name: &str,
    ) -> cna::Result<Arc<dyn std::any::Any + Send + Sync>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !(self.fail && self.panic),
            "a loader is either failing or panicking, not both"
        );
        if self.panic {
            panic!("a loader panic must not unwind into C");
        }
        if self.fail {
            return Err(CnaError::InvalidInput("this loader refuses every document"));
        }
        // The document is borrowed for exactly this call, so everything the
        // asset keeps is copied out here.
        Ok(Arc::new(ProbeAsset {
            origin: document.origin()?,
            asset_name: asset_name.to_owned(),
            container: document.container_version()?,
        }))
    }
}

/// A device and a native content manager, which the loader chain needs.
///
/// Both come from `GraphicsDevice::new`, so this whole test runs with no
/// `Game` in the process at all.
fn loader_host() -> Option<(GraphicsDevice, NativeContentManager)> {
    let device = independent_device_or_skip(|| {
        let adapter = GraphicsDeviceInformation::new().Adapter();
        let parameters = PresentationParameters::new();
        parameters.SetBackBufferWidth(64);
        parameters.SetBackBufferHeight(64);
        GraphicsDevice::new(&adapter, GraphicsProfile::Reach, &parameters)
    })?;
    let manager =
        NativeContentManager::new(&device, "").expect("native content manager on that device");
    Some((device, manager))
}

/// A device with no `Game` anywhere, or `None` when this renderer cannot make one.
///
/// EasyGL's context needs a platform surface, so every GL-family renderer
/// refuses `cna_graphics_device_create` with a `Platform` failure naming the
/// missing window, while `HEADLESS` creates one happily. That is a renderer
/// capability rather than a binding fault, so it is reported and skipped --
/// but *only* that exact refusal. Any other failure is still a failure, which
/// is what keeps this from quietly hiding a regression.
fn independent_device_or_skip(build: impl FnOnce() -> Result<GraphicsDevice>) -> Option<GraphicsDevice> {
    match build() {
        Ok(device) => Some(device),
        Err(CnaError::Native {
            category: ErrorCategory::Platform,
            ref message,
            ..
        }) if message.contains("platform window id") => {
            println!("this renderer cannot create a device without a window: {message}");
            None
        }
        Err(error) => panic!("independent GraphicsDevice construction failed: {error}"),
    }
}


/// A document of a game's own type, carrying the canonical name CNA checks.
fn custom_document(type_name: &str, content_name: &str) -> Vec<u8> {
    let asset_type = AssetTypeId::custom(type_name).expect("mint a custom asset type");
    let writer = CnbWriter::new(asset_type, 1).expect("writer");
    writer
        .set_metadata(type_name, content_name)
        .expect("metadata");
    writer.build().expect("build")
}

#[test]
fn a_rust_loader_decodes_a_game_s_own_asset_type() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let Some((_device, manager)) = loader_host() else { return };
    let type_name = "CnaRust.Test.RoundTripAsset";
    let asset_type = AssetTypeId::custom(type_name).expect("custom identity");

    let calls = Arc::new(AtomicUsize::new(0));
    let registration = CnbLoaderRegistry::register(
        type_name,
        Arc::new(CountingLoader {
            calls: Arc::clone(&calls),
            fail: false,
            panic: false,
        }),
    )
    .expect("register a Rust loader");
    assert_eq!(registration.asset_type(), asset_type);
    assert!(CnbLoaderRegistry::is_registered(asset_type).expect("registered"));
    assert_eq!(
        CnbLoaderRegistry::registered_type_name(asset_type).expect("registered name"),
        type_name,
        "CNA keeps the canonical name it dispatches on"
    );

    let bytes = custom_document(type_name, "round-trip");
    let document =
        CnbDocument::parse(&bytes, "round-trip.cnb", ReadLimits::default()).expect("parse");
    assert_eq!(document.asset_type().expect("asset type"), asset_type);

    let loader = CnbLoaderRegistry::resolve_for_document(&document).expect("resolve");
    let object = loader
        .invoke(&document, &manager, "round-trip")
        .expect("invoke the Rust loader");
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the loader really ran");

    let asset = object
        .downcast_ref::<ProbeAsset>()
        .expect("the object is the Rust value the loader built");
    assert_eq!(asset.asset_name, "round-trip");
    assert_eq!(asset.origin, "round-trip.cnb");
    assert_eq!(
        asset.container,
        document.container_version().expect("container version"),
        "the loader read the borrowed document, not a default"
    );

    // Dropping the registration withdraws it and releases what it produced.
    drop(registration);
    assert!(!CnbLoaderRegistry::is_registered(asset_type).expect("withdrawn"));
    assert!(
        CnbLoaderRegistry::resolve_for_document(&document).is_err(),
        "a withdrawn loader no longer resolves"
    );
}

#[test]
fn an_ambiguous_custom_typed_document_cannot_even_be_authored() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let type_name = "CnaRust.Test.NameCheckedAsset";
    let asset_type = AssetTypeId::custom(type_name).expect("custom identity");
    let calls = Arc::new(AtomicUsize::new(0));
    let _registration = CnbLoaderRegistry::register(
        type_name,
        Arc::new(CountingLoader {
            calls: Arc::clone(&calls),
            fail: false,
            panic: false,
        }),
    )
    .expect("register");

    // A file claiming this numeric identity while naming a different type
    // cannot even be authored: CNA checks the name against the identifier at
    // build time and says so, rather than producing a file that would be
    // refused later. The guarantee is therefore stronger than the load-time
    // check alone, and this is where it is observable.
    let writer = CnbWriter::new(asset_type, 1).expect("writer");
    writer
        .set_metadata("CnaRust.Test.ImpostorAsset", "impostor")
        .expect("metadata is recorded; the identity check happens at build");
    let refused = writer.build();
    assert!(
        matches!(&refused, Err(CnaError::Native { message, .. })
            if message.contains("hash collision")),
        "authoring a name that does not hash to the identifier must be refused, got {refused:?}"
    );

    // The other way to reach the load-time check -- a custom-typed file
    // carrying no canonical name at all -- is refused at build time too, and
    // the message says exactly what to call instead.
    let anonymous = CnbWriter::new(asset_type, 1).expect("writer");
    let nameless = anonymous.build();
    assert!(
        matches!(&nameless, Err(CnaError::Native { message, .. })
            if message.contains("must carry its canonical type name")),
        "a custom-typed file with no canonical name must be refused, got {nameless:?}"
    );

    // So both halves of the collision defence are enforced where the file is
    // authored, not only where it is loaded: this API cannot produce an
    // ambiguous custom-typed document at all. The load-time refusal upstream
    // documents still matters -- for a file some other toolchain wrote -- but
    // it is not reachable from here, and this test does not pretend otherwise.
    assert_eq!(calls.load(Ordering::SeqCst), 0, "the loader never ran");
}

#[test]
fn a_loader_failure_and_a_loader_panic_are_both_contained() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let Some((_device, manager)) = loader_host() else { return };
    for (label, fail, panic) in [("failing", true, false), ("panicking", false, true)] {
        let type_name = format!("CnaRust.Test.{label}Asset");
        let calls = Arc::new(AtomicUsize::new(0));
        let registration = CnbLoaderRegistry::register(
            &type_name,
            Arc::new(CountingLoader {
                calls: Arc::clone(&calls),
                fail,
                panic,
            }),
        )
        .expect("register");
        let bytes = custom_document(&type_name, label);
        let document =
            CnbDocument::parse(&bytes, "contained.cnb", ReadLimits::default()).expect("parse");
        let loader = CnbLoaderRegistry::resolve_for_document(&document).expect("resolve");
        let result = loader.invoke(&document, &manager, label);
        assert!(result.is_err(), "a {label} loader fails the load");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "the {label} loader ran");
        // The process is still usable, which is the point of containing a
        // panic rather than letting it unwind into C.
        assert!(CnbLoaderRegistry::is_registered(registration.asset_type()).expect("still there"));
        drop(registration);
    }
}

#[test]
fn only_a_custom_asset_type_may_be_registered() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // CNA's built-in identifiers belong to CNA, and the registry has no
    // parameter by which a caller could claim one. `AssetTypeId::custom`
    // always hashes into the custom range, so a caller cannot reach a built-in
    // through this API at all -- which is the property being asserted.
    let minted = AssetTypeId::custom("Texture2D").expect("hash a built-in's name");
    assert_ne!(
        minted,
        AssetTypeId::TEXTURE2D,
        "hashing a built-in's name mints a custom identity, not the built-in"
    );
    assert!(minted.is_custom().expect("custom range"));
    assert!(!AssetTypeId::TEXTURE2D.is_custom().expect("built-in range"));
}

#[test]
fn a_loader_lookup_without_a_document_finds_nothing_when_nothing_is_registered() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let asset_type =
        AssetTypeId::custom("CnaRust.Test.NeverRegisteredAsset").expect("custom identity");
    assert!(!CnbLoaderRegistry::is_registered(asset_type).expect("not registered"));
    assert!(
        CnbLoaderRegistry::find(asset_type).expect("find").is_none(),
        "absence is an ordinary answer, not a refusal"
    );
    assert_eq!(
        CnbLoaderRegistry::registered_type_name(asset_type).expect("name"),
        "",
        "nothing registered names nothing"
    );
}

#[test]
fn a_sprite_font_survives_a_cnb_round_trip_with_its_exact_glyphs() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let font = CnbSpriteFont::new().expect("start a font");
    font.set_info(CnbSpriteFontInfo {
        glyph_count: 0,
        line_spacing: 23,
        spacing: 1.5,
        // CNA requires the fallback glyph to be one the font actually has, so
        // this is the first authored character rather than an arbitrary one.
        default_character: Some(u16::from(b'A')),
    })
    .expect("metrics");

    // Distinguishable glyphs: a glyph read back with a neighbour's rectangle
    // or kerning is visible rather than plausible.
    let authored: Vec<CnbGlyph> = "Aq\u{00e9}\u{4e2d}"
        .encode_utf16()
        .enumerate()
        .map(|(ordinal, character)| {
            let ordinal = ordinal as i32;
            CnbGlyph {
                character,
                glyph_bounds: Rectangle::new(ordinal * 10, 4, 8 + ordinal, 12 + ordinal),
                cropping: Rectangle::new(ordinal, -ordinal, 3 + ordinal, 5 + ordinal),
                kerning: Vector3::from_x_and_y_and_z(
                    ordinal as f32 * 0.5,
                    7.25 + ordinal as f32,
                    -1.5 - ordinal as f32,
                ),
            }
        })
        .collect();
    for (ordinal, glyph) in authored.iter().enumerate() {
        assert_eq!(
            font.add_glyph(*glyph).expect("add glyph"),
            ordinal as u64,
            "glyphs are appended in order"
        );
    }

    // An atlas the glyph rectangles index into.
    let (width, height, rgba) = rgba_fixture();
    let atlas = CnbTextureData::from_rgba8(width, height, &rgba).expect("atlas");
    font.set_atlas(&atlas).expect("set atlas");

    let bytes = font.encode("font round trip").expect("encode");
    let document = CnbDocument::parse(&bytes, "font.cnb", ReadLimits::default()).expect("parse");
    assert_eq!(
        document.asset_type().expect("asset type"),
        AssetTypeId::SPRITE_FONT
    );
    let decoded = document.decode_sprite_font().expect("decode font");

    let info = decoded.info().expect("decoded metrics");
    assert_eq!(info.glyph_count, authored.len() as u64);
    assert_eq!(info.line_spacing, 23);
    assert_eq!(info.spacing, 1.5);
    assert_eq!(info.default_character, Some(u16::from(b'A')));

    for (ordinal, expected) in authored.iter().enumerate() {
        let glyph = decoded.glyph(ordinal as u64).expect("decoded glyph");
        assert_eq!(glyph, *expected, "glyph {ordinal} survives exactly");
    }

    // The atlas comes back as its own owned texture with the same pixels.
    let decoded_atlas = decoded.atlas().expect("decoded atlas");
    let atlas_info = decoded_atlas.info().expect("atlas info");
    assert_eq!((atlas_info.width, atlas_info.height), (width, height));
    assert_eq!(
        decoded_atlas.level_bytes(0, 0).expect("atlas pixels"),
        rgba,
        "the atlas pixels survive the round trip"
    );
}

#[test]
fn a_fallback_glyph_the_font_does_not_have_is_refused() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // A default character with no glyph would substitute nothing, so CNA
    // refuses it at encode time rather than producing a font that draws a hole.
    let font = CnbSpriteFont::new().expect("font");
    font.set_info(CnbSpriteFontInfo {
        glyph_count: 0,
        line_spacing: 10,
        spacing: 0.0,
        default_character: Some(u16::from(b'?')),
    })
    .expect("metrics");
    font.add_glyph(CnbGlyph {
        character: u16::from(b'x'),
        glyph_bounds: Rectangle::new(0, 0, 1, 1),
        cropping: Rectangle::new(0, 0, 1, 1),
        kerning: Vector3::from_x_and_y_and_z(0.0, 1.0, 0.0),
    })
    .expect("one glyph that is not '?'");
    let (width, height, rgba) = rgba_fixture();
    font.set_atlas(&CnbTextureData::from_rgba8(width, height, &rgba).expect("atlas"))
        .expect("atlas");
    let refused = font.encode("missing fallback");
    assert!(
        matches!(&refused, Err(CnaError::Native { message, .. })
            if message.contains("default character is not one of the font's characters")),
        "a fallback glyph the font lacks must be refused, got {refused:?}"
    );
}

#[test]
fn a_font_without_a_default_character_stays_without_one() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // XNA throws on a missing character when a font has no default and
    // substitutes when it has one, so "absent" must not become "some
    // particular character".
    let font = CnbSpriteFont::new().expect("font");
    font.set_info(CnbSpriteFontInfo {
        glyph_count: 0,
        line_spacing: 10,
        spacing: 0.0,
        default_character: None,
    })
    .expect("metrics");
    font.add_glyph(CnbGlyph {
        character: u16::from(b'x'),
        glyph_bounds: Rectangle::new(0, 0, 1, 1),
        cropping: Rectangle::new(0, 0, 1, 1),
        kerning: Vector3::from_x_and_y_and_z(0.0, 1.0, 0.0),
    })
    .expect("one glyph");
    let (width, height, rgba) = rgba_fixture();
    font.set_atlas(&CnbTextureData::from_rgba8(width, height, &rgba).expect("atlas"))
        .expect("atlas");

    let bytes = font.encode("no default").expect("encode");
    let document = CnbDocument::parse(&bytes, "no-default.cnb", ReadLimits::default()).expect("parse");
    let decoded = document.decode_sprite_font().expect("decode");
    assert_eq!(
        decoded.info().expect("metrics").default_character,
        None,
        "a font with no fallback glyph keeps none"
    );
}

#[test]
fn a_sound_effect_survives_a_cnb_round_trip_with_its_exact_samples() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // A stereo PCM16 ramp: every frame differs, so a sample that moves shows.
    let frames = 64_u32;
    let channels = 2_u32;
    let mut samples = Vec::with_capacity((frames * channels * 2) as usize);
    for frame in 0..frames {
        for channel in 0..channels {
            let value = (frame as i16)
                .wrapping_mul(257)
                .wrapping_add(channel as i16 * 13);
            samples.extend_from_slice(&value.to_le_bytes());
        }
    }
    let info = CnbSoundEffectInfo {
        format: CnbAudioFormat::Pcm16,
        sample_rate: 22_050,
        channels,
        frame_count: frames,
        loop_region: Some((8, 40)),
    };
    let sound = CnbSoundEffect::new(info, &samples).expect("build sound");
    assert_eq!(sound.info().expect("authored info"), info);

    let bytes = sound.encode("sound round trip").expect("encode");
    let document = CnbDocument::parse(&bytes, "sound.cnb", ReadLimits::default()).expect("parse");
    assert_eq!(
        document.asset_type().expect("asset type"),
        AssetTypeId::SOUND_EFFECT
    );
    let decoded = document.decode_sound_effect().expect("decode sound");
    assert_eq!(decoded.info().expect("decoded info"), info);
    assert_eq!(
        decoded.samples().expect("decoded samples"),
        samples,
        "every sample byte survives"
    );
}

#[test]
fn a_sound_without_a_loop_reports_no_loop_rather_than_an_empty_one() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // The container writes "no loop" as a zero length. Handing that back as a
    // zero-length region would be a region a caller could loop on forever.
    let samples = vec![0_u8; 32];
    let info = CnbSoundEffectInfo {
        format: CnbAudioFormat::Pcm16,
        sample_rate: 8_000,
        channels: 1,
        frame_count: 16,
        loop_region: None,
    };
    let sound = CnbSoundEffect::new(info, &samples).expect("build");
    let bytes = sound.encode("no loop").expect("encode");
    let document = CnbDocument::parse(&bytes, "no-loop.cnb", ReadLimits::default()).expect("parse");
    let decoded = document.decode_sound_effect().expect("decode");
    assert_eq!(decoded.info().expect("info").loop_region, None);
}
