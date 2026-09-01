//! `graphics_resource.h`: what CNA records about a resource, and where that
//! differs from what XNA's own base class says.
//!
//! The measurement that made this slice worth doing: before it, `Name` and
//! `SetName` lived in a Rust `Mutex` and CNA never heard about them. This file
//! asserts they now round-trip through CNA, and pins the three places where
//! the native property and its XNA neighbour are deliberately *not* the same
//! value.

use cna::extensions::graphics_resource::NativeGraphicsResource;
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, GraphicsResource, PresentationParameters, Texture2D,
};
use cna::Microsoft::Xna::Framework::GraphicsDeviceInformation;
use cna::{CnaError, ErrorCategory};

fn device_or_skip() -> Option<GraphicsDevice> {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return None;
    }
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    match GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    ) {
        Ok(device) => Some(device),
        Err(CnaError::Native {
            category: ErrorCategory::Platform,
            ref message,
            ..
        }) if message.contains("platform window id") => {
            println!("this renderer cannot create a device without a window: {message}");
            None
        }
        Err(error) => panic!("independent GraphicsDevice construction failed: {error:?}"),
    }
}

#[test]
fn a_name_set_through_rust_is_the_name_cna_reports() {
    let Some(device) = device_or_skip() else { return };
    let mut texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");

    // A fresh resource is unnamed on both sides.
    assert_eq!(texture.Name(), "", "a fresh resource has no name");

    texture.SetName("hero diffuse");
    assert_eq!(texture.Name(), "hero diffuse", "the name reads back");
    assert_eq!(
        texture.native_to_string().expect("CNA's ToString"),
        "hero diffuse",
        "once named, CNA's ToString is the name -- the same rule XNA has"
    );

    // The regression this pins: before `graphics_resource.h` was bound the
    // name lived only in Rust, so CNA reported an empty one here and the
    // device's ResourceDestroyed event did too.
    texture.SetName("");
    assert_eq!(
        texture.native_to_string().expect("CNA's ToString"),
        "Texture2D",
        "cleared through Rust, CNA falls back to its own type name -- which is \
         proof the clear reached CNA rather than only the Rust mirror"
    );
}

#[test]
fn cnas_to_string_and_xnas_are_different_strings_on_purpose() {
    let Some(device) = device_or_skip() else { return };
    let texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");

    let xna = texture.ToString();
    let cna = texture.native_to_string().expect("CNA's ToString");
    println!("NOTE: XNA ToString {xna:?} vs CNA {cna:?}");

    // XNA's GraphicsResource.ToString falls back to Object.ToString(), the
    // namespace-qualified type name -- verified against the decompiled
    // reference. CNA's falls back to the bare one. Neither is folded into the
    // other, so both spellings are asserted.
    assert_eq!(xna, "Microsoft.Xna.Framework.Graphics.Texture2D");
    assert_eq!(cna, "Texture2D");
}

#[test]
fn the_native_tag_is_not_the_xna_tag() {
    let Some(device) = device_or_skip() else { return };
    let mut texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");

    assert_eq!(texture.native_tag().expect("the native tag"), 0, "null to start");
    texture.set_native_tag(0x5EED).expect("set the native tag");
    assert_eq!(texture.native_tag().expect("the native tag"), 0x5EED);

    // XNA's Tag is an arbitrary managed object and lives in Rust; setting one
    // must not disturb the C token, and vice versa.
    texture.SetTag(Some(std::sync::Arc::new(7_u32)));
    assert_eq!(
        texture.native_tag().expect("the native tag"),
        0x5EED,
        "an XNA Tag does not overwrite CNA's token"
    );
    assert!(
        texture
            .Tag()
            .and_then(|tag| tag.downcast_ref::<u32>().copied())
            == Some(7),
        "and the XNA Tag is still the Rust object"
    );
}

#[test]
fn disposing_in_place_leaves_the_handle_usable() {
    let Some(device) = device_or_skip() else { return };
    let mut texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");
    texture.SetName("about to go");

    assert!(!texture.native_is_disposed().expect("CNA's disposal flag"));
    assert!(!texture.IsDisposed(), "and Rust's, which is a different question");

    texture.dispose_in_place().expect("dispose without releasing");

    assert!(
        texture.native_is_disposed().expect("CNA's disposal flag"),
        "CNA now reports the object disposed"
    );
    assert!(
        !texture.IsDisposed(),
        "but the C handle was not released, so Rust still holds one -- which is \
         exactly the difference between cna_graphics_resource_dispose and \
         cna_texture2d_destroy"
    );
    assert_eq!(
        texture.Name(),
        "about to go",
        "a disposed resource still answers its name"
    );

    // Repeated disposal is a documented no-op, and dropping the Rust value
    // still destroys the handle. Doing both must not fault.
    texture.dispose_in_place().expect("repeated disposal is a no-op");
}

#[test]
fn the_native_disposing_event_fires_for_cnas_own_disposal() {
    let Some(device) = device_or_skip() else { return };
    let texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");

    let fired = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = std::sync::Arc::clone(&fired);
    let subscription = texture
        .on_native_disposing(move || {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .expect("subscribe to CNA's disposing event");

    assert_eq!(fired.load(std::sync::atomic::Ordering::SeqCst), 0);
    texture.dispose_in_place().expect("dispose in place");
    let after_dispose = fired.load(std::sync::atomic::Ordering::SeqCst);
    println!("NOTE: native disposing fired {after_dispose} time(s)");
    assert_eq!(
        after_dispose, 1,
        "CNA raises Disposing exactly once for the disposal it performed"
    );

    // A second disposal is a no-op, so it must not raise again.
    texture.dispose_in_place().expect("repeated disposal");
    assert_eq!(
        fired.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "a no-op disposal raises nothing"
    );

    // Withdrawing frees the boxed closure; the order matters and the type
    // enforces it. Dropping the texture afterwards must not call into it.
    subscription.unsubscribe().expect("withdraw the registration");
    drop(subscription);
    drop(texture);
}

#[test]
fn a_device_owned_resource_will_not_lend_its_device_outside_a_callback() {
    let Some(device) = device_or_skip() else { return };
    let texture = Texture2D::new(&device, 4, 4).expect("a 4x4 texture");

    // Documented: a device-owned resource lends its device only while its game
    // is inside a lifecycle callback. This test is outside one, so the refusal
    // is the correct answer and is reported rather than folded into `None`.
    match texture.device_in_callback() {
        Ok(None) => println!("NOTE: this resource is standalone and has no owning device"),
        Ok(Some(_)) => panic!(
            "a device handle was lent outside a lifecycle callback, which the header \
             says cannot happen"
        ),
        Err(error) => println!("NOTE: refused outside a callback, as documented: {error}"),
    }
}
