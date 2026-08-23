# Small-family implementation queue

This queue was regenerated from the pinned XNA 4.0 Windows reference contract before implementation. `MEMBER_COUNT` is the declared XNA member count (the Rust verifier may expand properties and events into more projected members).

| TYPE | FAMILY | EXPECTED_RUST_PATH | XNA_KIND | MEMBER_COUNT | MANAGED_ONLY | NATIVE_DEPENDENCY | CNA_ABI_ROUTE | OWNERSHIP_MODEL | BACKEND_DEPENDENCY |
|---|---|---|---:|---:|---|---|---|---|---|
| GraphicsDeviceInformation | Framework/core | `cna::Microsoft::Xna::Framework::GraphicsDeviceInformation` | class | 7 | yes | indirect adapter/presentation values only | none required | owned managed value; clone deep-copies presentation parameters | none |
| GraphicsDeviceManager | Framework/core | `cna::Microsoft::Xna::Framework::GraphicsDeviceManager` | class | 30 | no | active `Game` and its existing graphics device | `cna_graphics_device_manager_*` selected routes | managed identity retains its `GameState`; callback-scoped native manager handle is released before Game destruction | device reconfiguration may be refused by the renderer/platform |
| IGraphicsDeviceManager | Framework/core | `cna::Microsoft::Xna::Framework::IGraphicsDeviceManager` | interface | 3 | yes | implementation-dependent | implemented by `GraphicsDeviceManager` through its CNA routes | borrowed trait contract | implementation-dependent |
| PreparingDeviceSettingsEventArgs | Framework/core | `cna::Microsoft::Xna::Framework::PreparingDeviceSettingsEventArgs` | class | 2 | yes | none | none | owned event value containing the candidate information | none |
| GestureSample | Input | `cna::Microsoft::Xna::Framework::Input::Touch::GestureSample` | struct | 7 | yes | none | converted from `CNA_GestureSample` when read | copy value | none |
| GestureType | Input | `cna::Microsoft::Xna::Framework::Input::Touch::GestureType` | flags enum | 12 | yes | none | `CNA_GESTURE_TYPE_*` identities | copy flags value | none |
| TouchPanel | Input | `cna::Microsoft::Xna::Framework::Input::Touch::TouchPanel` | static class | 9 | no | active callback-scoped Game handle | `cna_touch_panel_*`, `cna_touch_get_state`, `cna_touch_get_capabilities` | no owned native handle; snapshot values are copied | touch hardware and gesture queue may legitimately be absent |
| StorageContainer | Storage | `cna::Microsoft::Xna::Framework::Storage::StorageContainer` | class | 19 | no | storage device, filesystem abstraction, streams | `cna_storage_container_*`, `cna_storage_stream_*` | owned child retains parent device; streams retain container and close first | host storage filesystem |
| StorageDevice | Storage | `cna::Microsoft::Xna::Framework::Storage::StorageDevice` | sealed class | 12 | no | storage selector/root | `cna_storage_device_*` | owned native root retained by containers | storage location availability; no UI is fabricated |
| StorageDeviceNotConnectedException | Storage | `cna::Microsoft::Xna::Framework::Storage::StorageDeviceNotConnectedException` | class | 4 | yes | none | none | owned managed error value | none |
| GamerServicesComponent | GamerServices | `cna::Microsoft::Xna::Framework::GamerServices::GamerServicesComponent` | class | 3 | yes | no selected GamerServices profile exists | no Gamer/Guide/network routes are projected | composed `GameComponent` retaining the normal weak Game association | dispatcher services remain explicitly unavailable |

The selected profile ends at `GamerServicesComponent`; it does not pull the separate GamerServices/Avatar graph into this milestone.
