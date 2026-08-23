import importlib.util
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "cna_api_verify", ROOT / "tools/api-compat/verify.py"
)
VERIFY = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(VERIFY)
RULES = json.loads((ROOT / "tools/api-compat/mapping-rules.json").read_text(encoding="utf-8"))


class VerifierMappingTests(unittest.TestCase):
    @staticmethod
    def _empty_actual(kind, members, *, traits=(), drop=False):
        return {
            "kind": kind, "members": members, "traitMembers": {},
            "traits": set(traits), "generics": [], "repr": [], "drop": drop,
            "unsafeMembers": [], "internalLeak": False, "rawHandleLeak": False,
        }

    def test_graphics_device_clear_primary_signature_is_options_vector4(self):
        device = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsDevice", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "Clear", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "options", "type": "Microsoft.Xna.Framework.Graphics.ClearOptions"},
                     {"name": "color", "type": "Microsoft.Xna.Framework.Vector4"},
                     {"name": "depth", "type": "System.Single"},
                     {"name": "stencil", "type": "System.Int32"},
                 ]},
                {"kind": "method", "name": "Clear", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "color", "type": "Microsoft.Xna.Framework.Color"},
                 ]},
            ],
        }
        members = VERIFY.mapped_members(device, RULES, {device["name"]: device})
        self.assertEqual(
            [value["type"] for value in members["Clear"]["parameters"]],
            ["&Self", "Graphics::ClearOptions", "Vector4", "f32", "i32"],
        )
        self.assertIn("ClearWithColor", members)

    def test_graphics_resource_dispose_bool_is_primary_overload(self):
        resource = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsResource", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "Dispose", "static": False,
                 "returnType": "System.Void", "genericParameters": [],
                 "parameters": [{"name": "value", "type": "System.Boolean"}]},
                {"kind": "method", "name": "Dispose", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(resource, RULES, {resource["name"]: resource})
        self.assertEqual(
            [value["type"] for value in members["Dispose"]["parameters"]],
            ["&mut Self", "bool"],
        )
        self.assertIn("DisposeWithNoArguments", members)

    def test_texture2d_from_stream_resize_signature_is_primary(self):
        texture = {
            "name": "Microsoft.Xna.Framework.Graphics.Texture2D", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "FromStream", "static": True,
                 "returnType": "Microsoft.Xna.Framework.Graphics.Texture2D",
                 "genericParameters": [], "parameters": [
                     {"name": "graphicsDevice", "type": "Microsoft.Xna.Framework.Graphics.GraphicsDevice"},
                     {"name": "stream", "type": "System.IO.Stream"},
                     {"name": "width", "type": "System.Int32"},
                     {"name": "height", "type": "System.Int32"},
                     {"name": "zoom", "type": "System.Boolean"},
                 ]},
                {"kind": "method", "name": "FromStream", "static": True,
                 "returnType": "Microsoft.Xna.Framework.Graphics.Texture2D",
                 "genericParameters": [], "parameters": [
                     {"name": "graphicsDevice", "type": "Microsoft.Xna.Framework.Graphics.GraphicsDevice"},
                     {"name": "stream", "type": "System.IO.Stream"},
                 ]},
            ],
        }
        device = {"name": "Microsoft.Xna.Framework.Graphics.GraphicsDevice", "kind": "class"}
        members = VERIFY.mapped_members(
            texture, RULES, {texture["name"]: texture, device["name"]: device}
        )
        self.assertEqual(
            [value["type"] for value in members["FromStream"]["parameters"]],
            ["&Graphics::GraphicsDevice", "&mut R", "i32", "i32", "bool"],
        )
        self.assertIn("FromStreamWithGraphicsDeviceAndStream", members)

    def test_texture2d_save_stream_projects_as_write(self):
        texture = {
            "name": "Microsoft.Xna.Framework.Graphics.Texture2D", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "SaveAsPng", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "stream", "type": "System.IO.Stream"},
                     {"name": "width", "type": "System.Int32"},
                     {"name": "height", "type": "System.Int32"},
                 ]},
            ],
        }
        members = VERIFY.mapped_members(texture, RULES, {texture["name"]: texture})
        self.assertEqual(members["SaveAsPng"]["generics"], [{"name": "W", "bounds": ["Write"]}])
        self.assertEqual(members["SaveAsPng"]["parameters"][1]["type"], "&mut W")

    def test_game_trait_disposal_contract_does_not_require_drop(self):
        name = "cna::Microsoft::Xna::Framework::Game"
        expected = {name: {
            "kind": "trait", "clrKind": "class", "clrName": "Microsoft.Xna.Framework.Game",
            "members": {"Dispose": {"name": "Dispose", "kind": "function", "origin": "method",
                                      "parameters": [{"name": "self", "type": "&mut Self"}],
                                      "returnType": "()", "generics": [], "refOut": []}},
            "flags": False, "underlyingType": None, "baseType": "System.Object",
            "interfaces": [], "allInterfaces": ["System.IDisposable"],
            "genericParameters": [], "operatorTraits": set(),
        }}
        actual = {name: self._empty_actual("trait", expected[name]["members"])}
        self.assertFalse(any(item["code"] == "DISPOSAL_MAPPING_MISMATCH"
                             for item in VERIFY.compare(expected, actual, RULES)))

    def test_game_frame_hooks_use_mutable_receivers_and_exact_returns(self):
        game = {
            "name": "Microsoft.Xna.Framework.Game", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": name, "static": False,
                 "returnType": return_type, "genericParameters": [], "parameters": []}
                for name, return_type in (
                    ("BeginRun", "System.Void"),
                    ("EndRun", "System.Void"),
                    ("BeginDraw", "System.Boolean"),
                    ("EndDraw", "System.Void"),
                )
            ],
        }
        members = VERIFY.mapped_members(game, RULES, {game["name"]: game})
        for name in ("BeginRun", "EndRun", "BeginDraw", "EndDraw"):
            self.assertEqual(members[name]["parameters"], [
                {"name": "self", "type": "&mut Self"}
            ])
        self.assertEqual(members["BeginDraw"]["returnType"], "bool")
        self.assertEqual(members["BeginRun"]["returnType"], "()")

    def test_graphics_device_disposal_contract_includes_drop(self):
        name = "cna::Microsoft::Xna::Framework::Graphics::GraphicsDevice"
        expected = {name: {
            "kind": "struct", "clrKind": "class", "clrName": "Microsoft.Xna.Framework.Graphics.GraphicsDevice",
            "members": {"Dispose": {"name": "Dispose", "kind": "function", "origin": "method",
                                      "parameters": [{"name": "self", "type": "&mut Self"},
                                                     {"name": "value", "type": "bool"}],
                                      "returnType": "Result<()>", "generics": [], "refOut": []}},
            "flags": False, "underlyingType": None, "baseType": "System.Object",
            "interfaces": [], "allInterfaces": ["System.IDisposable"],
            "genericParameters": [], "operatorTraits": set(),
        }}
        actual = {name: self._empty_actual("struct", expected[name]["members"], drop=True)}
        self.assertFalse(any(item["code"] == "DISPOSAL_MAPPING_MISMATCH"
                             for item in VERIFY.compare(expected, actual, RULES)))

    def test_enum_values_are_contract_members_but_value_storage_is_not(self):
        keys = {
            "name": "Microsoft.Xna.Framework.Input.Keys",
            "kind": "enum",
            "flags": False,
            "genericParameters": [],
            "members": [
                {"kind": "field", "name": "value__", "type": "System.Int32", "static": False},
                {
                    "kind": "field", "name": "Escape", "type": "Microsoft.Xna.Framework.Input.Keys",
                    "static": True, "constant": True, "value": "27",
                },
            ],
        }
        members = VERIFY.mapped_members(keys, RULES, {keys["name"]: keys})
        self.assertNotIn("value__", members)
        self.assertEqual(members["Escape"]["kind"], "variant")
        self.assertEqual(members["Escape"]["value"], "27")

    def test_static_value_property_maps_to_associated_constant(self):
        color = {
            "name": "Microsoft.Xna.Framework.Color",
            "kind": "struct",
            "flags": False,
            "genericParameters": [],
            "members": [
                {
                    "kind": "property", "name": "White", "type": "Microsoft.Xna.Framework.Color",
                    "static": True, "get": True, "set": False, "parameters": [],
                }
            ],
        }
        members = VERIFY.mapped_members(color, RULES, {color["name"]: color})
        self.assertEqual(members["White"]["kind"], "assoc_const")
        self.assertEqual(members["White"]["type"], "Self")

    def test_curve_collection_returns_shared_key_handle_and_owned_iterator(self):
        key = {"name": "Microsoft.Xna.Framework.CurveKey", "kind": "class"}
        collection = {
            "name": "Microsoft.Xna.Framework.CurveKeyCollection", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Item", "type": key["name"],
                 "static": False, "get": True, "set": False,
                 "parameters": [{"name": "index", "type": "System.Int32"}]},
                {"kind": "method", "name": "GetEnumerator", "static": False,
                 "returnType": "System.Collections.Generic.IEnumerator`1[Microsoft.Xna.Framework.CurveKey]",
                 "genericParameters": [], "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(
            collection, RULES, {key["name"]: key, collection["name"]: collection}
        )
        self.assertEqual(members["Item"]["returnType"], "CurveKey")
        self.assertEqual(members["GetEnumerator"]["returnType"], "IntoIter<CurveKey>")

    def test_retained_object_property_uses_shared_owned_type_erasure(self):
        resource = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsResource", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Tag", "static": False,
                 "type": "System.Object", "get": True, "set": True, "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(resource, RULES, {resource["name"]: resource})
        projected = "Option<Arc<dyn Any+Send+Sync>>"
        self.assertEqual(members["Tag"]["returnType"], projected)
        self.assertEqual(members["SetTag"]["parameters"][-1]["type"], projected)

    def test_unbound_graphics_resource_device_is_optional_borrow(self):
        device = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsDevice", "kind": "class",
        }
        resource = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsResource", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "GraphicsDevice", "static": False,
                 "type": device["name"], "get": True, "set": False, "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(
            resource, RULES, {resource["name"]: resource, device["name"]: device}
        )
        self.assertEqual(members["GraphicsDevice"]["returnType"], "Option<&Graphics::GraphicsDevice>")

    def test_comparison_emits_every_structural_category(self):
        type_name = "cna::Microsoft::Xna::Framework::Derived"
        expected = {
            type_name: {
                "kind": "struct", "clrKind": "struct", "clrName": "Microsoft.Xna.Framework.Derived",
                "members": {
                    "new": {
                        "name": "new", "kind": "function", "origin": "constructor", "overload": 1,
                        "parameters": [
                            {"name": "value", "type": "&i32"},
                            {"name": "count", "type": "i32"},
                        ], "returnType": "Self",
                        "generics": [{"name": "T", "bounds": ["Copy"]}], "refOut": [0],
                    },
                    "Foo": {"name": "Foo", "kind": "assoc_const", "origin": "property-getter", "overload": 0, "type": "f32"},
                    "AddChangedHandler": {
                        "name": "AddChangedHandler", "kind": "function", "origin": "event", "overload": 0,
                        "parameters": [], "returnType": "u64", "generics": [], "refOut": [],
                    },
                    "Invoke": {
                        "name": "Invoke", "kind": "function", "origin": "delegate", "overload": 0,
                        "parameters": [], "returnType": "()", "generics": [], "refOut": [],
                    },
                },
                "flags": True, "underlyingType": "u32",
                "baseType": "Microsoft.Xna.Framework.Base",
                "interfaces": ["Microsoft.Xna.Framework.ITest"],
                "allInterfaces": ["System.IDisposable"],
                "genericParameters": [{"name": "T", "bounds": ["Copy"]}],
                "operatorTraits": {"Mul<f32>"},
            },
            "cna::Microsoft::Xna::Framework::Base": {"kind": "trait"},
            "cna::Microsoft::Xna::Framework::ITest": {"kind": "trait"},
        }
        actual = {
            type_name: {
                "kind": "struct",
                "members": {
                    "new": {
                        "name": "new", "kind": "function",
                        "parameters": [
                            {"name": "value", "type": "&mut i32"},
                            {"name": "count", "type": "u32"},
                        ], "returnType": "()",
                        "generics": [{"name": "U", "bounds": []}],
                    },
                    "Foo": {"name": "Foo", "kind": "function"},
                },
                "traitMembers": {}, "traits": set(), "generics": [{"name": "T", "bounds": []}], "repr": [], "drop": False,
                "unsafeMembers": [], "internalLeak": False, "rawHandleLeak": False,
            }
        }
        findings = VERIFY.compare(expected, actual, RULES)
        categories = {category for item in findings for category in item["categories"]}
        required = {
            "BASE_PROJECTION_MISMATCH", "TRAIT_MISMATCH", "INTERFACE_MISMATCH",
            "PARAMETER_MISMATCH", "RETURN_TYPE_MISMATCH", "GENERIC_MISMATCH",
            "GENERIC_BOUND_MISMATCH", "REF_OUT_MAPPING_MISMATCH", "FLAGS_MAPPING_MISMATCH",
            "DELEGATE_MAPPING_MISMATCH", "EVENT_MAPPING_MISMATCH",
            "DISPOSAL_MAPPING_MISMATCH", "CONSTRUCTOR_MAPPING_MISMATCH",
            "OVERLOAD_MAPPING_MISMATCH", "PROPERTY_MAPPING_MISMATCH",
        }
        self.assertTrue(required.issubset(categories), sorted(required - categories))

    def test_enum_representation_and_value_are_compared(self):
        name = "cna::Microsoft::Xna::Framework::Input::Keys"
        expected = {
            name: {
                "kind": "enum", "clrKind": "enum", "clrName": "Microsoft.Xna.Framework.Input.Keys",
                "members": {
                    "Escape": {"name": "Escape", "kind": "variant", "origin": "enum-value", "overload": 0, "value": "27"}
                },
                "flags": False, "underlyingType": "i32", "baseType": "System.Enum",
                "interfaces": [], "allInterfaces": [], "genericParameters": [], "operatorTraits": set(),
            }
        }
        actual = {
            name: {
                "kind": "enum", "members": {"Escape": {"name": "Escape", "kind": "variant", "value": "28"}},
                "traitMembers": {}, "traits": {"Copy", "Clone", "PartialEq"}, "generics": [],
                "repr": ["u32"], "drop": False, "unsafeMembers": [], "internalLeak": False, "rawHandleLeak": False,
            }
        }
        findings = VERIFY.compare(expected, actual, RULES)
        self.assertEqual(sum(item["code"] == "ENUM_VALUE_MISMATCH" for item in findings), 2)


if __name__ == "__main__":
    unittest.main()
