//! CNA's modern text input: committed characters, IME composition, candidates.
//!
//! XNA had nothing like this. `Keyboard.GetState` reports which physical keys
//! are down, which cannot spell a character an IME composed, cannot report a
//! draft the user has not committed, and cannot see a candidate list at all.
//! Every one of those is a CNA concept, so it lives here and not beside
//! `Microsoft::Xna::Framework::Input::Keyboard`.
//!
//! Three things about this boundary are load-bearing:
//!
//! - **Every string CNA passes is borrowed for the duration of the callback.**
//!   Upstream says so for both the composition draft and each candidate, and
//!   the views are not NUL-terminated. Everything reaching a Rust handler is
//!   therefore copied out before the call returns; no `CNA_StringView` escapes.
//! - **Committed text arrives as UTF-16 code units, not characters.** A code
//!   point above U+FFFF arrives as two calls, a high surrogate then a low one.
//!   [`TextInput::subscribe_text`] reports exactly that, and
//!   [`Utf16Assembler`] is the piece that turns it back into `char` without
//!   pretending a lone surrogate is one.
//! - **A panic must not unwind into C.** Every handler runs inside
//!   `catch_unwind`, and a panicking handler is dropped from delivery rather
//!   than taking the process down mid-callback.

#![allow(clippy::missing_errors_doc)]

use core::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::window::WindowHandle;
use crate::game::GameContext;
use crate::native::Native;
use crate::value::Rectangle;

/// What kind of text the platform should expect, so it can choose a keyboard.
///
/// A hint, not a constraint: it selects the on-screen keyboard layout and
/// whether the platform hides what is typed. It does not validate anything.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TextInputType {
    #[default]
    Text,
    Name,
    Email,
    Username,
    /// A password the platform should hide as it is entered.
    PasswordHidden,
    /// A password the platform may show, as a "reveal" control does.
    PasswordVisible,
    Number,
    PinHidden,
    PinVisible,
}

impl TextInputType {
    const fn to_native(self) -> sys::CNA_TextInputType {
        match self {
            Self::Text => sys::CNA_TEXT_INPUT_TYPE_TEXT,
            Self::Name => sys::CNA_TEXT_INPUT_TYPE_TEXT_NAME,
            Self::Email => sys::CNA_TEXT_INPUT_TYPE_TEXT_EMAIL,
            Self::Username => sys::CNA_TEXT_INPUT_TYPE_TEXT_USERNAME,
            Self::PasswordHidden => sys::CNA_TEXT_INPUT_TYPE_TEXT_PASSWORD_HIDDEN,
            Self::PasswordVisible => sys::CNA_TEXT_INPUT_TYPE_TEXT_PASSWORD_VISIBLE,
            Self::Number => sys::CNA_TEXT_INPUT_TYPE_NUMBER,
            Self::PinHidden => sys::CNA_TEXT_INPUT_TYPE_NUMBER_PASSWORD_HIDDEN,
            Self::PinVisible => sys::CNA_TEXT_INPUT_TYPE_NUMBER_PASSWORD_VISIBLE,
        }
    }
}

/// One IME composition update, owned rather than borrowed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEditing {
    /// The draft composition, copied out of CNA's borrowed view.
    pub text: String,
    /// Byte offset of the active editing region inside `text`.
    pub start: i32,
    /// Byte length of the active editing region inside `text`.
    pub length: i32,
}

/// One IME candidate list, owned rather than borrowed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEditingCandidates {
    /// The candidates, each copied out of CNA's borrowed view.
    pub candidates: Vec<String>,
    /// The pre-selected candidate, or `None` when the platform selected none.
    ///
    /// The container encodes "none" as `-1`; keeping that as a signed index
    /// would let a caller use it as one.
    pub selected: Option<usize>,
    /// Whether the platform lays the list out horizontally.
    pub horizontal: bool,
}

/// Reassembles UTF-16 code units into characters.
///
/// The event delivers code units, so an emoji or any other astral character
/// arrives as a surrogate pair across two calls. This joins them, and reports
/// an unpaired surrogate as exactly that rather than substituting U+FFFD --
/// a replacement character is indistinguishable from one the user really
/// typed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Utf16Assembler {
    pending_high: Option<u16>,
}

/// What one code unit completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Utf16Unit {
    /// A complete character.
    Character(char),
    /// A high surrogate; the next unit is expected to complete it.
    PendingHighSurrogate,
    /// A surrogate that cannot form a pair, reported rather than replaced.
    UnpairedSurrogate(u16),
}

/// The complete outcome of feeding one code unit.
///
/// One unit can settle two things at once: a high surrogate that was waiting
/// can turn out to be unpaired *and* the unit that revealed it can itself be a
/// character. Returning both is what makes this total -- an earlier draft
/// returned one value and silently dropped the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utf16Push {
    /// A high surrogate this unit proved unpaired, if any.
    pub orphaned: Option<u16>,
    /// What the unit just fed completed.
    pub unit: Utf16Unit,
}

const fn is_high_surrogate(unit: u16) -> bool {
    unit >= 0xD800 && unit < 0xDC00
}

const fn is_low_surrogate(unit: u16) -> bool {
    unit >= 0xDC00 && unit < 0xE000
}

impl Utf16Assembler {
    #[must_use]
    pub const fn new() -> Self {
        Self { pending_high: None }
    }

    /// Feeds one code unit and reports everything it settled.
    pub fn push(&mut self, unit: u16) -> Utf16Push {
        let pending = self.pending_high.take();
        if let Some(high) = pending {
            if is_low_surrogate(unit) {
                let value =
                    0x1_0000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(unit) - 0xDC00);
                let unit = char::from_u32(value)
                    .map_or(Utf16Unit::UnpairedSurrogate(unit), Utf16Unit::Character);
                return Utf16Push {
                    orphaned: None,
                    unit,
                };
            }
            // The waiting high surrogate never got its partner. Report it, and
            // still report what this unit is: dropping either would lose text
            // the user actually entered.
            return Utf16Push {
                orphaned: Some(high),
                unit: self.classify(unit),
            };
        }
        Utf16Push {
            orphaned: None,
            unit: self.classify(unit),
        }
    }

    /// Classifies one unit with nothing pending.
    fn classify(&mut self, unit: u16) -> Utf16Unit {
        if is_high_surrogate(unit) {
            self.pending_high = Some(unit);
            return Utf16Unit::PendingHighSurrogate;
        }
        if is_low_surrogate(unit) {
            return Utf16Unit::UnpairedSurrogate(unit);
        }
        char::from_u32(u32::from(unit))
            .map_or(Utf16Unit::UnpairedSurrogate(unit), Utf16Unit::Character)
    }

    /// Reports a high surrogate still waiting for a partner that never came.
    ///
    /// Call it when the stream ends; a trailing high surrogate is otherwise
    /// indistinguishable from one whose partner has not arrived yet.
    pub fn finish(&mut self) -> Option<u16> {
        self.pending_high.take()
    }
}

/// A live subscription. Dropping it stops delivery.
///
/// CNA's registrations are process-wide, so this is what bounds one; a handler
/// that outlived its data would be a use-after-free the moment the next event
/// arrived.
#[derive(Debug)]
pub struct TextInputSubscription {
    slot: u64,
    kind: HandlerKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum HandlerKind {
    Text,
    Editing,
    Candidates,
}

impl Drop for TextInputSubscription {
    fn drop(&mut self) {
        handlers().remove(self.kind, self.slot);
    }
}

type TextHandler = Box<dyn FnMut(u16) + Send>;
type EditingHandler = Box<dyn FnMut(&TextEditing) + Send>;
type CandidatesHandler = Box<dyn FnMut(&TextEditingCandidates) + Send>;

/// One native registration per event kind, plus the Rust handlers it feeds.
///
/// The registration is shared rather than per-subscriber, and that is not an
/// optimisation. CNA delivers each event once per *registration*, and this
/// crate's trampoline delivers to every Rust handler, so a registration per
/// subscriber would deliver `registrations x handlers` times: two subscribers
/// would each see every character twice. Registering once and multiplexing
/// here is what makes one event one delivery.
struct KindState<T> {
    registration: sys::CNA_TextInputRegistrationHandle,
    native: Option<Arc<Native>>,
    entries: Vec<(u64, T)>,
}

impl<T> Default for KindState<T> {
    fn default() -> Self {
        Self {
            registration: sys::CNA_INVALID_HANDLE,
            native: None,
            entries: Vec::new(),
        }
    }
}

#[derive(Default)]
struct HandlerTable {
    next: Mutex<u64>,
    text: Mutex<KindState<TextHandler>>,
    editing: Mutex<KindState<EditingHandler>>,
    candidates: Mutex<KindState<CandidatesHandler>>,
}

fn handlers() -> &'static HandlerTable {
    static TABLE: OnceLock<HandlerTable> = OnceLock::new();
    TABLE.get_or_init(HandlerTable::default)
}

impl HandlerTable {
    fn claim(&self) -> u64 {
        let mut next = self
            .next
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *next = next.wrapping_add(1).max(1);
        *next
    }

    fn remove(&self, kind: HandlerKind, slot: u64) {
        match kind {
            HandlerKind::Text => release(&self.text, slot),
            HandlerKind::Editing => release(&self.editing, slot),
            HandlerKind::Candidates => release(&self.candidates, slot),
        }
    }
}

/// Drops one handler, and the shared registration with the last of them.
fn release<T>(state: &Mutex<KindState<T>>, slot: u64) {
    let mut state = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.entries.retain(|(id, _)| *id != slot);
    if !state.entries.is_empty() {
        return;
    }
    if let (Some(native), registration) = (state.native.take(), state.registration) {
        if registration != sys::CNA_INVALID_HANDLE {
            // SAFETY: the registration was created here and is released once.
            let _ = unsafe { (native.runtime.text_input_unsubscribe)(registration) };
        }
    }
    state.registration = sys::CNA_INVALID_HANDLE;
}

/// Adds one handler, registering with CNA only for the first of its kind.
fn attach<T>(
    state: &Mutex<KindState<T>>,
    slot: u64,
    handler: T,
    native: &Arc<Native>,
    subscribe: impl FnOnce(&mut sys::CNA_TextInputRegistrationHandle) -> sys::CNA_Result,
) -> Result<()> {
    let mut state = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.entries.is_empty() {
        let mut registration = sys::CNA_INVALID_HANDLE;
        native.check(subscribe(&mut registration))?;
        state.registration = registration;
        state.native = Some(Arc::clone(native));
    }
    state.entries.push((slot, handler));
    Ok(())
}

/// Reads a borrowed view as owned text.
///
/// # Safety
/// The view must describe live bytes for the duration of this call.
unsafe fn owned_text(view: sys::CNA_StringView) -> String {
    if view.data.is_null() || view.byte_length == 0 {
        return String::new();
    }
    let Ok(length) = usize::try_from(view.byte_length) else {
        return String::new();
    };
    // SAFETY: the caller guarantees the view describes `length` live bytes.
    let bytes = unsafe { core::slice::from_raw_parts(view.data.cast::<u8>(), length) };
    String::from_utf8_lossy(bytes).into_owned()
}

unsafe extern "C" fn text_trampoline(code_unit: u16, _context: *mut c_void) {
    // A panic here would unwind into C. Containing it costs one handler's
    // delivery; letting it out costs the process.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut table = handlers()
            .text
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, handler) in table.entries.iter_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| handler(code_unit)));
        }
    }));
}

unsafe extern "C" fn editing_trampoline(
    info: *const sys::CNA_TextEditingEventInfo,
    _context: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if info.is_null() {
            return;
        }
        // SAFETY: CNA passes a live descriptor for the duration of this call.
        let info = unsafe { &*info };
        // The draft is copied here, while the view is still valid.
        let value = TextEditing {
            // SAFETY: the view is live for this call, as upstream documents.
            text: unsafe { owned_text(info.text) },
            start: info.start,
            length: info.length,
        };
        let mut table = handlers()
            .editing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, handler) in table.entries.iter_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| handler(&value)));
        }
    }));
}

unsafe extern "C" fn candidates_trampoline(
    info: *const sys::CNA_TextEditingCandidatesEventInfo,
    _context: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if info.is_null() {
            return;
        }
        // SAFETY: CNA passes a live descriptor for the duration of this call.
        let info = unsafe { &*info };
        let count = usize::try_from(info.candidate_count).unwrap_or(0);
        let mut candidates = Vec::with_capacity(count);
        if !info.candidates.is_null() {
            for index in 0..count {
                // SAFETY: upstream documents `candidates` as an array of
                // `candidate_count` views, live for this call, and null only
                // when the count is zero.
                let view = unsafe { *info.candidates.add(index) };
                // SAFETY: each view is live for this call.
                candidates.push(unsafe { owned_text(view) });
            }
        }
        let value = TextEditingCandidates {
            candidates,
            selected: usize::try_from(info.selected).ok().filter(|_| info.selected >= 0),
            horizontal: info.horizontal != sys::CNA_FALSE,
        };
        let mut table = handlers()
            .candidates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, handler) in table.entries.iter_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| handler(&value)));
        }
    }));
}

/// CNA's text-input service.
#[derive(Debug)]
pub struct TextInput;

impl TextInput {
    /// Begins text entry, showing an on-screen keyboard where there is one.
    pub fn start(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-scoped and live.
        native.check(unsafe { (native.runtime.text_input_start)(handle) })
    }

    /// Begins text entry with a hint about what is being entered.
    pub fn start_with_type(game: &GameContext<'_>, kind: TextInputType) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: as above; the type is a checked identity.
        native.check(unsafe {
            (native.runtime.text_input_start_with_type)(handle, kind.to_native())
        })
    }

    /// Ends text entry.
    pub fn stop(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-scoped and live.
        native.check(unsafe { (native.runtime.text_input_stop)(handle) })
    }

    /// Whether text entry is currently active.
    pub fn is_active(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe { (native.runtime.text_input_is_active)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether an on-screen keyboard is currently shown.
    pub fn is_screen_keyboard_shown(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe {
            (native.runtime.text_input_is_screen_keyboard_shown)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether an on-screen keyboard is shown for a particular window.
    pub fn is_screen_keyboard_shown_for_window(
        game: &GameContext<'_>,
        window: WindowHandle,
    ) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local; the window handle is by value.
        native.check(unsafe {
            (native.runtime.text_input_is_screen_keyboard_shown_for_window)(
                handle, window.0, &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Tells the platform where the text being edited is on screen.
    ///
    /// A platform that shows a candidate window uses this to avoid covering
    /// what the user is typing.
    pub fn set_input_rectangle(game: &GameContext<'_>, rectangle: Rectangle) -> Result<()> {
        let (native, handle) = game.native_game();
        let native_rectangle = sys::CNA_Rectangle {
            x: rectangle.X,
            y: rectangle.Y,
            width: rectangle.Width,
            height: rectangle.Height,
        };
        // SAFETY: the rectangle is passed by value.
        native.check(unsafe {
            (native.runtime.text_input_set_input_rectangle)(handle, native_rectangle)
        })
    }

    /// The window text input is directed at.
    pub fn window(game: &GameContext<'_>) -> Result<WindowHandle> {
        let (native, handle) = game.native_game();
        let mut value = 0_u64;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe { (native.runtime.text_input_get_window_handle)(handle, &mut value) })?;
        Ok(WindowHandle(value))
    }

    /// Directs text input at a window.
    pub fn set_window(game: &GameContext<'_>, window: WindowHandle) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: both arguments are by value.
        native.check(unsafe {
            (native.runtime.text_input_set_window_handle)(handle, window.0)
        })
    }

    /// Delivers committed UTF-16 code units to `handler`.
    ///
    /// The handler receives code *units*, not characters: an astral character
    /// arrives as two calls. Use [`Utf16Assembler`] to rejoin them.
    pub fn subscribe_text(
        handler: impl FnMut(u16) + Send + 'static,
    ) -> Result<TextInputSubscription> {
        let native = Native::process()?;
        let slot = handlers().claim();
        attach(
            &handlers().text,
            slot,
            Box::new(handler),
            &native,
            |registration| {
                // SAFETY: the trampoline has the canonical signature and needs
                // no context: it dispatches through this crate's own table, so
                // there is no pointer whose lifetime could be got wrong.
                unsafe {
                    (native.runtime.text_input_subscribe_text_input)(
                        Some(text_trampoline),
                        core::ptr::null_mut(),
                        registration,
                    )
                }
            },
        )?;
        Ok(TextInputSubscription {
            slot,
            kind: HandlerKind::Text,
        })
    }

    /// Delivers IME composition updates to `handler`.
    pub fn subscribe_editing(
        handler: impl FnMut(&TextEditing) + Send + 'static,
    ) -> Result<TextInputSubscription> {
        let native = Native::process()?;
        let slot = handlers().claim();
        attach(
            &handlers().editing,
            slot,
            Box::new(handler),
            &native,
            |registration| {
                // SAFETY: as for `subscribe_text`.
                unsafe {
                    (native.runtime.text_input_subscribe_text_editing)(
                        Some(editing_trampoline),
                        core::ptr::null_mut(),
                        registration,
                    )
                }
            },
        )?;
        Ok(TextInputSubscription {
            slot,
            kind: HandlerKind::Editing,
        })
    }

    /// Delivers IME candidate lists to `handler`.
    pub fn subscribe_candidates(
        handler: impl FnMut(&TextEditingCandidates) + Send + 'static,
    ) -> Result<TextInputSubscription> {
        let native = Native::process()?;
        let slot = handlers().claim();
        attach(
            &handlers().candidates,
            slot,
            Box::new(handler),
            &native,
            |registration| {
                // SAFETY: as for `subscribe_text`.
                unsafe {
                    (native.runtime.text_input_subscribe_text_editing_candidates)(
                        Some(candidates_trampoline),
                        core::ptr::null_mut(),
                        registration,
                    )
                }
            },
        )?;
        Ok(TextInputSubscription {
            slot,
            kind: HandlerKind::Candidates,
        })
    }

    /// Raises a committed code unit, as the platform would.
    ///
    /// CNA provides this so a game can be tested without a keyboard. It is a
    /// real delivery through the same path, not a shortcut around it.
    pub fn raise_text(game: &GameContext<'_>, code_unit: u16) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: both arguments are by value.
        native.check(unsafe { (native.runtime.text_input_raise_text_input)(handle, code_unit) })
    }

    /// Raises a composition update, as an IME would.
    pub fn raise_editing(
        game: &GameContext<'_>,
        text: &str,
        start: i32,
        length: i32,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let view = sys::CNA_StringView {
            data: text.as_ptr().cast::<core::ffi::c_char>(),
            byte_length: text.len() as u64,
        };
        // SAFETY: `text` is borrowed for the duration of the call, which is
        // exactly the lifetime CNA documents for the view it passes on.
        native.check(unsafe {
            (native.runtime.text_input_raise_text_editing)(handle, view, start, length)
        })
    }

    /// Raises a candidate list, as an IME would.
    pub fn raise_candidates(
        game: &GameContext<'_>,
        candidates: &[&str],
        selected: Option<usize>,
        horizontal: bool,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let views = candidates
            .iter()
            .map(|value| sys::CNA_StringView {
                data: value.as_ptr().cast::<core::ffi::c_char>(),
                byte_length: value.len() as u64,
            })
            .collect::<Vec<_>>();
        let count = i32::try_from(views.len())
            .map_err(|_| CnaError::InvalidInput("too many IME candidates"))?;
        let selected = match selected {
            None => -1,
            Some(index) => i32::try_from(index)
                .map_err(|_| CnaError::InvalidInput("candidate index is out of range"))?,
        };
        // SAFETY: `views` and the strings it borrows outlive the call.
        native.check(unsafe {
            (native.runtime.text_input_raise_text_editing_candidates)(
                handle,
                if views.is_empty() {
                    core::ptr::null()
                } else {
                    views.as_ptr()
                },
                count,
                selected,
                u8::from(horizontal),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Utf16Assembler, Utf16Push, Utf16Unit};

    fn units(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    #[test]
    fn ascii_and_bmp_units_are_characters() {
        let mut assembler = Utf16Assembler::new();
        for (unit, expected) in units("aé中").into_iter().zip(['a', 'é', '中']) {
            assert_eq!(
                assembler.push(unit),
                Utf16Push {
                    orphaned: None,
                    unit: Utf16Unit::Character(expected),
                }
            );
        }
        assert_eq!(assembler.finish(), None);
    }

    #[test]
    fn an_astral_character_arrives_as_a_pair_and_is_rejoined() {
        // The event delivers code units, so this really does arrive as two
        // calls; a projection that assumed characters would report two broken
        // ones instead of one emoji.
        let pair = units("🎮");
        assert_eq!(pair.len(), 2, "an astral character is two code units");
        let mut assembler = Utf16Assembler::new();
        assert_eq!(
            assembler.push(pair[0]),
            Utf16Push {
                orphaned: None,
                unit: Utf16Unit::PendingHighSurrogate,
            }
        );
        assert_eq!(
            assembler.push(pair[1]),
            Utf16Push {
                orphaned: None,
                unit: Utf16Unit::Character('🎮'),
            }
        );
        assert_eq!(assembler.finish(), None);
    }

    #[test]
    fn a_whole_astral_string_round_trips() {
        let source = "a🎮é🌍中";
        let mut assembler = Utf16Assembler::new();
        let mut rebuilt = String::new();
        for unit in units(source) {
            let push = assembler.push(unit);
            assert_eq!(push.orphaned, None);
            if let Utf16Unit::Character(value) = push.unit {
                rebuilt.push(value);
            }
        }
        assert_eq!(assembler.finish(), None);
        assert_eq!(rebuilt, source);
    }

    #[test]
    fn an_orphaned_high_surrogate_is_reported_without_losing_the_next_unit() {
        // Both facts must survive: the high surrogate was never completed, and
        // the unit that revealed it is itself a real character.
        let mut assembler = Utf16Assembler::new();
        assert_eq!(assembler.push(0xD83C).unit, Utf16Unit::PendingHighSurrogate);
        assert_eq!(
            assembler.push(u16::from(b'x')),
            Utf16Push {
                orphaned: Some(0xD83C),
                unit: Utf16Unit::Character('x'),
            }
        );
        assert_eq!(assembler.finish(), None);
    }

    #[test]
    fn one_high_surrogate_orphaned_by_another_starts_a_new_pair() {
        let mut assembler = Utf16Assembler::new();
        assembler.push(0xD83C);
        assert_eq!(
            assembler.push(0xD83D),
            Utf16Push {
                orphaned: Some(0xD83C),
                unit: Utf16Unit::PendingHighSurrogate,
            }
        );
        // The second high surrogate is still waiting, and completes normally.
        assert_eq!(assembler.push(0xDE00).unit, Utf16Unit::Character('😀'));
    }

    #[test]
    fn a_lone_low_surrogate_is_reported_rather_than_replaced() {
        // Substituting U+FFFD would be indistinguishable from a replacement
        // character the user really typed.
        let mut assembler = Utf16Assembler::new();
        assert_eq!(
            assembler.push(0xDE00),
            Utf16Push {
                orphaned: None,
                unit: Utf16Unit::UnpairedSurrogate(0xDE00),
            }
        );
    }

    #[test]
    fn a_trailing_high_surrogate_is_reported_by_finish() {
        let mut assembler = Utf16Assembler::new();
        assembler.push(0xD83C);
        assert_eq!(assembler.finish(), Some(0xD83C));
        assert_eq!(assembler.finish(), None, "finish reports it once");
    }
}
