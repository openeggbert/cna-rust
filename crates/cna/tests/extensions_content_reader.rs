//! `content_readers.h`: reading a compiled asset, and teaching CNA a new type.
//!
//! The registry half is the part that matters most. `content_readers.h` has two
//! extension points and this binding projects one of them; the other -- the
//! reflective builder, which writes at caller-supplied byte offsets -- is a
//! deliberate non-binding because a wrapper cannot check either the offset or
//! the kind. So these tests are what say the projected one actually works:
//! register a reader, see it in the registry, create an instance, read its
//! declared metadata back, and withdraw it.
//!
//! The reader half runs over a real `StorageStream`, so `read_bytes_exact` and
//! the two limit checks are measured against bytes on disk rather than a mock.

use std::sync::Arc;

use cna::extensions::content_reader::{
    clear_all_type_readers, register_type_reader, type_reader_is_registered, ContentReader,
    ContentReaderView, ContentReads, ContentTypeReader, TypeReader, UnsupportedReason,
};
use cna::Microsoft::Xna::Framework::Storage::{StorageContainer, StorageDevice};
use cna::{FileMode, StorageStream};
use cna::{CnaError, ErrorCategory, Result};

fn skip() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_none()
}

/// A reader that records what it was asked and produces a boxed integer.
struct CountingReader {
    name: String,
    reads: Arc<std::sync::atomic::AtomicUsize>,
}

impl TypeReader for CountingReader {
    fn target_type_name(&self) -> &str {
        &self.name
    }
    fn type_version(&self) -> i32 {
        7
    }
    fn can_read_into_existing(&self) -> bool {
        true
    }
    fn read(
        &self,
        _reader: &ContentReaderView<'_>,
        _existing: *mut core::ffi::c_void,
    ) -> Result<*mut core::ffi::c_void> {
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // The pointer is the caller's own and CNA never touches it; whoever
        // asked for the asset receives it. Driven through `read_and_discard`
        // below, nobody receives it and this leaks eight bytes per call --
        // which is the route's documented shape, not a defect, and is why this
        // reader allocates something trivial rather than something worth
        // reclaiming.
        Ok(Box::into_raw(Box::new(42_u64)).cast())
    }
}

#[test]
fn a_rust_type_reader_registers_and_reports_what_it_declared() {
    if skip() {
        return;
    }
    let canonical = format!("Test.Reader.Counting,{}", std::process::id());
    let target = format!("Test.Target.Counting,{}", std::process::id());
    assert!(
        !type_reader_is_registered(&canonical).expect("query an unregistered name"),
        "a name nobody registered is not registered"
    );

    let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let registration = register_type_reader(
        &canonical,
        Box::new(CountingReader {
            name: target.clone(),
            reads: Arc::clone(&reads),
        }),
    )
    .expect("register a reader");

    assert!(
        type_reader_is_registered(&canonical).expect("query the registered name"),
        "the registry now knows the name"
    );

    // Upstream's deliberate deviation from the canonical AddTypeCreator: a
    // duplicate is refused rather than silently ignored, so a caller who lost
    // the race finds out here instead of from assets deserializing wrongly.
    let duplicate = register_type_reader(
        &canonical,
        Box::new(CountingReader {
            name: target.clone(),
            reads: Arc::clone(&reads),
        }),
    )
    .expect_err("a second registration under the same name is refused");
    println!("NOTE: duplicate registration -> {duplicate}");
    assert!(
        matches!(
            duplicate,
            CnaError::Native {
                category: ErrorCategory::State,
                ..
            }
        ),
        "the refusal is a state error, not an argument one: {duplicate:?}"
    );

    // An instance carries the metadata the registration declared.
    let instance = ContentTypeReader::for_name(&canonical).expect("create an instance");
    assert_eq!(
        instance.target_type_name().expect("target type name"),
        target,
        "the instance reports the target type the registration declared"
    );
    assert_eq!(instance.type_version().expect("type version"), 7);
    assert!(
        instance
            .read_into_existing_is_allowed()
            .expect("read-into-existing flag"),
        "and the flag the registration set"
    );
    assert!(
        instance.supports_version(7).expect("supports its own version"),
        "a reader supports the version it declares"
    );
    instance.initialize().expect("initialize");

    // Each file gets a fresh instance, so two creates are two handles.
    let second = ContentTypeReader::for_name(&canonical).expect("a second instance");
    assert_eq!(second.target_type_name().expect("target"), target);
    drop(second);

    // The callback itself, driven through CNA rather than called directly.
    // `read_and_discard` runs the type reader against a content reader, which
    // is the only way from Rust to prove the trampoline, the borrowed view and
    // the produced pointer all work -- registering a reader that is never
    // invoked would qualify nothing.
    let payload: Vec<u8> = (0..16_u8).collect();
    if let Some((stream, container)) = stream_over(&payload, "callback.bin") {
        let content = ContentReader::new(stream, "callback.bin", 5, 1)
            .expect("a content reader for the callback");
        let produced = instance
            .read_and_discard(&content)
            .expect("run the reader through CNA");
        assert!(
            produced,
            "the callback returned a non-null object, so CNA reports it produced one"
        );
        assert_eq!(
            reads.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "and the callback ran exactly once"
        );

        // Twice, to show the instance is reusable and the count tracks calls
        // rather than instances.
        instance
            .read_and_discard(&content)
            .expect("run it again");
        assert_eq!(reads.load(std::sync::atomic::Ordering::SeqCst), 2);

        let _ = container.DeleteFile("callback.bin");
    } else {
        println!("this host has no storage device; the read callback is not driven");
    }

    // Withdrawing takes the name out of the registry.
    registration.unregister().expect("withdraw");
    assert!(
        !type_reader_is_registered(&canonical).expect("query after withdrawal"),
        "the name is gone once the registration is withdrawn"
    );
    let gone = ContentTypeReader::for_name(&canonical)
        .expect_err("an unregistered name creates nothing");
    println!("NOTE: after withdrawal -> {gone}");
    assert!(
        matches!(
            gone,
            CnaError::Native {
                category: ErrorCategory::NotSupported,
                ..
            }
        ),
        "an unregistered name is NOT_SUPPORTED, which is a different answer from a \
         reader that exists and refuses: {gone:?}"
    );

    // Withdrawing twice is a no-op, and dropping after that must not double-free
    // the boxed reader.
    registration.unregister().expect("withdrawing twice is a no-op");
    drop(registration);
}

#[test]
fn a_known_unsupported_reader_says_why_rather_than_saying_nothing() {
    if skip() {
        return;
    }
    let reader = ContentTypeReader::known_unsupported(
        "Microsoft.Xna.Framework.Graphics.Effect",
        UnsupportedReason::CompiledPlatformShaderBytecode,
    )
    .expect("a placeholder reader");

    assert_eq!(
        reader.target_type_name().expect("target type name"),
        "Microsoft.Xna.Framework.Graphics.Effect",
        "the placeholder carries the type it stands in for -- which is the whole \
         point: an asset naming it fails with the reason rather than with \
         'no such reader'"
    );
    println!(
        "NOTE: placeholder version {}, reads into existing {}",
        reader.type_version().expect("type version"),
        reader
            .read_into_existing_is_allowed()
            .expect("read-into-existing flag")
    );
}

/// Writes `bytes` into a storage file and hands back a stream over them.
fn stream_over(bytes: &[u8], name: &str) -> Option<(StorageStream, StorageContainer)> {
    let device = StorageDevice::EndShowSelector(
        &StorageDevice::BeginShowSelectorWithCallbackAndState(None, None).ok()?,
    )
    .ok()?;
    let container_name = format!("cna-rust-content-reader-{}", std::process::id());
    let open = device.BeginOpenContainer(&container_name, None, None).ok()?;
    let container = device.EndOpenContainer(&open).ok()?;

    {
        use std::io::Write;
        let mut writing = container.CreateFile(name).ok()?;
        writing.write_all(bytes).ok()?;
        writing.flush().ok()?;
    }
    let reading = container.OpenFile(name, FileMode::Open).ok()?;
    Some((reading, container))
}

#[test]
fn a_reader_reads_exact_bytes_and_enforces_its_own_limits() {
    if skip() {
        return;
    }
    let payload: Vec<u8> = (0..32_u8).collect();
    let Some((stream, container)) = stream_over(&payload, "reader.bin") else {
        println!("this host has no storage device; nothing to read from");
        return;
    };

    let reader = ContentReader::new(stream, "reader.bin", 5, 1).expect("a content reader");

    assert_eq!(
        reader.asset_name().expect("asset name"),
        "reader.bin",
        "the reader reports the logical name it was given"
    );
    assert_eq!(reader.version().expect("version"), 5);
    assert_eq!(reader.platform().expect("platform"), 1);
    assert_eq!(
        reader.content_manager_handle().expect("manager"),
        None,
        "a standalone reader has no content manager, which is a state and not a failure"
    );

    let first = reader
        .read_bytes_exact(8, "Test.Reader")
        .expect("eight bytes");
    assert_eq!(first, payload[..8], "the bytes come back in order");
    let second = reader
        .read_bytes_exact(8, "Test.Reader")
        .expect("eight more");
    assert_eq!(
        second,
        payload[8..16],
        "and the reader advanced -- a second read is the *next* eight, not the same"
    );

    // Asking for more than remains is refused rather than short-read, which is
    // what "exact" means and what stops a reader building an object out of
    // whatever happened to follow.
    let over = reader.read_bytes_exact(1_000, "Test.Reader");
    assert!(over.is_err(), "a read past the end is refused: {over:?}");
    println!("NOTE: over-read -> {:?}", over.err().map(|e| e.to_string()));

    // The limit checks. A sane count passes; a negative one is refused, which
    // is what stops a corrupt file's element count reaching an allocation.
    reader
        .check_element_count(16, "Test.Reader")
        .expect("a small count is within the limit");
    let negative = reader.check_element_count(-1, "Test.Reader");
    assert!(negative.is_err(), "a negative element count is refused");
    let huge = reader.check_element_count(i64::MAX, "Test.Reader");
    assert!(
        huge.is_err(),
        "and so is one past the reader's limit, which is the whole reason the check \
         exists: {huge:?}"
    );
    println!("NOTE: huge count -> {:?}", huge.err().map(|e| e.to_string()));

    reader
        .check_decoded_size(1024, "Test.Reader")
        .expect("a small decoded size is within the limit");
    assert!(
        reader.check_decoded_size(-1, "Test.Reader").is_err(),
        "a negative decoded size is refused"
    );

    reader.release().expect("release");
    reader.release().expect("releasing twice is a no-op");
    assert!(
        reader.asset_name().is_err(),
        "a released reader answers a refusal rather than reaching a freed handle"
    );

    let _ = container.DeleteFile("reader.bin");
}

#[test]
fn clearing_the_registry_is_process_wide_and_says_so() {
    if skip() {
        return;
    }
    // Not run: `clear_all_type_readers` removes the built-in readers too, so
    // calling it would leave the rest of this binary's tests -- and anything
    // else in the process -- unable to read any compiled asset. It is bound
    // because a test that wants a known-empty registry has no other way to get
    // one, and it is documented as process-wide for exactly this reason.
    //
    // What *is* asserted is that the function exists with the shape a caller
    // would use, so a signature change is caught here.
    let _: fn() -> Result<()> = clear_all_type_readers;
    println!("NOTE: clear_all_type_readers is bound and deliberately not exercised here");
}
