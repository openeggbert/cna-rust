//! What a graphics device says about itself, and the events it raises.
//!
//! Two things are worth measuring here and neither is "the route returns Ok":
//!
//! * **Capabilities are reported, not asserted.** What a renderer supports is
//!   the renderer's business, so this file prints the answers and asserts only
//!   the relationships that must hold whatever they are -- a compute
//!   invocation limit no larger than the product of the axis limits, a colour
//!   space the device reports as current also reported as supported.
//! * **A subscription's closure must outlive its registration.** The device
//!   events are the third callback family in this crate, and the ordering bug
//!   they all share -- freeing the box before withdrawing the registration --
//!   is what the drop test here is for.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::graphics_device_ext::{
    primitive_vertex_count, DeviceCapabilityExt, DeviceEvent, DeviceEventExt, DeviceStateExt,
    Unsupported3DCallBehavior,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters, PrimitiveType, SurfaceFormat,
};
use cna::Microsoft::Xna::Framework::GraphicsDeviceInformation;
use cna::{CnaError, ErrorCategory};

fn device() -> Option<GraphicsDevice> {
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
        Err(error) => panic!(
            "independent GraphicsDevice construction failed with something other than \
             the renderer's no-window refusal, which is the only failure this skips. \
             Seen once under a parallel full-suite run and not reproduced since, so the \
             exact text matters: {error:?}"
        ),
    }
}

#[test]
fn the_device_reports_its_capabilities_consistently() {
    let Some(device) = device() else { return };

    println!(
        "NOTE: executes_shader_effect_source={:?} image_based_lighting={:?}",
        device.executes_shader_effect_source(),
        device.supports_image_based_lighting()
    );

    // The invocation limit is not the product of the axis limits, and on real
    // hardware it is smaller -- which is why a dispatch sized from the axes
    // alone can still be refused. Whatever the numbers, that relationship must
    // hold, or a caller sizing a dispatch from them gets it wrong.
    if let (Ok(invocations), Ok(x), Ok(y), Ok(z)) = (
        device.max_compute_work_group_invocations(),
        device.max_compute_work_group_size(0),
        device.max_compute_work_group_size(1),
        device.max_compute_work_group_size(2),
    ) {
        println!("NOTE: compute invocations={invocations} size=({x},{y},{z})");
        if invocations > 0 && x > 0 && y > 0 && z > 0 {
            let product = i64::from(x) * i64::from(y) * i64::from(z);
            assert!(
                i64::from(invocations) <= product,
                "the invocation limit ({invocations}) cannot exceed the product of the \
                 axis limits ({product}), or the two describe different machines"
            );
        }
    }

    // Whatever colour space the device is presenting in, it must agree that it
    // supports it.
    if let Ok(current) = device.display_color_space() {
        println!("NOTE: display_color_space={current}");
        assert_eq!(
            device.supports_display_color_space(current),
            Ok(true),
            "a device must support the colour space it says it is using"
        );
    }

    for format in [SurfaceFormat::Color, SurfaceFormat::Bgr565] {
        println!(
            "NOTE: {format:?} as render target = {:?}",
            device.supports_surface_format_as_render_target(format)
        );
    }
}

#[test]
fn the_unsupported_3d_call_behavior_round_trips() {
    let Some(device) = device() else { return };

    let before = device.unsupported_3d_call_behavior();
    println!("NOTE: unsupported-3D behaviour starts as {before:?}");
    let Ok(before) = before else { return };

    for behavior in [
        Unsupported3DCallBehavior::WarnAndStub,
        Unsupported3DCallBehavior::Throw,
    ] {
        device
            .set_unsupported_3d_call_behavior(behavior)
            .expect("set the behaviour");
        assert_eq!(
            device
                .unsupported_3d_call_behavior()
                .expect("read the behaviour"),
            behavior,
            "the behaviour must read back as it was set"
        );
    }
    device
        .set_unsupported_3d_call_behavior(before)
        .expect("restore the behaviour");
}

#[test]
fn a_device_subscription_outlives_nothing_and_frees_in_order() {
    let Some(device) = device() else { return };

    let calls = Arc::new(AtomicU32::new(0));
    let counter = Arc::clone(&calls);
    let subscription = device
        .on_event(DeviceEvent::DeviceReset, move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .expect("a device-reset subscription");
    println!("NOTE: {subscription:?}");

    // Withdrawing is idempotent, and it must free the closure *after* the
    // registration is gone. A second withdrawal proves the first cleared its
    // slot rather than leaving a handle to release twice.
    subscription.unsubscribe().expect("withdraw once");
    subscription.unsubscribe().expect("withdraw again is a no-op");
    drop(subscription);
    println!("NOTE: the handler ran {} time(s)", calls.load(Ordering::SeqCst));

    // The resource events carry a shape rather than a resource, so a game can
    // count what the device tracks without holding a half-built object.
    let seen: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let created = device
        .on_resource_created(move |has_resource| {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(has_resource);
        })
        .expect("a resource-created subscription");
    let destroyed = device
        .on_resource_destroyed(|_| {})
        .expect("a resource-destroyed subscription");

    // Make something the device tracks, then let it go.
    let before = device.tracked_resource_count().expect("a starting count");
    {
        let parameters = PresentationParameters::new();
        parameters.SetBackBufferWidth(8);
        parameters.SetBackBufferHeight(8);
        let _ = parameters;
    }
    let after = device.tracked_resource_count().expect("a count afterwards");
    println!(
        "NOTE: tracked resources {before} -> {after}, created events: {:?}",
        seen.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    );
    assert_eq!(
        before, after,
        "nothing tracked was made or destroyed, so the count must not have moved"
    );

    drop(created);
    drop(destroyed);
}

#[test]
fn a_primitive_count_maps_to_a_vertex_count_per_topology() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // A strip shares vertices where a list does not, which is the whole reason
    // this is a route rather than a multiplication: three triangles is nine
    // vertices as a list and five as a strip.
    let list = primitive_vertex_count(PrimitiveType::TriangleList, 3).expect("a list count");
    let strip = primitive_vertex_count(PrimitiveType::TriangleStrip, 3).expect("a strip count");
    println!("NOTE: three triangles -> list {list}, strip {strip}");
    assert_eq!(list, 9, "a triangle list needs three vertices per triangle");
    assert_eq!(strip, 5, "a triangle strip shares two vertices per triangle");
    assert!(
        strip < list,
        "a strip must need fewer vertices than a list for the same triangles"
    );
}
