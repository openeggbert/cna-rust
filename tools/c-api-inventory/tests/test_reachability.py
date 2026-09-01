"""Mutation tests for the bound-route reachability walk.

A reachability check is only worth the differences it can see, and this one
replaced a check that could see almost none: the first version compared a C
route's name against Rust identifiers, and the field that holds a route's
pointer is not named after the route.  It reported 303 dead routes, of which
165 had a safe API in front of them.

Each test here plants one realistic defect in a synthetic crate shaped like
this one -- a table of function pointers in ``native/``, wrappers over it, and
a safe layer outside -- and asserts the walk notices.  The last group asserts
the same three chain lengths against the real crate, so a refactor that moves a
call one hop further away cannot quietly turn a live route into a dead one.
"""

import importlib.util
from pathlib import Path
import tempfile
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "cna_reachability", ROOT / "tools/c-api-inventory/reachability.py"
)
REACHABILITY = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(REACHABILITY)


TABLE = """\
use cna_sys as sys;

pub(crate) struct ExampleApi {
    pub(crate) widget_open: sys::cna_widget_open_fn,
    pub(crate) widget_close: sys::cna_widget_close_fn,
    pub(crate) widget_poke: sys::cna_widget_poke_fn,
    pub(crate) widget_orphan: sys::cna_widget_orphan_fn,
}

impl ExampleApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            widget_open: symbol!(cna_widget_open, sys::cna_widget_open_fn),
            widget_close: symbol!(cna_widget_close, sys::cna_widget_close_fn),
            widget_poke: symbol!(cna_widget_poke, sys::cna_widget_poke_fn),
            widget_orphan: symbol!(cna_widget_orphan, sys::cna_widget_orphan_fn),
        })
    }
}

impl Native {
    pub(crate) fn open_widget(&self) -> Result<()> {
        self.check(unsafe { (self.example.widget_open)() })
    }

    pub(crate) fn shut_the_widget(&self) -> Result<()> {
        self.finish_widget()
    }

    fn finish_widget(&self) -> Result<()> {
        self.really_finish_widget()
    }

    fn really_finish_widget(&self) -> Result<()> {
        self.check(unsafe { (self.example.widget_close)() })
    }

    fn nobody_calls_this(&self) -> Result<()> {
        self.check(unsafe { (self.example.widget_poke)() })
    }
}
"""

SAFE = """\
//! A safe layer that mentions widget_orphan only in prose.

impl Widget {
    pub fn Open(&self) -> Result<()> {
        self.native.open_widget()
    }

    pub fn Close(&self) -> Result<()> {
        self.native.shut_the_widget()
    }

    pub fn Describe(&self) -> &'static str {
        "widget_poke is named here only as text"
    }
}
"""

# The shape `RUST-SURFACE-001` made load-bearing. Every CNA-only member is now
# reached through `impl Trait for Type` rather than through an inherent impl,
# and a walk that only understood inherent impls would have turned 109 live
# routes dead without anything failing.
SAFE_AS_TRAIT_IMPL = """\
//! The same safe layer, with every caller behind an extension trait.

pub trait WidgetExt {
    fn Open(&self) -> Result<()>;
    fn Close(&self) -> Result<()>;
}

impl WidgetExt for Widget {
    fn Open(&self) -> Result<()> {
        self.native.open_widget()
    }

    fn Close(&self) -> Result<()> {
        self.native.shut_the_widget()
    }
}
"""


def crate(table: str = TABLE, safe: str = SAFE) -> Path:
    directory = Path(tempfile.mkdtemp())
    (directory / "native").mkdir()
    (directory / "native" / "example.rs").write_text(table, encoding="utf-8")
    (directory / "widget.rs").write_text(safe, encoding="utf-8")
    return directory


def routes(table: str = TABLE, safe: str = SAFE) -> tuple[set, set]:
    result = REACHABILITY.analyse(crate(table, safe))
    return result["reachable"], result["unreachable"]


class WalkTests(unittest.TestCase):
    def test_a_route_the_safe_layer_reaches_in_one_hop_is_reachable(self):
        reachable, _ = routes()
        self.assertIn("cna_widget_open", reachable)

    def test_a_route_three_wrappers_deep_is_reachable(self):
        # safe -> shut_the_widget -> finish_widget -> really_finish_widget.
        # The two-hop check this replaced could not see past the first wrapper.
        reachable, _ = routes()
        self.assertIn("cna_widget_close", reachable)

    def test_a_route_no_safe_caller_reaches_is_unreachable(self):
        _, unreachable = routes()
        self.assertIn("cna_widget_poke", unreachable)

    def test_a_route_nothing_names_at_all_is_unreachable(self):
        _, unreachable = routes()
        self.assertIn("cna_widget_orphan", unreachable)

    def test_removing_the_safe_caller_makes_a_live_route_dead(self):
        without = SAFE.replace("self.native.open_widget()", "Ok(())")
        _, unreachable = routes(safe=without)
        self.assertIn("cna_widget_open", unreachable)

    def test_breaking_one_link_of_the_chain_makes_the_route_dead(self):
        broken = TABLE.replace("self.really_finish_widget()", "Ok(())")
        _, unreachable = routes(table=broken)
        self.assertIn("cna_widget_close", unreachable)

    def test_a_caller_inside_a_trait_impl_is_a_safe_call_site(self):
        # `RUST-SURFACE-001` moved every CNA-only member into `impl Trait for
        # Type`. The walk collects identifiers from files outside `native/`
        # without caring what kind of block they sit in, and this is what says
        # so rather than leaving it to be assumed.
        reachable, _ = routes(safe=SAFE_AS_TRAIT_IMPL)
        self.assertIn("cna_widget_open", reachable)
        self.assertIn("cna_widget_close", reachable)

    def test_deleting_the_only_trait_impl_caller_makes_the_route_dead(self):
        without = SAFE_AS_TRAIT_IMPL.replace("self.native.open_widget()", "Ok(())")
        _, unreachable = routes(safe=without)
        self.assertIn("cna_widget_open", unreachable)

    def test_the_acquisition_is_not_a_use(self):
        # `load` names every field.  Counting that as a call site would make
        # every route in the crate look reachable, which is the failure this
        # check exists to avoid.
        _, unreachable = routes()
        self.assertEqual(
            {"cna_widget_poke", "cna_widget_orphan"},
            unreachable,
            "the loader's field initialisers must not count as call sites",
        )

    def test_a_field_named_only_in_a_string_or_comment_is_not_a_use(self):
        # `Describe` returns a string naming widget_poke, and the module
        # comment names widget_orphan.
        _, unreachable = routes()
        self.assertIn("cna_widget_poke", unreachable)
        self.assertIn("cna_widget_orphan", unreachable)

    def test_a_new_safe_caller_revives_a_dead_route(self):
        revived = SAFE.replace(
            "self.native.open_widget()", "self.native.nobody_calls_this()"
        )
        reachable, _ = routes(safe=revived)
        self.assertIn("cna_widget_poke", reachable)


class RealCrateTests(unittest.TestCase):
    """The chains that actually exist here, at each length the crate uses."""

    @classmethod
    def setUpClass(cls):
        cls.result = REACHABILITY.analyse(ROOT / "crates/cna/src")

    def test_every_acquisition_is_accounted_for(self):
        total = len(self.result["reachable"]) + len(self.result["unreachable"])
        self.assertEqual(total, len(self.result["fieldToRoute"]))

    def test_a_field_the_safe_layer_names_directly_is_reachable(self):
        # crates/cna/src/extensions/devices.rs names `camera_get_state_ext`.
        self.assertIn("cna_camera_get_state_ext", self.result["reachable"])

    def test_a_field_behind_one_wrapper_is_reachable(self):
        # AudioCategory::Pause -> Native::audio_category_action ->
        # self.audio.category_pause. The field is not named after the route,
        # which is exactly what the name-matching check could not follow.
        self.assertIn("cna_audio_category_pause", self.result["reachable"])

    def test_a_field_behind_two_wrappers_is_reachable(self):
        # Every fallible call -> Native::check -> Native::last_error_category
        # -> self.error_get_last_info.
        self.assertIn("cna_error_get_last_info", self.result["reachable"])

    def test_field_names_are_unique_so_a_use_names_one_route(self):
        fields = self.result["fieldToRoute"]
        self.assertEqual(len(fields), len(set(fields)))


if __name__ == "__main__":
    unittest.main()
