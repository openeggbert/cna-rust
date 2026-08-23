#![allow(non_snake_case)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::events::EventArgs;
use cna::Microsoft::Xna::Framework::Content::{ContentManager, ResourceContentManager};
use cna::Microsoft::Xna::Framework::{Game, GameServiceContainer};
use cna::{
    CnaError, ContentDisposable, ContentLoadable, ContentManagerBase, ContentReaderExt,
    ContentResourceProvider, ContentTypeReaderRegistry, GameState, GameStateAccess, Result,
    ServiceProvider,
};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new() -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("cna-rust-xnb-{}-{serial}", std::process::id()));
        fs::create_dir_all(&path).expect("create XNB fixture root");
        Self(path)
    }

    fn write(&self, asset_name: &str, bytes: &[u8]) {
        let path = self.0.join(format!("{asset_name}.xnb"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create XNB fixture directory");
        }
        fs::write(path, bytes).expect("write XNB fixture");
    }

    fn manager(&self) -> ContentManager {
        let services: Arc<GameServiceContainer> = Arc::new(GameServiceContainer::new());
        let provider: Arc<dyn ServiceProvider> = services;
        ContentManager::from_service_provider_and_root_directory(
            provider,
            self.0.to_str().expect("UTF-8 fixture path"),
        )
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_7bit(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let mut next = u8::try_from(value & 0x7f).expect("seven-bit chunk");
        value >>= 7;
        if value != 0 {
            next |= 0x80;
        }
        bytes.push(next);
        if value == 0 {
            return;
        }
    }
}

fn write_string(bytes: &mut Vec<u8>, value: &str) {
    write_7bit(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn xnb(readers: &[(&str, i32)], shared_count: usize, body: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    write_7bit(&mut payload, readers.len());
    for (name, version) in readers {
        write_string(&mut payload, name);
        payload.extend_from_slice(&version.to_le_bytes());
    }
    write_7bit(&mut payload, shared_count);
    payload.extend_from_slice(body);

    let mut bytes = b"XNBw\x05\x00".to_vec();
    let size = u32::try_from(10 + payload.len()).expect("fixture size");
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

#[derive(Debug, Eq, PartialEq)]
struct CustomAsset {
    name: String,
    value: i32,
}

impl ContentLoadable for CustomAsset {}

#[test]
fn custom_reader_cache_identity_and_validation_are_real() {
    const READER: &str = "CnaRust.Tests.CustomAssetReader";
    const STRING_READER: &str = "Microsoft.Xna.Framework.Content.StringReader";

    let initialized = Arc::new(AtomicBool::new(false));
    let initialized_for_reader = Arc::clone(&initialized);
    let _registration = ContentTypeReaderRegistry::RegisterWithInitialize::<CustomAsset, _, _>(
        READER,
        7,
        false,
        move |manager| {
            initialized_for_reader.store(
                manager
                    .GetTypeReader(std::any::TypeId::of::<String>())
                    .is_some(),
                Ordering::Release,
            );
            Ok(())
        },
        |input, _| {
            Ok(Arc::new(CustomAsset {
                name: input.ReadString()?,
                value: input.ReadInt32()?,
            }))
        },
    )
    .expect("register custom reader");

    let root = FixtureRoot::new();
    let mut body = vec![1];
    write_string(&mut body, "reader table");
    body.extend_from_slice(&42_i32.to_le_bytes());
    root.write("custom", &xnb(&[(READER, 7), (STRING_READER, 0)], 0, &body));
    root.write("wrong-version", &xnb(&[(READER, 8)], 0, &[1]));

    let manager = root.manager();
    let first = manager
        .Load::<CustomAsset>("custom")
        .expect("load custom XNB");
    let second = manager
        .Load::<CustomAsset>("CUSTOM")
        .expect("case-insensitive cache hit");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        *first,
        CustomAsset {
            name: "reader table".to_owned(),
            value: 42
        }
    );
    assert!(initialized.load(Ordering::Acquire));

    let wrong_type = manager
        .Load::<String>("custom")
        .expect_err("wrong cached type");
    assert!(wrong_type.to_string().contains("different Rust type"));
    let missing = manager
        .Load::<CustomAsset>("missing")
        .expect_err("missing asset");
    assert!(missing.to_string().contains("could not open content asset"));
    let wrong_version = manager
        .Load::<CustomAsset>("wrong-version")
        .expect_err("reader version mismatch");
    assert!(wrong_version
        .to_string()
        .contains("reader version mismatch"));
}

#[derive(Debug)]
struct ExistingAsset {
    value: AtomicUsize,
}

impl ContentLoadable for ExistingAsset {}

#[derive(Debug)]
struct ExternalAsset(String);

impl ContentLoadable for ExternalAsset {}

#[test]
fn reader_table_preserves_existing_instance_and_external_reference() {
    const WRAPPER_READER: &str = "CnaRust.Tests.ExistingWrapperReader";
    const EXISTING_READER: &str = "CnaRust.Tests.ExistingAssetReader";
    const EXTERNAL_READER: &str = "CnaRust.Tests.ExternalAssetReader";
    const CHILD_READER: &str = "CnaRust.Tests.ExternalChildReader";

    let existing_seen = Arc::new(AtomicBool::new(false));
    let existing_seen_by_reader = Arc::clone(&existing_seen);
    let _existing = ContentTypeReaderRegistry::Register::<ExistingAsset, _>(
        EXISTING_READER,
        0,
        true,
        move |input, existing| {
            let value = existing.expect("existing object supplied by wrapper");
            value.value.store(
                usize::try_from(input.ReadInt32()?).expect("positive fixture"),
                Ordering::Release,
            );
            existing_seen_by_reader.store(true, Ordering::Release);
            Ok(value)
        },
    )
    .expect("register existing reader");
    let _wrapper = ContentTypeReaderRegistry::Register::<ExistingAsset, _>(
        WRAPPER_READER,
        0,
        false,
        |input, _| {
            let existing = Arc::new(ExistingAsset {
                value: AtomicUsize::new(0),
            });
            input.ReadObjectWithExistingInstance(Some(existing))
        },
    )
    .expect("register wrapper reader");

    let _child = ContentTypeReaderRegistry::Register::<ExternalAsset, _>(
        CHILD_READER,
        0,
        false,
        |input, _| Ok(Arc::new(ExternalAsset(input.ReadString()?))),
    )
    .expect("register child reader");
    let _external = ContentTypeReaderRegistry::Register::<ExternalAsset, _>(
        EXTERNAL_READER,
        0,
        false,
        |input, _| {
            input
                .ReadExternalReference()?
                .ok_or(CnaError::InvalidInput("null external reference"))
        },
    )
    .expect("register external reader");

    let root = FixtureRoot::new();
    let mut existing_body = vec![1, 2];
    existing_body.extend_from_slice(&73_i32.to_le_bytes());
    root.write(
        "existing",
        &xnb(
            &[(WRAPPER_READER, 0), (EXISTING_READER, 0)],
            0,
            &existing_body,
        ),
    );

    let mut external_body = vec![1];
    write_string(&mut external_body, "child");
    root.write(
        "folder/root",
        &xnb(&[(EXTERNAL_READER, 0)], 0, &external_body),
    );
    let mut child_body = vec![1];
    write_string(&mut child_body, "external object");
    root.write("folder/child", &xnb(&[(CHILD_READER, 0)], 0, &child_body));

    let manager = root.manager();
    let existing = manager
        .Load::<ExistingAsset>("existing")
        .expect("load into existing instance");
    assert_eq!(existing.value.load(Ordering::Acquire), 73);
    assert!(existing_seen.load(Ordering::Acquire));

    let external = manager
        .Load::<ExternalAsset>("folder/root")
        .expect("load external reference");
    assert_eq!(external.0, "external object");
    let child = manager
        .Load::<ExternalAsset>("folder/child")
        .expect("external reference populated normal cache");
    assert!(Arc::ptr_eq(&external, &child));
}

struct TrackedDisposable {
    dispose_count: Arc<AtomicUsize>,
    fail_first_dispose: bool,
}

impl ContentDisposable for TrackedDisposable {
    fn DisposeContent(&self) -> Result<()> {
        let previous = self.dispose_count.fetch_add(1, Ordering::AcqRel);
        if self.fail_first_dispose && previous == 0 {
            Err(CnaError::InvalidInput("injected content dispose failure"))
        } else {
            Ok(())
        }
    }
}

impl ContentLoadable for TrackedDisposable {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}

struct SharedOwner {
    values: Mutex<Vec<Arc<TrackedDisposable>>>,
}

impl ContentLoadable for SharedOwner {}

#[test]
fn shared_resources_are_identical_and_disposed_once() {
    const OWNER_READER: &str = "CnaRust.Tests.SharedOwnerReader";
    const TRACKED_READER: &str = "CnaRust.Tests.SharedTrackedReader";

    let dispose_count = Arc::new(AtomicUsize::new(0));
    let count_for_reader = Arc::clone(&dispose_count);
    let _tracked = ContentTypeReaderRegistry::Register::<TrackedDisposable, _>(
        TRACKED_READER,
        0,
        false,
        move |_, _| {
            Ok(Arc::new(TrackedDisposable {
                dispose_count: Arc::clone(&count_for_reader),
                fail_first_dispose: false,
            }))
        },
    )
    .expect("register tracked reader");
    let _owner = ContentTypeReaderRegistry::Register::<SharedOwner, _>(
        OWNER_READER,
        0,
        false,
        |input, _| {
            let owner = Arc::new(SharedOwner {
                values: Mutex::new(Vec::new()),
            });
            for _ in 0..2 {
                let owner_for_fixup = Arc::clone(&owner);
                input.ReadSharedResource(Box::new(move |value| {
                    owner_for_fixup
                        .values
                        .lock()
                        .expect("owner lock")
                        .push(value);
                }))?;
            }
            Ok(owner)
        },
    )
    .expect("register owner reader");

    let root = FixtureRoot::new();
    // root reader, two references to shared slot 1, then shared object reader
    root.write(
        "shared",
        &xnb(&[(OWNER_READER, 0), (TRACKED_READER, 0)], 1, &[1, 1, 1, 2]),
    );
    let manager = root.manager();
    let owner = manager
        .Load::<SharedOwner>("shared")
        .expect("load shared graph");
    let values = owner.values.lock().expect("owner lock");
    assert_eq!(values.len(), 2);
    assert!(Arc::ptr_eq(&values[0], &values[1]));
    drop(values);
    manager.Unload().expect("unload shared graph");
    assert_eq!(dispose_count.load(Ordering::Acquire), 1);
}

#[test]
fn partial_failures_clean_resources_and_failed_unload_remains_disposable() {
    const FAIL_READER: &str = "CnaRust.Tests.PartialFailureReader";
    const TRACKED_READER: &str = "CnaRust.Tests.PartialTrackedReader";
    const UNLOAD_READER: &str = "CnaRust.Tests.FailingUnloadReader";

    let partial_disposes = Arc::new(AtomicUsize::new(0));
    let partial_for_reader = Arc::clone(&partial_disposes);
    let _tracked = ContentTypeReaderRegistry::Register::<TrackedDisposable, _>(
        TRACKED_READER,
        0,
        false,
        move |_, _| {
            Ok(Arc::new(TrackedDisposable {
                dispose_count: Arc::clone(&partial_for_reader),
                fail_first_dispose: false,
            }))
        },
    )
    .expect("register partial tracked reader");
    let _failure =
        ContentTypeReaderRegistry::Register::<CustomAsset, _>(FAIL_READER, 0, false, |input, _| {
            let _partial = input.ReadObject::<TrackedDisposable>()?;
            Err(CnaError::InvalidInput("injected reader failure"))
        })
        .expect("register failing reader");

    let unload_disposes = Arc::new(AtomicUsize::new(0));
    let unload_for_reader = Arc::clone(&unload_disposes);
    let _unload = ContentTypeReaderRegistry::Register::<TrackedDisposable, _>(
        UNLOAD_READER,
        0,
        false,
        move |_, _| {
            Ok(Arc::new(TrackedDisposable {
                dispose_count: Arc::clone(&unload_for_reader),
                fail_first_dispose: true,
            }))
        },
    )
    .expect("register failing-unload reader");

    let root = FixtureRoot::new();
    root.write(
        "partial",
        &xnb(&[(FAIL_READER, 0), (TRACKED_READER, 0)], 0, &[1, 2]),
    );
    root.write("unload", &xnb(&[(UNLOAD_READER, 0)], 0, &[1]));
    let manager = root.manager();

    let error = manager
        .Load::<CustomAsset>("partial")
        .expect_err("reader failure");
    assert!(error.to_string().contains("injected reader failure"));
    assert_eq!(partial_disposes.load(Ordering::Acquire), 1);

    let _loaded = manager
        .Load::<TrackedDisposable>("unload")
        .expect("load resource whose first dispose fails");
    let unload_error = manager.Unload().expect_err("propagate dispose failure");
    assert!(unload_error
        .to_string()
        .contains("injected content dispose failure"));
    assert_eq!(unload_disposes.load(Ordering::Acquire), 1);
    manager.Dispose().expect("dispose after failed unload");
    manager
        .Dispose()
        .expect("double dispose remains idempotent");
}

#[test]
fn malformed_xnb_headers_are_rejected_before_reader_activation() {
    let root = FixtureRoot::new();
    let valid = xnb(
        &[("Microsoft.Xna.Framework.Content.StringReader", 0)],
        0,
        &[1, 0],
    );

    let mut bad_magic = valid.clone();
    bad_magic[0] = b'Z';
    root.write("magic", &bad_magic);
    let mut compressed = valid.clone();
    compressed[5] = 0x80;
    root.write("compressed", &compressed);
    let mut bad_size = valid;
    bad_size[6..10].copy_from_slice(&11_u32.to_le_bytes());
    root.write("size", &bad_size);

    let manager = root.manager();
    for (asset, detail) in [
        ("magic", "invalid XNB magic"),
        ("compressed", "LZX-compressed"),
        ("size", "does not match stream size"),
    ] {
        let error = manager.Load::<String>(asset).expect_err("malformed XNB");
        assert!(error.to_string().contains(detail), "{error}");
    }
}

struct SingleResource {
    name: String,
    bytes: Vec<u8>,
}

impl ContentResourceProvider for SingleResource {
    fn GetObject(&self, assetName: &str) -> Result<Option<Vec<u8>>> {
        Ok((assetName == self.name).then(|| self.bytes.clone()))
    }
}

#[test]
fn resource_content_manager_uses_the_same_reader_and_cache_pipeline() {
    let mut body = vec![1];
    write_string(&mut body, "resource content");
    let resource: Arc<dyn ContentResourceProvider> = Arc::new(SingleResource {
        name: "embedded".to_owned(),
        bytes: xnb(
            &[("Microsoft.Xna.Framework.Content.StringReader", 0)],
            0,
            &body,
        ),
    });
    let services: Arc<GameServiceContainer> = Arc::new(GameServiceContainer::new());
    let provider: Arc<dyn ServiceProvider> = services;
    let manager = ResourceContentManager::new(provider, resource);

    let first =
        ContentManagerBase::Load::<String>(&manager, "embedded").expect("load embedded resource");
    let second =
        ContentManagerBase::Load::<String>(&manager, "embedded").expect("cache embedded resource");
    assert_eq!(first.as_str(), "resource content");
    assert!(Arc::ptr_eq(&first, &second));
    assert!(ContentManagerBase::Load::<String>(&manager, "missing").is_err());
}

#[derive(Default)]
struct ContentGame {
    state: Arc<GameState>,
}

impl GameStateAccess for ContentGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ContentGame {}

#[test]
fn game_content_identity_and_disposal_order_are_stable() {
    const READER: &str = "CnaRust.Tests.GameContentTrackedReader";

    let dispose_count = Arc::new(AtomicUsize::new(0));
    let count_for_reader = Arc::clone(&dispose_count);
    let _reader = ContentTypeReaderRegistry::Register::<TrackedDisposable, _>(
        READER,
        0,
        false,
        move |_, _| {
            Ok(Arc::new(TrackedDisposable {
                dispose_count: Arc::clone(&count_for_reader),
                fail_first_dispose: false,
            }))
        },
    )
    .expect("register game content reader");

    let root = FixtureRoot::new();
    root.write("game-resource", &xnb(&[(READER, 0)], 0, &[1]));
    let manager = Arc::new(root.manager());
    let _resource = manager
        .Load::<TrackedDisposable>("game-resource")
        .expect("load game-owned resource");

    let mut game = ContentGame::default();
    assert!(Arc::ptr_eq(&game.Content(), &game.Content()));
    game.SetContent(Arc::clone(&manager));
    assert!(Arc::ptr_eq(&game.Content(), &manager));

    let count_at_disposed_event = Arc::new(AtomicUsize::new(usize::MAX));
    let event_count = Arc::clone(&count_at_disposed_event);
    let dispose_count_for_event = Arc::clone(&dispose_count);
    game.AddDisposedHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
        event_count.store(
            dispose_count_for_event.load(Ordering::Acquire),
            Ordering::Release,
        );
    }));
    game.Dispose();

    assert_eq!(dispose_count.load(Ordering::Acquire), 1);
    assert_eq!(count_at_disposed_event.load(Ordering::Acquire), 1);
    assert!(manager.Load::<TrackedDisposable>("game-resource").is_err());
}
