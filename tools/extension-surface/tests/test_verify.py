"""Mutation tests for the extension-surface gate.

The strict XNA verifier reaches zero by *removing* CNA's own members from the
XNA hierarchy, so on its own it cannot tell a member that moved behind an
extension trait from one that was deleted. This gate answers that, and these
tests plant the five ways it could be wrong into a synthetic rustdoc document
shaped like the real one: a strict type in a public module chain, an extension
trait re-exported out of a private module, and the impl that joins them.

The sixth is the defect that motivated the third gate. `PresentationMode` was
`pub` inside a private module and re-exported nowhere, so a public method
answered with a type no consumer could name. Nothing measured that.
"""

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "cna_extension_surface", ROOT / "tools/extension-surface/verify.py"
)
SURFACE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(SURFACE)


def module(name, items, visibility="public"):
    return {"name": name, "visibility": visibility, "inner": {"module": {"items": items}}}


def use(name, target, visibility="public"):
    return {"name": None, "visibility": visibility,
            "inner": {"use": {"name": name, "id": target, "is_glob": False}}}


def function(name, inputs, output, visibility="public"):
    return {"name": name, "visibility": visibility,
            "inner": {"function": {"sig": {"inputs": inputs, "output": output}}}}


def resolved(name, identifier):
    return {"resolved_path": {"name": name, "id": identifier, "args": None}}


BOOL = {"primitive": "bool"}


def document(*, inherent_member=False, trait_declares=True, implemented=True,
             trait_public=True, strict_leak=False, unnameable=False):
    """A crate with one strict type, one extension trait, and one moved member."""
    index = {
        # 1..4: the strict module chain, and 5 the strict type.
        "1": module("cna", ["2", "10", "20", "99"]),
        "2": module("Microsoft", ["3"]),
        "3": module("Xna", ["4"]),
        "4": module("Framework", ["5"] + (["31"] if strict_leak else [])),
        "5": {"name": "Song", "visibility": "public", "inner": {"struct": {"kind": "unit"}}},
        # 10..12: extensions, and the trait it publishes.
        "10": module("extensions", ["11"]),
        "11": module("media", ["12"] + (["14"] if unnameable else [])),
        "12": use("SongExt", 13, visibility="public" if trait_public else "crate"),
        "13": {"name": "SongExt", "visibility": "public",
               "inner": {"trait": {"items": ["15"] if trait_declares else []}}},
        "15": function("HandleText", [["self", {}]], resolved("Result", 99)),
        # 20: the private module the backend type lives in.
        "20": module("hidden", [], visibility="crate"),
        "30": {"name": "TouchPanelTestBackend", "visibility": "public",
               "inner": {"struct": {"kind": "unit"}}},
        "31": use("TouchPanelTestBackend", 30),
        "99": {"name": "Result", "visibility": "public", "inner": {"type_alias": {}}},
    }
    if unnameable:
        # A public function answering with a type no public path reaches.
        index["14"] = function("preferred_mode", [], resolved("PresentationMode", 40))
        index["40"] = {"name": "PresentationMode", "visibility": "public",
                       "span": {"filename": "crates/cna/src/game/device_manager.rs"},
                       "inner": {"enum": {"variants": []}}}
    members = []
    if implemented:
        members.append("16")
        index["16"] = function("HandleText", [["self", {}]], resolved("Result", 99),
                               visibility="default")
        index["17"] = {"name": None, "visibility": "default", "inner": {"impl": {
            "trait": {"name": "SongExt", "id": 13},
            "for": resolved("Song", 5), "items": ["16"]}}}
    if inherent_member:
        index["18"] = function("HandleText", [["self", {}]], resolved("Result", 99))
        index["19"] = {"name": None, "visibility": "default", "inner": {"impl": {
            "trait": None, "for": resolved("Song", 5), "items": ["18"]}}}
    return {"root": 1, "index": index}


MANIFEST = {
    "members": [{
        "strictType": "cna::Microsoft::Xna::Framework::Song",
        "member": "HandleText",
        "trait": "cna::extensions::media::SongExt",
        "shape": {"receiver": "self", "parameters": ["self"], "returns": "Result"},
    }],
    "types": [{
        "extensionPath": "cna::extensions::media::SongExt",
        "absentFrom": ["cna::Microsoft::Xna::Framework::TouchPanelTestBackend"],
    }],
}


def codes(report):
    return sorted(value["code"] for value in report["findings"])


class ExtensionSurface(unittest.TestCase):
    def test_a_migrated_member_behind_its_trait_passes(self):
        report = SURFACE.measure(document(), MANIFEST)
        self.assertEqual(report["findings"], [])
        self.assertEqual(report["totalDiagnostics"], 0)

    def test_a_member_returned_to_an_inherent_impl_fails(self):
        report = SURFACE.measure(document(inherent_member=True), MANIFEST)
        self.assertIn("MEMBER_STILL_INHERENT", codes(report))

    def test_a_member_dropped_from_the_trait_fails(self):
        report = SURFACE.measure(document(trait_declares=False), MANIFEST)
        self.assertIn("MEMBER_MISSING_FROM_TRAIT", codes(report))

    def test_a_trait_no_longer_implemented_for_the_strict_type_fails(self):
        report = SURFACE.measure(document(implemented=False), MANIFEST)
        self.assertIn("EXTENSION_TRAIT_NOT_IMPLEMENTED", codes(report))

    def test_a_trait_that_stops_being_exported_fails(self):
        report = SURFACE.measure(document(trait_public=False), MANIFEST)
        self.assertIn("EXTENSION_TRAIT_NOT_PUBLIC", codes(report))

    def test_an_extension_type_re_exported_into_the_strict_namespace_fails(self):
        report = SURFACE.measure(document(strict_leak=True), MANIFEST)
        self.assertIn("EXTENSION_TYPE_IN_STRICT_NAMESPACE", codes(report))

    def test_a_public_signature_naming_an_unreachable_type_fails(self):
        report = SURFACE.measure(document(unnameable=True), MANIFEST)
        finding = [f for f in report["findings"] if f["code"] == "UNNAMEABLE_PUBLIC_TYPE"]
        self.assertEqual(len(finding), 1)
        self.assertEqual(finding[0]["type"], "PresentationMode")
        self.assertEqual(finding[0]["usedBy"], ["preferred_mode"])

    def test_reachability_follows_a_re_export_out_of_a_private_module(self):
        # The shape every `cna::extensions` type has: the defining module is
        # private, so rustdoc records no canonical path and only the walk finds
        # it.
        paths = SURFACE.public_paths(document()["index"], "1")
        self.assertEqual(paths["13"], {"cna::extensions::media::SongExt"})
        self.assertNotIn("20", paths)


if __name__ == "__main__":
    unittest.main()
