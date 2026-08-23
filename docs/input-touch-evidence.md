# Touch and gesture evidence

Status date: 2026-08-23

## Exact selected family

The regenerated queue contained `GestureSample`, `GestureType`, and
`TouchPanel`. All three now have zero local strict diagnostics. Existing
`TouchLocation`, `TouchCollection`, their enumerator/state, and
`TouchPanelCapabilities` remain the shared snapshot model.

`GestureType` carries the exact ten XNA flag identities. `GestureSample` is a
copy value preserving the gesture, signed 100-nanosecond `TimeSpan` timestamp,
two positions, and two deltas. Platform-neutral corpus observations cover flag
composition and all six sample fields.

## Native routing

Every `TouchPanel` operation requires an active `GameContext`; no process-global
raw Game handle is exposed. The reviewed routes are:

- `cna_touch_get_state` and `cna_touch_get_capabilities`;
- display width/height/orientation get/set;
- enabled-gesture get/set and gesture-availability query;
- `cna_touch_panel_read_gesture`; and
- window-handle query through the selected read-only XNA property.

Native structures have compiler-checked C/Rust layouts. Touch locations are
copied into an eight-entry immutable snapshot; gesture timestamps and vectors
are converted field-for-field. Unknown state or gesture flag identities are
rejected.

## Platform boundary

The qualified Linux HEADLESS run reported no connected touch hardware, an
empty disconnected touch snapshot, and no queued gesture. Enabled gestures
round-tripped as `Tap | DoubleTap`; `ReadGesture` returned the native no-gesture
error. These are legitimate platform results, not fabricated hardware or a
Rust gesture recognizer. Actual touch input and native gesture recognition
remain hardware/platform pending.
