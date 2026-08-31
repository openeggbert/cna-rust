//! CNA's `.cnb` content container, end to end.
//!
//! The fixture is built here rather than checked in: encoding texture data and
//! parsing the result back is the round trip that proves both directions, and
//! it needs no asset the repository is not allowed to carry.

use cna::extensions::content::{
    AssetTypeId, CnbDocument, CnbEffectKind, CnbMaterial, CnbMaterialTexture, CnbModel,
    CnbModelPart, CnbTextureData, ReadLimits,
};
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
