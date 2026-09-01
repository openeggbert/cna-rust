//! RUST-UPSTREAM-023: building independent `GraphicsDevice`s from several
//! threads at once must not corrupt the process.
//!
//! `cna_graphics_device_create` races with itself on the GL renderers -- the
//! SDL3/EGL context construction underneath it is not thread-safe, and the ABI
//! neither documents a thread affinity nor answers `CNA_RESULT_THREAD`. The
//! binding serialises construction so that safe Rust does not expose the race;
//! this test is what says the serialisation is still there.
//!
//! It is a *crash* test. There is no assertion that catches the defect: an
//! unserialised build dies with `SIGSEGV` or glibc's "double free or
//! corruption" and the harness reports the signal. So the test also asserts
//! the outcomes it can see -- every thread agrees on whether the renderer can
//! supply a windowless device, and every device that was created is usable
//! afterwards -- because a test that only has to not crash is a test that
//! passes when the call is quietly skipped.

use std::sync::mpsc;
use std::thread;

use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::GraphicsDeviceInformation;
use cna::{CnaError, ErrorCategory, Result};

const THREADS: usize = 6;

fn build() -> Result<GraphicsDevice> {
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    )
}

/// True when the failure is the one refusal a windowless GL renderer gives,
/// which is a legitimate answer for this call and not the defect.
fn is_the_no_window_refusal(error: &CnaError) -> bool {
    matches!(
        error,
        CnaError::Native { category: ErrorCategory::Platform, message, .. }
            if message.contains("platform window id")
    )
}

#[test]
fn six_threads_may_build_a_device_at_once() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }

    let (sender, receiver) = mpsc::channel();
    let workers: Vec<_> = (0..THREADS)
        .map(|slot| {
            let sender = sender.clone();
            thread::spawn(move || {
                // Every thread reports what it got before any device drops, so
                // construction is what overlaps rather than teardown.
                match build() {
                    Ok(device) => {
                        // Touch the device on the thread that made it: a
                        // handle that was raced into existence would not
                        // survive being asked anything.
                        let disposed = device.IsDisposed();
                        sender.send((slot, Ok(disposed))).unwrap();
                        drop(device);
                    }
                    Err(error) => sender.send((slot, Err(error))).unwrap(),
                }
            })
        })
        .collect();
    drop(sender);

    let mut created = 0usize;
    let mut refused = 0usize;
    for (slot, outcome) in receiver {
        match outcome {
            Ok(disposed) => {
                assert_eq!(
                    disposed.as_ref().ok(),
                    Some(&false),
                    "thread {slot} built a device that would not answer IsDisposed: {disposed:?}"
                );
                created += 1;
            }
            Err(error) => {
                assert!(
                    is_the_no_window_refusal(&error),
                    "thread {slot} failed with something other than the renderer's \
                     no-window refusal, which is the only failure this tolerates: {error:?}"
                );
                refused += 1;
            }
        }
    }
    for worker in workers {
        worker.join().expect("a device-building thread panicked");
    }

    assert_eq!(created + refused, THREADS, "a thread reported nothing");
    // Whether this renderer can build a windowless device is a property of the
    // renderer, not of the thread that asked. A split answer would mean the
    // race changed the outcome rather than only the memory.
    assert!(
        created == THREADS || refused == THREADS,
        "the {THREADS} threads disagreed about whether this renderer can supply a \
         windowless device: {created} built one, {refused} were refused"
    );
    println!("NOTE: {created} built, {refused} refused the windowless device");
}
