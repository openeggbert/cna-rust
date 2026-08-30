//! Canonical CNA ABI admission policy.
//!
//! `docs/c-api/ABI_VERSIONING.md` fixes both the encoding and the rule a
//! consumer must apply: *"A consumer must reject a different major and may
//! require a minimum minor."* The two halves of that rule behave differently
//! before and after ABI 1.0, and this module encodes the difference rather
//! than a single comparison.
//!
//! While the canonical major is `0` the ABI is experimental, and CNA ships an
//! incompatible change **as a minor increment** — that is exactly how `0.20.0`
//! removed eleven renderer identities and moved `CNA_GRAPHICS_RENDERER_MAXIMUM`
//! from 50 to 49. CNA has also moved the minor for every purely additive
//! generation since `0.4.0`, so a minor increment does not distinguish the two
//! cases. A `0.x` consumer therefore cannot admit a higher minor without
//! re-reviewing it, and this binding admits the reviewed minor exactly.
//!
//! From ABI `1.0` onward the canonical contract permits only additive,
//! backward-compatible change within a major, so a higher minor is admissible
//! and the documented "minimum minor" reading applies.
//!
//! A patch is additive within its minor in both regimes, so a library may
//! carry a higher patch than the one these declarations were reviewed against.

use cna_sys as sys;

/// Why a library's reported ABI version is not admissible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Rejection {
    /// The major differs; no compatibility is defined across a major.
    Major,
    /// The experimental `0.x` minor differs from the reviewed minor.
    ExperimentalMinor,
    /// The library predates the reviewed minor within a stable major.
    MinorTooOld,
    /// The library predates the reviewed patch within the reviewed minor.
    PatchTooOld,
}

/// Decides whether a library's reported ABI version may be loaded.
pub(crate) const fn admit(actual: u32) -> core::result::Result<(), Rejection> {
    let expected = sys::CNA_ABI_VERSION;
    let (major, minor, patch) = (
        sys::cna_abi_version_major(actual),
        sys::cna_abi_version_minor(actual),
        sys::cna_abi_version_patch(actual),
    );
    let (want_major, want_minor, want_patch) = (
        sys::cna_abi_version_major(expected),
        sys::cna_abi_version_minor(expected),
        sys::cna_abi_version_patch(expected),
    );
    if major != want_major {
        return Err(Rejection::Major);
    }
    if want_major == 0 {
        if minor != want_minor {
            return Err(Rejection::ExperimentalMinor);
        }
    } else if minor < want_minor {
        return Err(Rejection::MinorTooOld);
    }
    if minor == want_minor && patch < want_patch {
        return Err(Rejection::PatchTooOld);
    }
    Ok(())
}

impl Rejection {
    /// A stable one-line explanation of the canonical rule that was violated.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Major => "a different ABI major defines no compatibility",
            Self::ExperimentalMinor => {
                "the experimental 0.x ABI ships incompatible change as a minor increment, \
                 so only the reviewed minor is admitted"
            }
            Self::MinorTooOld => "the library predates the reviewed minor",
            Self::PatchTooOld => "the library predates the reviewed patch",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{admit, Rejection};
    use cna_sys as sys;

    const fn version(major: u32, minor: u32, patch: u32) -> u32 {
        sys::cna_abi_version_encode(major, minor, patch)
    }

    #[test]
    fn reviewed_version_is_admitted() {
        assert_eq!(admit(sys::CNA_ABI_VERSION), Ok(()));
        assert_eq!(sys::CNA_ABI_VERSION, version(0, 20, 0));
    }

    #[test]
    fn a_different_major_is_rejected() {
        assert_eq!(admit(version(1, 20, 0)), Err(Rejection::Major));
        assert_eq!(admit(version(0xFFFF, 20, 0)), Err(Rejection::Major));
    }

    #[test]
    fn an_experimental_minor_must_match_exactly() {
        assert_eq!(admit(version(0, 19, 0)), Err(Rejection::ExperimentalMinor));
        assert_eq!(admit(version(0, 21, 0)), Err(Rejection::ExperimentalMinor));
        assert_eq!(admit(version(0, 7, 0)), Err(Rejection::ExperimentalMinor));
    }

    #[test]
    fn a_higher_patch_within_the_reviewed_minor_is_additive() {
        assert_eq!(admit(version(0, 20, 1)), Ok(()));
        assert_eq!(admit(version(0, 20, 255)), Ok(()));
    }

    #[test]
    fn encoding_matches_the_canonical_layout() {
        assert_eq!(version(0, 20, 0), 0x0000_1400);
        assert_eq!(version(1, 2, 3), 0x0001_0203);
        assert_eq!(sys::cna_abi_version_major(0x0001_0203), 1);
        assert_eq!(sys::cna_abi_version_minor(0x0001_0203), 2);
        assert_eq!(sys::cna_abi_version_patch(0x0001_0203), 3);
    }
}
