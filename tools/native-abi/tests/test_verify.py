"""Mutation tests for the native ABI verifier.

A verifier is only worth the differences it can see. Each test here mutates one
property of a declaration and asserts the verifier notices, so a future
simplification that quietly stops distinguishing signedness or pointer depth
fails here rather than passing a wrong binding.
"""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "cna_native_abi_verify", ROOT / "tools/native-abi/verify.py"
)
VERIFY = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(VERIFY)
MANIFEST = json.loads((ROOT / "tools/native-abi/bindings.json").read_text(encoding="utf-8"))


def canonical_pair(c_type: str, rust_type: str) -> tuple[dict, dict]:
    return VERIFY.canonical_c_type(c_type), VERIFY.canonical_rust_type(rust_type)


class PrototypeDiscriminationTests(unittest.TestCase):
    """Every property the ABI depends on must change the canonical form."""

    def test_an_exact_prototype_agrees(self):
        c_value, rust_value = canonical_pair("const uint32_t *", "*const u32")
        self.assertEqual(c_value, rust_value)

    def test_a_wrong_return_type_disagrees(self):
        c_value, rust_value = canonical_pair("CNA_Result", "u64")
        self.assertNotEqual(c_value, rust_value)

    def test_wrong_signedness_disagrees(self):
        c_value, rust_value = canonical_pair("int32_t", "u32")
        self.assertNotEqual(c_value, rust_value)

    def test_wrong_width_disagrees(self):
        c_value, rust_value = canonical_pair("uint64_t", "u32")
        self.assertNotEqual(c_value, rust_value)

    def test_wrong_pointer_depth_disagrees(self):
        c_value, rust_value = canonical_pair("uint32_t *", "*mut *mut u32")
        self.assertNotEqual(c_value, rust_value)

    def test_wrong_pointee_constness_disagrees(self):
        c_value, rust_value = canonical_pair("const uint32_t *", "*mut u32")
        self.assertNotEqual(c_value, rust_value)

    def test_a_semantic_handle_is_not_a_bare_handle(self):
        # cna-sys spells the exact handle alias the header uses, so a route that
        # takes a VideoPlayer cannot be declared as taking any handle.
        c_value, rust_value = canonical_pair("CNA_VideoPlayerHandle", "CNA_Handle")
        self.assertNotEqual(c_value, rust_value)

    def test_cna_bool_is_not_a_bare_byte(self):
        c_value, rust_value = canonical_pair("CNA_Bool", "u8")
        self.assertNotEqual(c_value, rust_value)
        self.assertEqual(*canonical_pair("CNA_Bool", "CNA_Bool"))

    def test_a_versioned_descriptor_is_not_an_opaque_pointer(self):
        c_value, rust_value = canonical_pair("CNA_VideoFrameEXT *", "*mut c_void")
        self.assertNotEqual(c_value, rust_value)


class HeaderAgreementTests(unittest.TestCase):
    def header_directory(self, text: str) -> Path:
        # addCleanup rather than enterContext: the latter needs Python 3.11 and
        # nothing else here does.
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        directory = Path(temporary.name)
        (directory / "probe.h").write_text(text, encoding="utf-8")
        return directory

    def test_declarations_reads_arity_from_the_header(self):
        directory = self.header_directory(
            "CNA_C_API CNA_Result cna_probe_none(void);\n"
            "CNA_C_API CNA_Result cna_probe_two(uint32_t a, uint32_t *b);\n"
        )
        found = VERIFY.declarations(directory)
        self.assertEqual(found, {"cna_probe_none": 0, "cna_probe_two": 2})

    def test_a_removed_symbol_is_absent_from_the_header_set(self):
        directory = self.header_directory("CNA_C_API CNA_Result cna_probe_kept(void);\n")
        found = VERIFY.declarations(directory)
        self.assertNotIn("cna_probe_removed", found)

    def test_a_function_pointer_parameter_counts_as_one_parameter(self):
        directory = self.header_directory(
            "CNA_C_API CNA_Result cna_probe_callback("
            "CNA_Handle game, CNA_Result (*handler)(CNA_Handle, void *), void *context);\n"
        )
        self.assertEqual(VERIFY.declarations(directory)["cna_probe_callback"], 3)


class ManifestAgreementTests(unittest.TestCase):
    def test_the_manifest_and_cna_sys_declare_exactly_the_same_functions(self):
        self.assertEqual(VERIFY.unaudited_declarations(set(MANIFEST["symbols"])), [])

    def test_an_extra_cna_sys_declaration_is_reported(self):
        expected = set(MANIFEST["symbols"]) - {"cna_get_abi_version"}
        findings = VERIFY.unaudited_declarations(expected)
        self.assertIn(
            {"code": "UNAUDITED_DECLARATION", "symbol": "cna_get_abi_version"}, findings
        )

    def test_a_manifest_symbol_with_no_declaration_is_reported(self):
        findings = VERIFY.unaudited_declarations(set(MANIFEST["symbols"]) | {"cna_not_declared"})
        self.assertIn({"code": "MISSING_DECLARATION", "symbol": "cna_not_declared"}, findings)

    def test_every_probed_callback_is_declared_by_cna_sys(self):
        source = (ROOT / "crates/cna-sys/src/lib.rs").read_text(encoding="utf-8")
        for callback in MANIFEST["callbackSignatures"]:
            self.assertIn(f"pub type {callback} =", source, callback)

    def test_every_probed_layout_is_declared_by_cna_sys(self):
        source = (ROOT / "crates/cna-sys/src/lib.rs").read_text(encoding="utf-8")
        for layout in MANIFEST["layouts"]:
            self.assertIn(f"pub struct {layout} {{", source, layout)


class LayoutFieldCoverageTests(unittest.TestCase):
    """A field missing from both the Rust struct and the manifest must be seen.

    Offsets and `sizeof` cannot catch that on their own: trailing padding can
    absorb the missing field exactly, so every listed offset and the total size
    still agree. `CNA_CnbReadLimits` did precisely that -- seven C fields
    against six declared ones, all offsets correct, both 48 bytes -- and passed
    the gate until this check existed.
    """

    def cna_root(self) -> Path | None:
        import os

        root = os.environ.get("CNA_ROOT")
        return Path(root) if root else None

    def test_a_dropped_trailing_field_is_reported(self):
        root = self.cna_root()
        if root is None:
            self.skipTest("CNA_ROOT is not set")
        manifest = {
            "layouts": {
                "CNA_CnbReadLimits": list(MANIFEST["layouts"]["CNA_CnbReadLimits"])[:-1]
            }
        }
        checked, findings = VERIFY.layout_field_coverage(root, manifest)
        self.assertEqual(checked, 1)
        self.assertEqual([x["code"] for x in findings], ["LAYOUT_FIELD_SET_MISMATCH"])

    def test_a_reordered_field_list_is_reported(self):
        root = self.cna_root()
        if root is None:
            self.skipTest("CNA_ROOT is not set")
        fields = list(MANIFEST["layouts"]["CNA_CnbReadLimits"])
        fields[2], fields[3] = fields[3], fields[2]
        _, findings = VERIFY.layout_field_coverage(root, {"layouts": {"CNA_CnbReadLimits": fields}})
        self.assertEqual([x["code"] for x in findings], ["LAYOUT_FIELD_SET_MISMATCH"])

    def test_the_checked_in_manifest_names_every_field(self):
        root = self.cna_root()
        if root is None:
            self.skipTest("CNA_ROOT is not set")
        checked, findings = VERIFY.layout_field_coverage(root, MANIFEST)
        self.assertEqual(findings, [])
        self.assertEqual(checked, len(MANIFEST["layouts"]))

    def test_a_prefix_named_neighbour_is_not_mistaken_for_the_type(self):
        # `-ast-dump-filter` matches by prefix, so CNA_Point also prints
        # CNA_PointLightEXT. Reporting the neighbour's fields would be a false
        # failure that trains readers to ignore this gate.
        root = self.cna_root()
        if root is None:
            self.skipTest("CNA_ROOT is not set")
        _, findings = VERIFY.layout_field_coverage(
            root, {"layouts": {"CNA_Point": MANIFEST["layouts"]["CNA_Point"]}}
        )
        self.assertEqual(findings, [])


class ProbeComparisonTests(unittest.TestCase):
    def test_probe_output_is_parsed_as_named_measurements(self):
        parsed = VERIFY.parse_probe_output("layout.X.size=8\nscalar.Y.align=4\n")
        self.assertEqual(parsed, {"layout.X.size": 8, "scalar.Y.align": 4})

    def test_a_moved_field_offset_is_a_difference(self):
        c_probe = VERIFY.parse_probe_output("layout.X.field=8\n")
        rust_probe = VERIFY.parse_probe_output("layout.X.field=12\n")
        self.assertNotEqual(c_probe, rust_probe)

    def test_a_constant_the_rust_probe_omits_is_a_difference(self):
        c_probe = VERIFY.parse_probe_output("constant.CNA_X=1\n")
        rust_probe = VERIFY.parse_probe_output("")
        self.assertNotEqual(c_probe.keys() | rust_probe.keys(), rust_probe.keys())


if __name__ == "__main__":
    unittest.main()


class SymbolAcquisitionTests(unittest.TestCase):
    """A resolved symbol must carry that symbol's own signature.

    No other check sees this mistake: pairing a field with another route's
    alias resolves a symbol that exists, loads without complaint, and then
    calls it through the wrong prototype.
    """

    @staticmethod
    def probe(source: str) -> list[dict]:
        with tempfile.TemporaryDirectory(prefix="cna-rust-acquisition-") as name:
            directory = Path(name)
            (directory / "probe.rs").write_text(source, encoding="utf-8")
            return VERIFY.acquisition_pairings(directory)

    EXPLICIT = (
        'pub(crate) alpha: sys::cna_alpha_fn,\n'
        'alpha: symbol!(cna_alpha, sys::cna_alpha_fn),\n'
    )
    INFERRED = (
        'pub(crate) beta: sys::cna_beta_fn,\n'
        'beta: symbol!(cna_beta, _),\n'
    )

    def test_the_checked_in_native_tables_pair_correctly(self):
        self.assertEqual(VERIFY.acquisition_pairings(), [])

    def test_the_gate_measures_the_real_tables(self):
        self.assertGreater(VERIFY.acquisition_count(), 0)

    def test_an_exact_explicit_pairing_agrees(self):
        self.assertEqual(self.probe(self.EXPLICIT), [])

    def test_an_inferred_type_resolves_through_the_field_declaration(self):
        self.assertEqual(self.probe(self.INFERRED), [])

    def test_a_field_paired_with_another_routes_signature_is_reported(self):
        findings = self.probe(
            'pub(crate) alpha: sys::cna_alpha_fn,\n'
            'alpha: symbol!(cna_alpha, sys::cna_gamma_fn),\n'
        )
        self.assertEqual(
            findings,
            [{
                "code": "SYMBOL_TYPE_MISMATCH",
                "file": "probe.rs",
                "field": "alpha",
                "symbol": "cna_alpha",
                "expected": "cna_alpha_fn",
                "actual": "cna_gamma_fn",
            }],
        )

    def test_an_inferred_type_over_a_wrong_field_declaration_is_reported(self):
        findings = self.probe(
            'pub(crate) beta: sys::cna_delta_fn,\n'
            'beta: symbol!(cna_beta, _),\n'
        )
        self.assertEqual([value["code"] for value in findings], ["SYMBOL_TYPE_MISMATCH"])

    def test_an_inferred_type_with_no_field_declaration_is_reported(self):
        findings = self.probe('beta: symbol!(cna_beta, _),\n')
        self.assertEqual([value["code"] for value in findings], ["UNRESOLVED_ACQUISITION_TYPE"])

    def test_the_scan_finds_every_table_field(self):
        """A gate that stops matching reports zero mismatches, which looks like a pass.

        That is not hypothetical: moving every call site from a string name to
        an identifier, so direct-link mode could name the linked declaration,
        silently took the acquisition count from 1,184 to 0 while the report
        still said zero mismatches. The floor is what makes that failure loud.
        """
        self.assertGreaterEqual(VERIFY.acquisition_count(), VERIFY.ACQUISITION_FLOOR)

    def test_a_call_syntax_the_scan_cannot_read_is_reported(self):
        # The old string form is exactly what a future edit might reintroduce.
        findings = self.probe(
            'pub(crate) alpha: sys::cna_alpha_fn,\n'
            'alpha: symbol!("cna_alpha", sys::cna_alpha_fn),\n'
        )
        self.assertEqual(findings, [], "an unreadable call form yields no findings at all")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "probe.rs"
            path.write_text(
                'pub(crate) alpha: sys::cna_alpha_fn,\n'
                'alpha: symbol!("cna_alpha", sys::cna_alpha_fn),\n',
                encoding="utf-8",
            )
            self.assertEqual(
                VERIFY.acquisition_count(Path(directory)),
                0,
                "which is why the count, not the finding list, is what the floor guards",
            )
