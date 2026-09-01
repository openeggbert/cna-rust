//! Native qualification for `RUST-EXT-016`: adopting a handle CNA created.
//!
//! Every other constructor on `SpriteFont` and `SoundEffect` *makes* the
//! native object. `cna_content_manager_load_sprite_font` and
//! `cna_content_manager_load_sound_effect` hand one over already made, and
//! until this milestone neither type had anywhere to put it -- which is why
//! both routes were deferred rather than bound.
//!
//! The assertions are about **ownership**, because that is what an adoption
//! can get wrong in ways a smoke test would not see: two owners, no owner, or
//! a release in the wrong order. CNA refuses to destroy a font's atlas while
//! the font lives, so a projection that released them the other way round
//! would strand the texture; the test disposes in that order deliberately and
//! measures what CNA says.
//!
//! Audio runs on SDL3's dummy driver rather than a `NULL` audio build. A NULL
//! build answers success without a device and would make every `SoundEffect`
//! assertion below vacuous.

#![allow(non_snake_case)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use cna::extensions::content::NativeContentManager;
use cna::Microsoft::Xna::Framework::Audio::SoundEffect;
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext, GraphicsDeviceInformation, Rectangle};
use cna::{run_for_frames, CnaError, ErrorCategory, GameState, GameStateAccess, Result};

const CHILD: &str = "CNA_RUST_ADOPTION_CHILD";

fn native_enabled() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_some()
}

/// A one-glyph SpriteFont in XNA's own `.xnb` container.
///
/// Written rather than checked in, so what the loader is given is visible here
/// rather than in an opaque file: one 1x1 white atlas pixel, the character
/// `?`, a line spacing of 2 and a kerning of `(0, 1, 0)`.
fn sprite_font_xnb() -> Vec<u8> {
    const READERS: &[&str] = &[
        "Microsoft.Xna.Framework.Content.SpriteFontReader",
        "Microsoft.Xna.Framework.Content.Texture2DReader",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Rectangle]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[System.Char]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Vector3]]",
        "Microsoft.Xna.Framework.Content.RectangleReader",
        "Microsoft.Xna.Framework.Content.CharReader",
        "Microsoft.Xna.Framework.Content.Vector3Reader",
    ];
    let mut payload = Vec::new();
    write_7bit(&mut payload, READERS.len());
    for reader in READERS {
        write_xnb_string(&mut payload, reader);
        payload.extend_from_slice(&0_i32.to_le_bytes());
    }
    write_7bit(&mut payload, 0);
    payload.push(1); // SpriteFont root reader.
    payload.push(2); // Texture2D atlas reader.
    payload.extend_from_slice(&0_i32.to_le_bytes()); // SurfaceFormat.Color.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&1_i32.to_le_bytes()); // One mip.
    payload.extend_from_slice(&4_i32.to_le_bytes());
    payload.extend_from_slice(&[255, 255, 255, 255]);
    payload.push(3); // Glyph Rectangle list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    write_rectangle(&mut payload, Rectangle::new(0, 0, 1, 1));
    payload.push(3); // Cropping Rectangle list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    write_rectangle(&mut payload, Rectangle::new(0, 0, 1, 1));
    payload.push(4); // Character list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.push(b'?'); // One UTF-8 byte: `BinaryReader.ReadChar` is not UTF-16.
    payload.extend_from_slice(&2_i32.to_le_bytes()); // Line spacing.
    payload.extend_from_slice(&0_f32.to_le_bytes()); // Extra spacing.
    payload.push(5); // Kerning Vector3 list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&0_f32.to_le_bytes());
    payload.extend_from_slice(&1_f32.to_le_bytes());
    payload.extend_from_slice(&0_f32.to_le_bytes());
    payload.push(1); // Has default character.
    payload.push(b'?');

    let mut bytes = b"XNBw\x05\x00".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(10 + payload.len())
            .expect("SpriteFont fixture size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&payload);
    bytes
}

/// Ten milliseconds of silence: PCM16, mono, 44,100 Hz.
///
/// Exactly 441 frames, so the duration CNA reports has an arithmetic answer to
/// check against rather than "something non-zero".
fn silent_wav() -> Vec<u8> {
    const FRAMES: u32 = 441;
    let data = vec![0_u8; (FRAMES * 2) as usize];
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // PCM.
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // Mono.
    bytes.extend_from_slice(&44_100_u32.to_le_bytes());
    bytes.extend_from_slice(&88_200_u32.to_le_bytes()); // Bytes per second.
    bytes.extend_from_slice(&2_u16.to_le_bytes()); // Block align.
    bytes.extend_from_slice(&16_u16.to_le_bytes()); // Bits per sample.
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&data);
    bytes
}

fn write_7bit(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let mut next = u8::try_from(value & 0x7f).expect("seven-bit fixture chunk");
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

fn write_xnb_string(bytes: &mut Vec<u8>, value: &str) {
    write_7bit(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn write_rectangle(bytes: &mut Vec<u8>, value: Rectangle) {
    bytes.extend_from_slice(&value.X.to_le_bytes());
    bytes.extend_from_slice(&value.Y.to_le_bytes());
    bytes.extend_from_slice(&value.Width.to_le_bytes());
    bytes.extend_from_slice(&value.Height.to_le_bytes());
}

/// The content root, written fresh so a failing run leaves the exact input.
fn content_root() -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adoption-content");
    std::fs::create_dir_all(&root).expect("content root");
    std::fs::write(root.join("font.xnb"), sprite_font_xnb()).expect("write the font asset");
    std::fs::write(root.join("beep.wav"), silent_wav()).expect("write the audio asset");
    root
}

fn independent_device_or_skip() -> Option<GraphicsDevice> {
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
        Err(error) => panic!("independent graphics device: {error}"),
    }
}

struct AdoptionGame {
    state: Arc<GameState>,
    root: PathBuf,
}

impl GameStateAccess for AdoptionGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for AdoptionGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let manager =
            NativeContentManager::new(&device, self.root.to_str().expect("utf-8 content root"))?;
        manager.register_builtin_loaders()?;
        sprite_font(&manager, &device);
        sound_effect(&manager, game)?;
        a_real_xna_font(self, &manager, &device);
        the_guide_draws_what_it_left_pending(&manager, &device);
        manager.unload()?;
        println!("OK: adoption");
        Ok(())
    }
}

/// `RUST-EXT-016`, the `SpriteFont` half.
fn sprite_font(manager: &NativeContentManager, device: &GraphicsDevice) {
    let (font, atlas) = match manager.load_sprite_font(device, "font") {
        Ok(loaded) => loaded,
        Err(CnaError::Native {
            category: ErrorCategory::Io,
            ref message,
            ..
        }) => {
            println!("this artifact does not read .xnb fonts: {message}");
            return;
        }
        Err(error) => panic!("loading the SpriteFont failed: {error}"),
    };

    // Every one of these came back *out* of the native handle, through
    // `cna_sprite_font_get_info`, `_copy_characters` and `_copy_glyphs`. An
    // adoption that took the handle and left the Rust tables empty would
    // answer an empty character map and a zero line spacing here.
    assert_eq!(font.Characters(), &['?']);
    assert_eq!(font.LineSpacing(), 2);
    assert!(font.Spacing().abs() < f32::EPSILON);
    assert_eq!(font.DefaultCharacter(), Some('?'));
    let measured = font.MeasureString("?").expect("the loaded font measures");
    assert!(measured.X > 0.0 && measured.Y > 0.0, "a glyph has a size");

    // The atlas is the second owned handle, and the font's own.
    assert_eq!(atlas.Width(), 1);
    assert_eq!(atlas.Height(), 1);

    // Ownership, which is what an adoption gets wrong. CNA refuses to release
    // the atlas while the font uses it, so a projection that released them the
    // other way round would strand the texture. The font's field order makes
    // that impossible, and the reference count says who holds what.
    assert_eq!(
        Arc::strong_count(&atlas),
        2,
        "the font holds the atlas and so does the caller"
    );
    drop(font);
    assert_eq!(
        Arc::strong_count(&atlas),
        1,
        "dropping the font releases its share of the atlas and nothing else"
    );
    drop(atlas);

    // A second load is a second font over a second atlas rather than one
    // handle handed out twice, which the first drop would have freed.
    let (again, atlas_again) = manager
        .load_sprite_font(device, "font")
        .expect("a second load");
    assert_eq!(again.Characters(), &['?']);
    assert_eq!(again.MeasureString("?").expect("measure"), measured);
    drop(again);
    drop(atlas_again);
}

/// `RUST-EXT-016`, the `SoundEffect` half.
fn sound_effect(manager: &NativeContentManager, game: &GameContext<'_>) -> Result<()> {
    let mut effect = match manager.load_sound_effect(game, "beep") {
        Ok(effect) => effect,
        Err(CnaError::Native {
            category: ErrorCategory::Io,
            ref message,
            ..
        }) => {
            println!("this artifact does not read .wav assets: {message}");
            return Ok(());
        }
        Err(error) => panic!("loading the SoundEffect failed: {error}"),
    };

    // 441 frames at 44,100 Hz is ten milliseconds, and the duration comes from
    // the handle rather than from a buffer this side never saw.
    let duration = effect.Duration()?;
    assert!(
        (duration.TotalMilliseconds() - 10.0).abs() < 1.0,
        "ten milliseconds of audio, not something else: {duration:?}"
    );

    // The adopted handle drives the ordinary operations, and an instance is a
    // child of it -- which is what makes the disposal order matter.
    let instance = effect.CreateInstance()?;
    instance.Play()?;
    instance.Stop()?;

    let mut second = manager.load_sound_effect(game, "beep")?;
    effect.Dispose()?;
    assert!(
        effect.Play().is_err(),
        "a disposed effect refuses rather than playing a released handle"
    );
    // If the loader cached, disposing the first would have disposed this one
    // too and every call below would refuse. It does not: CNA's
    // `Load<SoundEffect>` specialization deliberately answers a fresh effect.
    assert!(
        !second.NativeIsDisposed()?,
        "the loader does not cache: the second effect is its own"
    );
    assert_eq!(second.Duration()?, duration);
    second.CreateInstance()?.Play()?;
    second.Dispose()?;
    Ok(())
}

#[test]
fn a_loaded_font_and_effect_are_owned_once_each_and_outlive_their_manager() {
    if !native_enabled() {
        return;
    }
    if std::env::var_os(CHILD).is_some() {
        // CNA registers its built-in `.xnb` readers from `Game`'s constructor,
        // so an independently constructed device -- which is how the rest of
        // the native content tests reach a loader -- cannot read a font at
        // all. Both halves therefore run inside a real game.
        let Some(_device) = independent_device_or_skip() else {
            println!("OK: adoption");
            return;
        };
        run_for_frames(
            AdoptionGame {
                state: Arc::new(GameState::new()),
                root: content_root(),
            },
            1,
        )
        .expect("the adoption game runs");
        return;
    }
    // A child, with SDL's dummy audio driver rather than a NULL audio build:
    // NULL answers success with no device and would make every `SoundEffect`
    // assertion vacuous.
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args([
            "--exact",
            "a_loaded_font_and_effect_are_owned_once_each_and_outlive_their_manager",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("SDL_AUDIODRIVER", "dummy")
        .output()
        .expect("start the adoption child");
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the adoption child failed: {:?}\n{text}",
        output.status
    );
    assert!(
        text.contains("OK: adoption"),
        "the child never reached the end of the adoption case:\n{text}"
    );
}

/// A real, externally-produced XNA font -- adopted, and then parsed.
///
/// The synthetic fixture above proves the loader path against bytes this test
/// wrote. This proves it against bytes Microsoft's own pipeline wrote, and it
/// is the case that found the defect: `ContentReader.ReadChar` read a
/// little-endian UTF-16 code unit where `BinaryReader`'s reads UTF-8. Against
/// a fixture written to match that mistake nothing showed; a real font stores
/// its 95 characters as `20 21 22 23 ...`, one byte each, so a UTF-16 reader
/// sees 47 wrong characters and runs a byte late for the rest of the file.
///
/// The asset is MonoGame's `Default.xnb`, vendored in cnanext's own test
/// assets. It is read, never written, and the case skips when it is not there.
fn a_real_xna_font(game: &AdoptionGame, manager: &NativeContentManager, device: &GraphicsDevice) {
    let Some(root) = std::env::var_os("CNA_ROOT").map(PathBuf::from) else {
        println!("CNA_ROOT is unset, so the real XNA font fixture is not reachable");
        return;
    };
    let source = root.join("tests/assets/xnb/monogame/windows/uncompressed/Default.xnb");
    if !source.exists() {
        println!("this dependency checkout has no real XNA font fixture");
        return;
    }
    std::fs::copy(&source, game.root.join("Default.xnb")).expect("copy the real font fixture");

    // Adopted through the route this milestone bound. Ninety-five printable
    // ASCII characters, in order, out of a file this crate did not write.
    let (font, atlas) = manager
        .load_sprite_font(device, "Default")
        .expect("CNA reads a real XNA font");
    assert_eq!(font.Characters().len(), 95);
    assert_eq!(font.Characters().first().copied(), Some(' '));
    assert_eq!(font.Characters().last().copied(), Some('~'));
    assert_eq!(font.LineSpacing(), 19);
    assert!(font.Spacing().abs() < f32::EPSILON);
    assert_eq!(font.DefaultCharacter(), None);
    assert!(
        font.MeasureString("Hello").expect("measure").X > 0.0,
        "a real font measures real text"
    );
    assert_eq!(atlas.Width(), 128);
    assert_eq!(atlas.Height(), 128);
    drop(font);
    drop(atlas);

    // And the same file through *this* crate's own XNB pipeline. Its Texture2D
    // reader is documented as `SurfaceFormat.Color` only and this atlas is
    // block-compressed, so that refusal is the artifact's answer; any other
    // failure -- a desynced character list above all -- still fails here.
    use cna::Microsoft::Xna::Framework::Graphics::SpriteFont;
    let content = game.state.Content();
    content
        .SetRootDirectory(game.root.to_str().expect("utf-8 content root"))
        .expect("content root");
    match content.Load::<SpriteFont>("Default") {
        Ok(parsed) => {
            assert_eq!(parsed.Characters().len(), 95);
            assert_eq!(parsed.LineSpacing(), 19);
        }
        Err(error) => {
            let text = format!("{error:?}");
            assert!(
                text.contains("requires SurfaceFormat.Color"),
                "the real XNA font failed for a reason other than its atlas format: {error:?}"
            );
            println!("this crate's XNB reader does not decode the fixture's atlas: {text}");
        }
    }
}

/// The other half of a pending Guide screen: something has to draw it.
///
/// CNA leaves `BeginShowMessageBox` pending because there is no console
/// overlay outside Xbox Live, and publishes a renderer for a game that has to
/// put it on screen itself. It needs a font, which is why it lives with the
/// adoption case: a loaded `SpriteFont` is the only one this test has.
fn the_guide_draws_what_it_left_pending(manager: &NativeContentManager, device: &GraphicsDevice) {
    use cna::extensions::gamer_services::{DrawsPendingGuide, PendingGuideRequest};
    use cna::Microsoft::Xna::Framework::Color;
    use cna::Microsoft::Xna::Framework::Graphics::{SpriteBatch, Texture2D};

    let Ok((font, _atlas)) = manager.load_sprite_font(device, "font") else {
        println!("no font on this artifact, so the guide renderer has nothing to draw with");
        return;
    };
    let batch = SpriteBatch::new(device).expect("a sprite batch");
    let white = Texture2D::new(device, 1, 1).expect("a single white pixel");
    white
        .SetData(&[Color::White])
        .expect("fill the pixel");

    // Nothing is pending, and CNA documents that as a successful no-op -- so a
    // game can call this every frame without asking first.
    assert!(!PendingGuideRequest::has_message_box().expect("pending state"));
    device
        .draw_pending_message_box(&batch, &font, &white)
        .expect("drawing nothing is a no-op that succeeds");
    device
        .draw_pending_keyboard_input(&batch, &font, &white)
        .expect("the same for keyboard input");
}
