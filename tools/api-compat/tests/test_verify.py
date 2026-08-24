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
    def test_actual_contract_normalizes_rustdoc_string_index_keys(self):
        rustdoc = {
            "index": {
                "1": {
                    "name": "Framework",
                    "span": {"filename": "crates/cna/src/lib.rs"},
                    "inner": {"module": {"items": [2]}},
                },
                "2": {
                    "name": "AudioChannels",
                    "inner": {"enum": {"variants": []}},
                },
            }
        }
        actual = VERIFY.actual_contract(rustdoc)
        self.assertIn(
            "cna::Microsoft::Xna::Framework::AudioChannels", actual
        )

    def test_media_static_state_uses_game_context_and_is_not_a_constant(self):
        player = {
            "name": "Microsoft.Xna.Framework.Media.MediaPlayer", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "State",
                 "type": "Microsoft.Xna.Framework.Media.MediaState", "static": True,
                 "get": True, "set": False, "parameters": []},
                {"kind": "property", "name": "GameHasControl",
                 "type": "System.Boolean", "static": True,
                 "get": True, "set": False, "parameters": []},
            ],
        }
        state = {"name": "Microsoft.Xna.Framework.Media.MediaState", "kind": "enum"}
        members = VERIFY.mapped_members(player, RULES, {
            player["name"]: player, state["name"]: state,
        })
        self.assertEqual(members["State"]["parameters"], [
            {"name": "game", "type": "&GameContext"}
        ])
        self.assertEqual(members["State"]["returnType"], "Result<Media::MediaState>")
        self.assertEqual(members["GameHasControl"]["parameters"], [
            {"name": "game", "type": "&GameContext"}
        ])
        self.assertEqual(members["GameHasControl"]["returnType"], "Result<bool>")

    def test_media_datetime_visualization_and_retained_collection_mappings(self):
        visualization = {
            "name": "Microsoft.Xna.Framework.Media.VisualizationData", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Frequencies",
                 "type": "System.Collections.ObjectModel.ReadOnlyCollection`1[System.Single]",
                 "static": False, "get": True, "set": False, "parameters": []},
                {"kind": "property", "name": "Samples",
                 "type": "System.Collections.ObjectModel.ReadOnlyCollection`1[System.Single]",
                 "static": False, "get": True, "set": False, "parameters": []},
            ],
        }
        picture = {
            "name": "Microsoft.Xna.Framework.Media.Picture", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Date", "type": "System.DateTime",
                 "static": False, "get": True, "set": False, "parameters": []},
            ],
        }
        collection = {
            "name": "Microsoft.Xna.Framework.Media.AlbumCollection", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Item",
                 "type": "Microsoft.Xna.Framework.Media.Album", "static": False,
                 "get": True, "set": False,
                 "parameters": [{"name": "index", "type": "System.Int32"}]},
            ],
        }
        album = {"name": "Microsoft.Xna.Framework.Media.Album", "kind": "class"}
        reference = {value["name"]: value for value in
                     (visualization, picture, collection, album)}
        visualization_members = VERIFY.mapped_members(visualization, RULES, reference)
        self.assertEqual(visualization_members["Frequencies"]["returnType"], "&[f32]")
        self.assertEqual(visualization_members["Samples"]["returnType"], "&[f32]")
        picture_members = VERIFY.mapped_members(picture, RULES, reference)
        self.assertEqual(picture_members["Date"]["returnType"], "Result<SystemTime>")
        collection_members = VERIFY.mapped_members(collection, RULES, reference)
        self.assertEqual(collection_members["Item"]["returnType"],
                         "Result<Arc<Media::Album>>")

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
        self.assertEqual(
            members["FromStreamWithGraphicsDeviceAndStream"]["returnType"],
            "Result<Self>",
        )

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

    def test_dynamic_static_value_property_maps_to_fallible_context_function(self):
        panel = {
            "name": "Microsoft.Xna.Framework.Input.Touch.TouchPanel", "kind": "class",
            "genericParameters": [], "members": [
                {
                    "kind": "property", "name": "IsGestureAvailable",
                    "type": "System.Boolean", "static": True,
                    "get": True, "set": False, "parameters": [],
                }
            ],
        }
        members = VERIFY.mapped_members(panel, RULES, {panel["name"]: panel})
        self.assertEqual(members["IsGestureAvailable"]["kind"], "function")
        self.assertEqual(members["IsGestureAvailable"]["parameters"], [
            {"name": "game", "type": "&GameContext"},
        ])
        self.assertEqual(members["IsGestureAvailable"]["returnType"], "Result<bool>")

    def test_audio_native_roots_use_explicit_game_context_and_results(self):
        effect = {
            "name": "Microsoft.Xna.Framework.Audio.SoundEffect", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "constructor", "name": ".ctor", "parameters": [
                    {"name": "buffer", "type": "System.Byte[]"},
                    {"name": "sampleRate", "type": "System.Int32"},
                    {"name": "channels", "type": "Microsoft.Xna.Framework.Audio.AudioChannels"},
                ]},
                {"kind": "property", "name": "MasterVolume", "static": True,
                 "type": "System.Single", "get": True, "set": True, "parameters": []},
                {"kind": "method", "name": "GetSampleDuration", "static": True,
                 "returnType": "System.TimeSpan", "genericParameters": [], "parameters": [
                    {"name": "sizeInBytes", "type": "System.Int32"},
                    {"name": "sampleRate", "type": "System.Int32"},
                    {"name": "channels", "type": "Microsoft.Xna.Framework.Audio.AudioChannels"},
                 ]},
            ],
        }
        channels = {
            "name": "Microsoft.Xna.Framework.Audio.AudioChannels", "kind": "enum",
        }
        members = VERIFY.mapped_members(
            effect, RULES, {effect["name"]: effect, channels["name"]: channels}
        )
        self.assertEqual(members["new"]["parameters"][0],
                         {"name": "game", "type": "&GameContext"})
        self.assertEqual(members["new"]["returnType"], "Result<Self>")
        self.assertEqual(members["MasterVolume"]["parameters"],
                         [{"name": "game", "type": "&GameContext"}])
        self.assertEqual(members["MasterVolume"]["returnType"], "Result<f32>")
        self.assertEqual(members["SetMasterVolume"]["returnType"], "Result<()>")
        self.assertEqual(members["GetSampleDuration"]["returnType"], "TimeSpan")

    def test_storage_begin_end_and_stream_projection_is_concrete(self):
        device = {
            "name": "Microsoft.Xna.Framework.Storage.StorageDevice", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "BeginOpenContainer", "static": False,
                 "returnType": "System.IAsyncResult", "genericParameters": [], "parameters": [
                     {"name": "displayName", "type": "System.String"},
                     {"name": "callback", "type": "System.AsyncCallback"},
                     {"name": "state", "type": "System.Object"},
                 ]},
                {"kind": "method", "name": "EndOpenContainer", "static": False,
                 "returnType": "Microsoft.Xna.Framework.Storage.StorageContainer",
                 "genericParameters": [], "parameters": [
                     {"name": "result", "type": "System.IAsyncResult"},
                 ]},
            ],
        }
        container = {
            "name": "Microsoft.Xna.Framework.Storage.StorageContainer", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "CreateFile", "static": False,
                 "returnType": "System.IO.Stream", "genericParameters": [], "parameters": [
                     {"name": "file", "type": "System.String"},
                 ]},
            ],
        }
        types = {device["name"]: device, container["name"]: container}
        device_members = VERIFY.mapped_members(device, RULES, types)
        self.assertEqual(device_members["BeginOpenContainer"]["returnType"],
                         "Result<StorageAsyncResult>")
        self.assertEqual(device_members["BeginOpenContainer"]["parameters"][-2:], [
            {"name": "callback", "type": "Option<StorageAsyncCallback>"},
            {"name": "state", "type": "StorageAsyncState"},
        ])
        self.assertEqual(device_members["EndOpenContainer"]["parameters"][-1],
                         {"name": "result", "type": "&StorageAsyncResult"})
        container_members = VERIFY.mapped_members(container, RULES, types)
        self.assertEqual(container_members["CreateFile"]["returnType"],
                         "Result<StorageStream>")

    def test_design_type_converter_uses_typed_rust_projection(self):
        math = {
            "name": "Microsoft.Xna.Framework.Design.MathTypeConverter", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "constructor", "name": ".ctor", "parameters": []},
                {"kind": "method", "name": "CanConvertFrom", "static": False,
                 "returnType": "System.Boolean", "genericParameters": [], "parameters": [
                     {"name": "context", "type": "System.ComponentModel.ITypeDescriptorContext"},
                     {"name": "sourceType", "type": "System.Type"},
                 ]},
                {"kind": "method", "name": "GetProperties", "static": False,
                 "returnType": "System.ComponentModel.PropertyDescriptorCollection",
                 "genericParameters": [], "parameters": [
                     {"name": "context", "type": "System.ComponentModel.ITypeDescriptorContext"},
                     {"name": "value", "type": "System.Object"},
                     {"name": "attributes", "type": "System.Attribute[]"},
                 ]},
                {"kind": "field", "name": "propertyDescriptions",
                 "type": "System.ComponentModel.PropertyDescriptorCollection", "static": False},
                {"kind": "field", "name": "supportStringConvert",
                 "type": "System.Boolean", "static": False},
            ],
        }
        vector = {
            "name": "Microsoft.Xna.Framework.Design.Vector3Converter", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "constructor", "name": ".ctor", "parameters": []},
                {"kind": "method", "name": "ConvertFrom", "static": False,
                 "returnType": "System.Object", "genericParameters": [], "parameters": []},
                {"kind": "method", "name": "ConvertTo", "static": False,
                 "returnType": "System.Object", "genericParameters": [], "parameters": []},
                {"kind": "method", "name": "CreateInstance", "static": False,
                 "returnType": "System.Object", "genericParameters": [], "parameters": []},
            ],
        }
        index = {math["name"]: math, vector["name"]: vector}
        math_members = VERIFY.mapped_members(math, RULES, index)
        self.assertEqual(math_members["CanConvertFrom"]["parameters"], [
            {"name": "self", "type": "&Self"},
            {"name": "sourceType", "type": "DesignType"},
        ])
        self.assertEqual(math_members["GetProperties"]["returnType"],
                         "&[DesignPropertyDescriptor]")
        self.assertNotIn("propertyDescriptions", math_members)
        self.assertNotIn("supportStringConvert", math_members)
        vector_members = VERIFY.mapped_members(vector, RULES, index)
        self.assertEqual(vector_members["ConvertFrom"]["parameters"], [
            {"name": "self", "type": "&Self"},
            {"name": "culture", "type": "&DesignCulture"},
            {"name": "value", "type": "Option<&DesignValue>"},
        ])
        self.assertEqual(vector_members["ConvertFrom"]["returnType"], "Result<Vector3>")
        self.assertEqual(vector_members["ConvertTo"]["returnType"], "Result<DesignConversion>")
        self.assertEqual(vector_members["CreateInstance"]["parameters"][-1],
                         {"name": "propertyValues", "type": "Option<&[DesignPropertyValue]>"})

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

    def test_device_retained_state_property_preserves_shared_identity(self):
        state = {
            "name": "Microsoft.Xna.Framework.Graphics.BlendState", "kind": "class",
        }
        device = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsDevice", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "BlendState", "static": False,
                 "type": state["name"], "get": True, "set": True, "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(
            device, RULES, {device["name"]: device, state["name"]: state}
        )
        self.assertEqual(
            members["BlendState"]["returnType"],
            "Result<Arc<Graphics::BlendState>>",
        )
        self.assertEqual(
            members["SetBlendState"]["parameters"][-1]["type"],
            "Arc<Graphics::BlendState>",
        )

    def test_generic_event_uses_payload_and_shared_receiver(self):
        event_args = {
            "name": "Microsoft.Xna.Framework.GameComponentCollectionEventArgs",
            "kind": "class",
        }
        collection = {
            "name": "Microsoft.Xna.Framework.GameComponentCollection", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "event", "name": "ComponentAdded", "static": False,
                 "type": "System.EventHandler`1[Microsoft.Xna.Framework.GameComponentCollectionEventArgs]"},
            ],
        }
        members = VERIFY.mapped_members(
            collection, RULES, {collection["name"]: collection, event_args["name"]: event_args}
        )
        self.assertEqual(members["AddComponentAddedHandler"]["parameters"], [
            {"name": "self", "type": "&Self"},
            {"name": "handler", "type": "Box<dyn EventHandler<GameComponentCollectionEventArgs>>"},
        ])
        self.assertEqual(members["RemoveComponentAddedHandler"]["parameters"], [
            {"name": "self", "type": "&Self"},
            {"name": "registration", "type": "u64"},
        ])

    def test_preparing_device_settings_retains_shared_reference_graph(self):
        adapter = {
            "name": "Microsoft.Xna.Framework.Graphics.GraphicsAdapter", "kind": "class",
        }
        parameters = {
            "name": "Microsoft.Xna.Framework.Graphics.PresentationParameters", "kind": "class",
        }
        information = {
            "name": "Microsoft.Xna.Framework.GraphicsDeviceInformation", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "Adapter", "static": False,
                 "type": adapter["name"], "get": True, "set": True, "parameters": []},
                {"kind": "property", "name": "PresentationParameters", "static": False,
                 "type": parameters["name"], "get": True, "set": True, "parameters": []},
            ],
        }
        event_args = {
            "name": "Microsoft.Xna.Framework.PreparingDeviceSettingsEventArgs", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "constructor", "name": ".ctor", "static": False,
                 "returnType": None, "genericParameters": [], "parameters": [
                     {"name": "graphicsDeviceInformation", "type": information["name"]},
                 ]},
                {"kind": "property", "name": "GraphicsDeviceInformation", "static": False,
                 "type": information["name"], "get": True, "set": False, "parameters": []},
            ],
        }
        manager = {
            "name": "Microsoft.Xna.Framework.GraphicsDeviceManager", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "RankDevices", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "foundDevices",
                      "type": "System.Collections.Generic.List`1[Microsoft.Xna.Framework.GraphicsDeviceInformation]"},
                 ]},
            ],
        }
        index = {value["name"]: value for value in [
            adapter, parameters, information, event_args, manager,
        ]}
        info_members = VERIFY.mapped_members(information, RULES, index)
        self.assertEqual(info_members["Adapter"]["returnType"], "Arc<Graphics::GraphicsAdapter>")
        self.assertEqual(info_members["SetAdapter"]["parameters"], [
            {"name": "self", "type": "&Self"},
            {"name": "value", "type": "Arc<Graphics::GraphicsAdapter>"},
        ])
        args_members = VERIFY.mapped_members(event_args, RULES, index)
        self.assertEqual(args_members["new"]["parameters"][-1]["type"],
                         "Arc<GraphicsDeviceInformation>")
        self.assertEqual(args_members["GraphicsDeviceInformation"]["returnType"],
                         "Arc<GraphicsDeviceInformation>")
        manager_members = VERIFY.mapped_members(manager, RULES, index)
        self.assertEqual(manager_members["RankDevices"]["parameters"][-1],
                         {"name": "foundDevices",
                          "type": "&mut Vec<GraphicsDeviceInformation>"})

    def test_interface_parameter_projects_as_trait_object(self):
        component = {
            "name": "Microsoft.Xna.Framework.IGameComponent", "kind": "interface",
        }
        collection = {
            "name": "Microsoft.Xna.Framework.GameComponentCollection", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "RemoveItem", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "item", "type": component["name"]},
                 ]},
            ],
        }
        members = VERIFY.mapped_members(
            collection, RULES, {collection["name"]: collection, component["name"]: component}
        )
        self.assertEqual(members["RemoveItem"]["parameters"][-1]["type"], "&dyn IGameComponent")

    def test_retained_component_parameter_override_is_owned_arc(self):
        component = {
            "name": "Microsoft.Xna.Framework.IGameComponent", "kind": "interface",
        }
        collection = {
            "name": "Microsoft.Xna.Framework.GameComponentCollection", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "method", "name": "InsertItem", "static": False,
                 "returnType": "System.Void", "genericParameters": [], "parameters": [
                     {"name": "index", "type": "System.Int32"},
                     {"name": "item", "type": component["name"]},
                 ]},
            ],
        }
        members = VERIFY.mapped_members(
            collection, RULES, {collection["name"]: collection, component["name"]: component}
        )
        self.assertEqual(members["InsertItem"]["parameters"][-1]["type"], "Arc<dyn IGameComponent>")

    def test_projected_property_setter_name_drives_fallibility(self):
        game = {
            "name": "Microsoft.Xna.Framework.Game", "kind": "class",
            "genericParameters": [], "members": [
                {"kind": "property", "name": "IsMouseVisible", "static": False,
                 "type": "System.Boolean", "get": True, "set": True, "parameters": []},
            ],
        }
        members = VERIFY.mapped_members(game, RULES, {game["name"]: game})
        self.assertEqual(members["IsMouseVisible"]["returnType"], "bool")
        self.assertEqual(members["SetIsMouseVisible"]["returnType"], "Result<()>")

    def test_drawable_base_uses_explicit_composition_trait(self):
        name = "cna::Microsoft::Xna::Framework::DrawableGameComponent"
        expected = {name: {
            "kind": "struct", "clrKind": "class",
            "clrName": "Microsoft.Xna.Framework.DrawableGameComponent",
            "members": {}, "flags": False, "underlyingType": None,
            "baseType": "Microsoft.Xna.Framework.GameComponent",
            "interfaces": [], "allInterfaces": [], "genericParameters": [],
            "operatorTraits": set(),
        }}
        actual = {name: self._empty_actual("struct", {}, traits=("GameComponentBase",))}
        self.assertFalse(any(item["code"] == "BASE_PROJECTION_MISMATCH"
                             for item in VERIFY.compare(expected, actual, RULES)))

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
