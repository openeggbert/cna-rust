# Framework/core evidence

Status date: 2026-08-23

## Exact selected family

The regenerated XNA 4.0 Windows scoreboard identified exactly four missing
Framework/core types: `GraphicsDeviceInformation`, `GraphicsDeviceManager`,
`IGraphicsDeviceManager`, and `PreparingDeviceSettingsEventArgs`. All four now
have zero local strict diagnostics.

`GraphicsDeviceInformation` is a managed mutable reference object. Ordinary
Rust `Clone` retains CLR-style shared identity; its explicit XNA `Clone`
deep-copies `PresentationParameters` while retaining the adapter identity.
`PreparingDeviceSettingsEventArgs` retains the mutable proposal through an
`Arc`, so changes made by a handler are copied back to the native candidate.

## Game and device integration

`GraphicsDeviceManager` registers exactly once with its `GameState`, publishes
the manager and device-service interfaces through the per-game service
container, and attaches its native manager only while the Game is active. It
never constructs a second graphics device: `GraphicsDevice` is the durable
wrapper for CNA's Game-owned device.

Preferences may be set before `Run`; attach copies them to CNA. While active,
setters, `ApplyChanges`, `ToggleFullScreen`, `CreateDevice`, `BeginDraw`, and
`EndDraw` use the reviewed `cna_graphics_device_manager_*` routes. Cleanup
unsubscribes six event registrations and disposes/destroys the manager before
the Game is destroyed. A retained manager then fails safely.

Native event callbacks expose the public `GraphicsDeviceManager` as sender,
snapshot handlers so self-removal is safe, contain Rust panic, and report a
pending `CnaError::Callback` from the next safe operation boundary. The native
qualification observed `PreparingDeviceSettings` and exactly one `Disposed`;
an intentional preparing-handler panic was contained and a new Game then ran
successfully.

## Qualified and blocked behavior

The exact ABI-0.7 HEADLESS artifact accepted preference synchronization and
`ApplyChanges`, and supplied a mutable preparing candidate. HEADLESS did not
originate device reset/resetting transitions in this run. The protected
`OnDeviceResetting` and `OnDeviceReset` paths were therefore qualified as
managed event dispatch only, not reported as OS/backend transitions.

`FindBestDevice` builds the XNA proposal from current preferences and
`CanResetDevice` validates the current Game device/profile. CNA ABI 0.7 does
not expose XNA's device-candidate enumeration/ranking policy, so `RankDevices`
returns a precise `CnaError::UnsupportedRuntime`; no ranking or reset is
fabricated. `ApplyChanges` before an active native Game likewise reports the
explicit runtime requirement.
