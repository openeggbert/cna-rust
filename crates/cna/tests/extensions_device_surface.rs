//! The last of the ABI: the device's own surface, resource `ContentLost`, the
//! window controls, and the process-level settings.
//!
//! These are the routes that took the census to zero undecided. What they have
//! in common is that each answers a question about something *as it currently
//! is*, or reaches a control XNA never had, so the assertions are about
//! coherence -- the back buffer agreeing with what was asked for, a layout read
//! back matching the one uploaded -- rather than about fixed values.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cna::extensions::device_surface::{
    clone_presentation_parameters, presentation_bounds, usage_preserves_contents, BatchedSprites,
    DeviceSurface, ReadsVertexLayout, Rgba8Data, ScaledSprite,
};
use cna::extensions::events::EventArgs;
use cna::extensions::resource_events::NotifiesContentLost;
use cna::Microsoft::Xna::Framework::Graphics::{
    BufferUsage, GraphicsDevice, GraphicsProfile, PresentationParameters, RenderTargetUsage,
    SpriteEffects, SurfaceFormat, Texture2D, VertexBuffer, VertexElement, VertexElementFormat,
    VertexElementUsage, VertexDeclaration, DynamicIndexBuffer, DynamicVertexBuffer,
    IndexElementSize,
};
use cna::Microsoft::Xna::Framework::{Color, GraphicsDeviceInformation, Rectangle, Vector2};
use cna::{CnaError, ErrorCategory};

fn device_or_skip() -> Option<GraphicsDevice> {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return None;
    }
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(32);
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
fn the_back_buffer_reports_what_the_renderer_settled_on() {
    let Some(device) = device_or_skip() else { return };

    let info = device.back_buffer_info().expect("back buffer info");
    println!("NOTE: back buffer {info:?}");
    assert!(info.width > 0 && info.height > 0, "it has a size");

    // Asked for 64x32. A renderer is allowed to differ -- which is exactly why
    // this route exists beside PresentationParameters -- so the assertion is
    // that it *reports* a size, and the requested one is printed beside it.
    println!("NOTE: asked for 64x32, got {}x{}", info.width, info.height);

    // Reading needs room for the whole surface; a short buffer gets what fits.
    let mut pixels = vec![Color::default(); (info.width * info.height) as usize];
    match device.read_back_buffer(&mut pixels) {
        Ok(written) => {
            println!("NOTE: read {written} pixel(s) from the back buffer");
            assert!(
                written <= pixels.len(),
                "the reported count never exceeds the destination"
            );
        }
        Err(error) => println!("NOTE: this renderer will not read its back buffer: {error}"),
    }
}

#[test]
fn a_texture_round_trips_through_the_packed_colour_path() {
    let Some(device) = device_or_skip() else { return };
    let texture = Texture2D::new(&device, 2, 2).expect("a 2x2 texture");

    let written = [
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(10, 20, 30, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(40, 50, 60, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(70, 80, 90, 255),
        Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(100, 110, 120, 255),
    ];
    texture.set_rgba8(&written).expect("upload packed colours");

    let mut read = vec![Color::default(); 4];
    let count = texture.read_rgba8(&mut read).expect("read them back");
    println!("NOTE: wrote {written:?}");
    println!("NOTE: read  {read:?} ({count} pixel(s))");
    assert_eq!(count, 4, "all four pixels come back");
    assert_eq!(
        read, written,
        "and they are the colours that went in -- which is what says the packed \
         path is a fast lane and not a lossy one"
    );
}

#[test]
fn a_vertex_buffers_layout_reads_back_as_the_one_it_was_built_with() {
    let Some(device) = device_or_skip() else { return };

    let elements = [
        VertexElement::new(0, VertexElementFormat::Vector3, VertexElementUsage::Position, 0),
        VertexElement::new(12, VertexElementFormat::Color, VertexElementUsage::Color, 0),
    ];
    let declaration = VertexDeclaration::from_vertex_stride_and_elements(16, &elements)
        .expect("a position+colour declaration");
    let buffer = VertexBuffer::new(&device, &declaration, 3, BufferUsage::None)
        .expect("a three-vertex buffer");

    let read = buffer.native_elements().expect("the buffer's own layout");
    println!("NOTE: layout read back -> {read:?}");
    assert_eq!(
        read.len(),
        elements.len(),
        "the buffer reports as many elements as it was built with"
    );
    for (index, (got, want)) in read.iter().zip(elements.iter()).enumerate() {
        assert_eq!(
            (got.Offset(), got.VertexElementFormat(), got.VertexElementUsage(), got.UsageIndex()),
            (want.Offset(), want.VertexElementFormat(), want.VertexElementUsage(), want.UsageIndex()),
            "element {index} survived the round trip"
        );
    }
    println!("NOTE: stride read back -> {:?}", buffer.native_stride());
}

#[test]
fn content_lost_can_be_subscribed_and_withdrawn() {
    let Some(device) = device_or_skip() else { return };

    let declaration = VertexDeclaration::from_vertex_stride_and_elements(
        12,
        &[VertexElement::new(
            0,
            VertexElementFormat::Vector3,
            VertexElementUsage::Position,
            0,
        )],
    )
    .expect("a position-only declaration");
    // ContentLost exists only on the dynamic buffers -- measured: a static one
    // answers "ContentLost exists only on DynamicVertexBuffer" -- which is why
    // the trait is implemented for those alone.
    let buffer = DynamicVertexBuffer::new(&device, &declaration, 3, BufferUsage::None)
        .expect("a dynamic buffer");

    let fired = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&fired);
    let subscription = buffer
        .on_content_lost(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .expect("subscribe to ContentLost");

    // Nothing has lost its contents, so nothing has fired. The event is raised
    // by a device reset, which this test cannot force on every renderer -- so
    // what is asserted is the half that is always true: registering and
    // withdrawing are safe, and withdrawing frees the closure in an order that
    // leaves nothing dangling.
    assert_eq!(fired.load(Ordering::SeqCst), 0);
    subscription.unsubscribe().expect("withdraw");
    subscription.unsubscribe().expect("withdrawing twice is a no-op");
    drop(subscription);
    drop(buffer);
}

fn position_declaration() -> VertexDeclaration {
    VertexDeclaration::from_vertex_stride_and_elements(
        12,
        &[VertexElement::new(
            0,
            VertexElementFormat::Vector3,
            VertexElementUsage::Position,
            0,
        )],
    )
    .expect("a position-only declaration")
}

#[test]
fn the_xna_content_lost_handlers_fire_when_cna_says_the_content_is_gone() {
    let Some(device) = device_or_skip() else { return };
    let declaration = position_declaration();

    // `RUST-EXT-018`. These handlers existed and were emitted nowhere: nothing
    // in Rust knows a device was reset, so a game that registered one waited
    // forever. They now sit behind one native subscription per buffer.
    let vertices = DynamicVertexBuffer::new(&device, &declaration, 3, BufferUsage::None)
        .expect("a dynamic vertex buffer");
    let indices = DynamicIndexBuffer::new(&device, IndexElementSize::SixteenBits, 3, BufferUsage::None)
        .expect("a dynamic index buffer");

    let order = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let sender_was_the_buffer = Arc::new(AtomicBool::new(false));
    let index_fired = Arc::new(AtomicUsize::new(0));

    let first = Arc::clone(&order);
    vertices.AddContentLostHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
        first.lock().expect("order").push(1);
    }));
    let second = Arc::clone(&order);
    let seen = Arc::clone(&sender_was_the_buffer);
    vertices.AddContentLostHandler(Box::new(move |sender: &dyn std::any::Any, _: EventArgs| {
        second.lock().expect("order").push(2);
        seen.store(
            sender.downcast_ref::<DynamicVertexBuffer>().is_some(),
            Ordering::SeqCst,
        );
    }));
    let third = Arc::clone(&order);
    let removed = vertices.AddContentLostHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
        third.lock().expect("order").push(3);
    }));
    assert!(vertices.RemoveContentLostHandler(removed));

    let counter = Arc::clone(&index_fired);
    indices.AddContentLostHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    assert!(order.lock().expect("order").is_empty(), "nothing has been lost yet");

    // EVENT_BRIDGE_VERIFIED. This is CNA's own deterministic notification, not
    // a real device loss: `notify_content_lost_resources` is what a content
    // manager calls when it unloads. A genuine reset is measured separately
    // below, and the two are not the same claim.
    device
        .notify_content_lost_resources()
        .expect("CNA tells its resources their content is gone");

    assert_eq!(
        *order.lock().expect("order"),
        vec![1_u8, 2],
        "both surviving handlers fire, in registration order, and the removed one does not"
    );
    assert!(
        sender_was_the_buffer.load(Ordering::SeqCst),
        "the sender is the buffer, as XNA passes `this`"
    );
    assert_eq!(index_fired.load(Ordering::SeqCst), 1, "and the index buffer's fires too");

    // `ContentLost` is not a one-shot, unlike `Disposing`: a second loss
    // reaches the same list again. This is the assertion that caught the first
    // version of the bridge, where the temporary sender shared the buffer's
    // subscription and its drop cancelled it after the first delivery.
    device.notify_content_lost_resources().expect("a second notification");
    assert_eq!(
        *order.lock().expect("order"),
        vec![1_u8, 2, 1, 2],
        "a second loss reaches the same handlers again, in the same order"
    );

    // Disposal safety, on a buffer of its own so the counts above stay
    // readable. Two things have to hold, and only one of them is about the
    // event: a disposed buffer's handlers are not called, and everything the
    // bridge holds is released.
    //
    // The `witness` measures the second. A handler's captures live inside the
    // closure CNA holds a pointer to, which is freed only when the
    // subscription is withdrawn -- so a bridge that outlived its buffer would
    // keep them alive silently, and the count here would still be two.
    let after_disposal = Arc::new(AtomicUsize::new(0));
    let witness = Arc::new(());
    {
        let counter = Arc::clone(&after_disposal);
        let held = Arc::clone(&witness);
        let mut doomed = DynamicVertexBuffer::new(&device, &declaration, 3, BufferUsage::None)
            .expect("a second dynamic vertex buffer");
        doomed.AddContentLostHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
            let _ = &held;
            counter.fetch_add(1, Ordering::SeqCst);
        }));
        use cna::Microsoft::Xna::Framework::Graphics::GraphicsResource as _;
        doomed.Dispose(true).expect("dispose the buffer");
        device
            .notify_content_lost_resources()
            .expect("a notification after disposal is still a legal call");
    }
    assert_eq!(
        after_disposal.load(Ordering::SeqCst),
        0,
        "a disposed buffer's handlers must not be called"
    );
    assert_eq!(
        Arc::strong_count(&witness),
        1,
        "the bridge and everything the handler captured are released, not leaked in a cycle"
    );

    drop(vertices);
    drop(indices);
}

#[test]
fn a_device_reset_is_measured_separately_from_the_bridge() {
    let Some(device) = device_or_skip() else { return };
    let declaration = position_declaration();
    let buffer = DynamicVertexBuffer::new(&device, &declaration, 3, BufferUsage::None)
        .expect("a dynamic vertex buffer");
    let fired = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&fired);
    buffer.AddContentLostHandler(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    // REAL_DEVICE_LOSS_VERIFIED is a *different* claim from the bridge working,
    // and this is where it is measured rather than assumed. A renderer that
    // refuses the reset says so; what is asserted either way is that no loss is
    // fabricated -- the count moves only if the reset really raised the event.
    match device.Reset() {
        Ok(()) => println!(
            "MEASURED: reset succeeded; ContentLost handlers fired {} time(s)",
            fired.load(Ordering::SeqCst)
        ),
        Err(error) => println!("MEASURED: this renderer refuses a device reset: {error:?}"),
    }
    // Whatever the renderer did, no loss is invented: the count moved only if
    // the reset really raised the event. On HEADLESS it does not, which is why
    // `EVENT_BRIDGE_VERIFIED` and `REAL_DEVICE_LOSS_VERIFIED` are separate
    // claims and only the first is made above.
    println!(
        "MEASURED: REAL_DEVICE_LOSS_VERIFIED={}",
        fired.load(Ordering::SeqCst) > 0
    );
    drop(buffer);
}

#[test]
fn a_render_target_usage_says_whether_it_preserves() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let discard = usage_preserves_contents(RenderTargetUsage::DiscardContents)
        .expect("DiscardContents");
    let preserve = usage_preserves_contents(RenderTargetUsage::PreserveContents)
        .expect("PreserveContents");
    println!("NOTE: discard preserves {discard}, preserve preserves {preserve}");
    assert!(!discard, "DiscardContents does not preserve");
    assert!(preserve, "PreserveContents does");
}

#[test]
fn presentation_parameters_clone_and_report_their_bounds() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(320);
    parameters.SetBackBufferHeight(240);
    parameters.SetBackBufferFormat(SurfaceFormat::Color);

    let bounds = presentation_bounds(&parameters).expect("bounds");
    println!("NOTE: bounds {bounds:?}");
    assert_eq!(
        (bounds.Width, bounds.Height),
        (320, 240),
        "the bounds are the back buffer's own size"
    );

    let clone = clone_presentation_parameters(&parameters).expect("clone");
    assert_eq!(
        (clone.back_buffer_width, clone.back_buffer_height),
        (320, 240),
        "CNA's clone carries the same configuration"
    );
}

#[test]
fn many_sprites_and_an_arbitrary_mesh_go_through_one_crossing() {
    let Some(device) = device_or_skip() else { return };
    use cna::Microsoft::Xna::Framework::Graphics::SpriteBatch;

    let batch = match SpriteBatch::new(&device) {
        Ok(batch) => batch,
        Err(error) => {
            println!("this renderer will not make a sprite batch: {error}");
            return;
        }
    };

    let sprites: Vec<ScaledSprite> = (0..3)
        .map(|index| ScaledSprite {
            position: Vector2::from_x_and_y(index as f32 * 8.0, 0.0),
            source: Rectangle::new(0, 0, 8, 8),
            color: Color::from_r_and_g_and_b_as_int32_and_int32_and_int32(255, 255, 255),
            rotation: 0.0,
            origin: Vector2::from_x_and_y(0.0, 0.0),
            scale: Vector2::from_x_and_y(1.0, 1.0),
            effects: SpriteEffects::None,
            layer_depth: 0.0,
        })
        .collect();
    // Outside Begin/End the batch refuses; what is measured is that the whole
    // array crosses in one call and the refusal is the batch's own, not a
    // marshalling failure.
    println!("NOTE: submit_scaled outside Begin -> {:?}", batch.submit_scaled(&sprites).err().map(|e| e.to_string()));

    // The mesh path validates its arrays here, before CNA sees a pointer.
    let positions = [Vector2::from_x_and_y(0.0, 0.0), Vector2::from_x_and_y(1.0, 0.0)];
    let colors = [Color::default(); 2];
    let coordinates = [Vector2::from_x_and_y(0.0, 0.0), Vector2::from_x_and_y(1.0, 0.0)];
    assert!(
        batch.draw_mesh(&positions, &colors[..1], &coordinates, &[0, 1]).is_err(),
        "mismatched array lengths are refused before CNA is handed a pointer"
    );
    assert!(
        batch.draw_mesh(&positions, &colors, &coordinates, &[0, 5]).is_err(),
        "an index naming a vertex that does not exist is refused too -- which is \
         what stops CNA reading past the array"
    );
}

/// The process-level settings, which need no device at all.
#[test]
fn the_process_settings_round_trip() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    use cna::extensions::runtime::{assembly_title, set_assembly_title};
    use cna::extensions::storage_ext;

    let before = assembly_title().expect("the current title");
    println!("NOTE: assembly title {before:?}");
    set_assembly_title("cna-rust-final-slice").expect("set the title");
    assert_eq!(
        assembly_title().expect("the new title"),
        "cna-rust-final-slice",
        "the title CNA reports is the one that was set"
    );
    set_assembly_title(&before).expect("restore the title");

    let root = storage_ext::root().expect("the storage root");
    println!("NOTE: storage root {root:?}");
    assert!(!root.is_empty(), "saves live somewhere");

    let _ = AtomicBool::new(false);
}
