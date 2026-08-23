# Storage evidence

Status date: 2026-08-23

## Exact selected family and async mapping

The regenerated queue contained `StorageDevice`, `StorageContainer`, and
`StorageDeviceNotConnectedException`; all three have zero local strict
diagnostics.

XNA's Begin/End surface maps to the documented concrete
`StorageAsyncResult`. CNA completes selectors and container opens before Begin
returns, so the result reports `CompletedSynchronously` and `IsCompleted`,
retains the caller state, invokes the optional callback before returning, and
is consumed by End exactly once. End rejects the wrong operation kind and a
container result belonging to another device. Callback panic is contained and
returned as `CnaError::Callback`; no CLR thread pool, `IAsyncResult`, future,
or fake pending task exists.

All four selector forms route to CNA, including player and size/directory
selections. XNA Windows rejects negative size but accepts a negative directory
count; the bridge preserves that observable validation and supplies CNA's
documented minimum directory requirement. HEADLESS uses CNA's deterministic
selector and fabricates no UI.

## Filesystem and containment

Container creation, deletion, directory/file existence and mutation,
wildcard enumeration, all OpenFile forms, stream read/write/seek/length/
flush/capability operations, and disposal use only canonical
`cna_storage_*` routes. The Rust layer never bypasses CNA with `std::fs`.

The qualified ABI-0.7 C adapter's `RelativePath` currently validates UTF-8 but
does not itself reject parent traversal for every container child route. Rust
therefore enforces XNA containment before native dispatch. It rejects absolute
and drive/UNC paths, root escapes, empty child names, and multi-component
search patterns. Nested relative names, repeated separators, dot components,
contained `a/../child`, mixed separators, and one-component wildcards remain
valid. Native qualification covered these rules plus a nested file round trip.
This is an upstream CNA semantic gap, not a host-filesystem permission claim.

## Ownership and events

An owned `StorageDevice` is the native root. A container retains that device;
each stream retains its container. Container Dispose closes live streams,
requires CNA's synchronous per-instance `Disposing` notification, marks the
public object disposed, emits the Rust event exactly once, unsubscribes, and
destroys the handle. Explicit and repeated Dispose, device-before-container,
container-before-stream, wrong-thread stream/container refusal followed by an
owner-thread retry, and callback-panic cleanup are covered.

`StorageDevice.DeviceChanged` is registered through CNA's process-wide static
subscription; removing the final Rust handler unsubscribes its native
registration. Registration, removal, and stale removal were verified. A
contained handler/unsubscription failure is returned from the next fallible
Storage boundary. A notification arriving off the subscription owner thread is
queued and drained at the next owner-thread Storage boundary; user code is not
called from the native worker. The platform did not provide an OS-originated
storage transition, so delivery of such a transition remains platform pending.
`StorageContainer.Disposing` was
observed synchronously and exactly once. User panics cannot cross either C
callback boundary.

Native tests pin `XDG_DATA_HOME` to a temporary qualification root. They cover
selector callback/state/one-shot rules, foreign End rejection, space and
connection queries, container and stream work, containment, live-stream
shutdown, event disposal, deletion, and repeated cycles.
