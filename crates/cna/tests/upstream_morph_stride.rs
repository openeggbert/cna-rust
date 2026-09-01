//! RUST-UPSTREAM-024: `cna_morph_target_data_ext_create` accepts three of the
//! eleven canonical vertex strides.
//!
//! CNA's renderer keeps one table of what a stride means --
//! `InferredLayoutForStride` in `VertexDeclarationFidelity.hpp` -- and it lists
//! eleven: 16, 20, 24, 32, 48, 52, 56, 60, 68, 76 and 80. The blender that
//! actually applies morph deltas, `BlendMorphTargetsEXT`, queries that table.
//! The C API's own `ValidateMorphShape` does not: it restates a literal
//! `{32, 52, 56}`, which is the list the blender was changed *away from*
//! (GLTF-278) after it went stale.
//!
//! So the strides an ordinary physically based glTF mesh now gets -- 48
//! unskinned, 68 skinned -- cannot be handed to the C entry point at all, even
//! though the engine underneath knows exactly where their normals live.
//!
//! This test does not assert a fix. It records which strides the route takes
//! today, so that when the list changes the measurement changes with it.

use cna::extensions::models::{MorphTargetData, MorphTargetDelta, MorphWeightTrack};
use cna::Microsoft::Xna::Framework::Vector3;

/// Every stride `InferredLayoutForStride` has a canonical layout for.
const CANONICAL_STRIDES: [i32; 11] = [16, 20, 24, 32, 48, 52, 56, 60, 68, 76, 80];

/// The three the C API's literal list allows.
const ACCEPTED_BY_THE_C_API: [i32; 3] = [32, 52, 56];

const VERTICES: usize = 3;

fn morph_data_of_stride(stride: i32) -> cna::Result<MorphTargetData> {
    // The bytes are never read here -- validation happens before any blend --
    // so zeroed vertices of the right shape are the whole requirement.
    let base = vec![0_u8; stride as usize * VERTICES];
    let targets = vec![MorphTargetDelta {
        position_deltas: vec![Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0); VERTICES],
        normal_deltas: vec![Vector3::from_x_and_y_and_z(0.0, 1.0, 0.0); VERTICES],
    }];
    MorphTargetData::new(&base, stride, &targets, &[0.0], &MorphWeightTrack::default())
}

#[test]
fn eight_of_the_eleven_canonical_strides_are_refused() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }

    let mut accepted = Vec::new();
    let mut refused = Vec::new();
    for stride in CANONICAL_STRIDES {
        match morph_data_of_stride(stride) {
            Ok(data) => {
                // An accepted stride must also report itself back unchanged;
                // "accepted" that silently rewrote the stride would be worse
                // than a refusal.
                assert_eq!(
                    data.stride().ok(),
                    Some(stride),
                    "stride {stride} was accepted but does not read back as itself"
                );
                accepted.push(stride);
            }
            Err(error) => refused.push((stride, error.to_string())),
        }
    }

    println!("NOTE: accepted {accepted:?}");
    for (stride, message) in &refused {
        println!("NOTE: refused {stride}: {message}");
    }

    assert_eq!(
        accepted, ACCEPTED_BY_THE_C_API,
        "the set of strides cna_morph_target_data_ext_create accepts has changed. \
         If it now covers the canonical table, RUST-UPSTREAM-024 is fixed and both \
         this test and MorphTargetData::new's documentation should say so."
    );

    // The two the finding is actually about: what a metallic-roughness glTF
    // mesh gets. Named separately so a regression on them is legible even if
    // the rest of the list moves.
    for pbr_stride in [48, 68] {
        let message = refused
            .iter()
            .find(|(stride, _)| *stride == pbr_stride)
            .map(|(_, message)| message.as_str())
            .unwrap_or_else(|| panic!("stride {pbr_stride} is no longer refused"));
        assert!(
            message.contains("32, 52, or 56"),
            "stride {pbr_stride} was refused for a reason other than the stale literal \
             list, which would mean the defect moved rather than persisted: {message}"
        );
    }
}
