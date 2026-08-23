#![allow(
    non_snake_case,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::too_many_arguments
)]

use core::mem::size_of;
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::value::{Rectangle, Vector2, Vector3};

use super::resource::{ResourceKind, ResourceState};
use super::{GraphicsResource, Texture2D};

struct FontProperties {
    line_spacing: i32,
    spacing: f32,
    default_character: Option<char>,
}

/// Native XNA `SpriteFont` backed by an owned font handle and retained atlas.
///
/// XNA exposes no public constructor. Instances are produced by the normal
/// `ContentManager.Load::<SpriteFont>` XNB reader path.
pub struct SpriteFont {
    state: Arc<ResourceState>,
    _texture: Arc<Texture2D>,
    characters: Vec<char>,
    _glyphs: Vec<Rectangle>,
    _cropping: Vec<Rectangle>,
    _kerning: Vec<Vector3>,
    properties: Mutex<FontProperties>,
}

#[allow(non_snake_case)]
impl SpriteFont {
    pub(crate) fn from_parts(
        texture: Arc<Texture2D>,
        glyphs: Vec<Rectangle>,
        cropping: Vec<Rectangle>,
        characters: Vec<char>,
        line_spacing: i32,
        spacing: f32,
        kerning: Vec<Vector3>,
        default_character: Option<char>,
    ) -> Result<Self> {
        let count = characters.len();
        if count == 0 || glyphs.len() != count || cropping.len() != count || kerning.len() != count
        {
            return Err(CnaError::InvalidInput(
                "SpriteFont glyph, crop, character and kerning tables must have the same nonzero length",
            ));
        }
        if !spacing.is_finite()
            || kerning
                .iter()
                .any(|value| !value.X.is_finite() || !value.Y.is_finite() || !value.Z.is_finite())
        {
            return Err(CnaError::InvalidInput(
                "SpriteFont spacing and kerning values must be finite",
            ));
        }
        if default_character.is_some_and(|value| !characters.contains(&value)) {
            return Err(CnaError::InvalidInput(
                "SpriteFont default character is absent from the character map",
            ));
        }

        let native_characters = characters
            .iter()
            .copied()
            .map(char16)
            .collect::<Result<Vec<_>>>()?;
        let native_default = default_character.map(char16).transpose()?;
        let native_glyphs = glyphs
            .iter()
            .zip(&cropping)
            .zip(native_characters.iter().copied().zip(&kerning))
            .map(
                |((glyph, crop), (character, kern))| sys::CNA_SpriteFontGlyph {
                    struct_size: size_of::<sys::CNA_SpriteFontGlyph>() as u32,
                    struct_version: 1,
                    glyph_bounds: native_rectangle(*glyph),
                    cropping: native_rectangle(*crop),
                    character,
                    reserved: 0,
                    kerning: sys::CNA_Vector3 {
                        x: kern.X,
                        y: kern.Y,
                        z: kern.Z,
                    },
                },
            )
            .collect::<Vec<_>>();
        let device = texture.GraphicsDevice().ok_or(CnaError::InvalidInput(
            "SpriteFont atlas has no graphics device",
        ))?;
        let info = sys::CNA_SpriteFontCreateInfo {
            struct_size: size_of::<sys::CNA_SpriteFontCreateInfo>() as u32,
            struct_version: 1,
            texture: texture.handle()?,
            glyphs: native_glyphs.as_ptr(),
            glyph_count: u64::try_from(native_glyphs.len())
                .map_err(|_| CnaError::InvalidInput("SpriteFont glyph table is too large"))?,
            line_spacing,
            spacing,
            default_character: native_default.unwrap_or(0),
            has_default_character: if native_default.is_some() {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            },
            reserved: [0; 5],
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        device
            .state
            .native()
            .create_sprite_font(&info, &mut handle)?;
        let verified = verify_native_font(device, handle, &native_glyphs, &native_characters);
        if let Err(error) = verified {
            let _ = device.state.native().destroy_sprite_font(handle);
            return Err(error);
        }

        Ok(Self {
            state: ResourceState::new(device, handle, ResourceKind::SpriteFont),
            _texture: texture,
            characters,
            _glyphs: glyphs,
            _cropping: cropping,
            _kerning: kerning,
            properties: Mutex::new(FontProperties {
                line_spacing,
                spacing,
                default_character,
            }),
        })
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }

    pub(crate) fn is_same_device(&self, device: &super::GraphicsDevice) -> bool {
        self.state.device().is_same_device(device)
    }

    pub fn MeasureString(&self, text: &str) -> Result<Vector2> {
        self.measure(text)
    }

    pub fn MeasureStringWithText(&self, text: &str) -> Result<Vector2> {
        self.measure(text)
    }

    fn measure(&self, text: &str) -> Result<Vector2> {
        let view = sys::CNA_StringView {
            data: text.as_ptr().cast(),
            byte_length: u64::try_from(text.len())
                .map_err(|_| CnaError::InvalidInput("SpriteFont text is too large"))?,
        };
        let mut size = sys::CNA_Vector2::default();
        self.state.device().state.native().measure_sprite_font(
            self.state.require_handle()?,
            view,
            &mut size,
        )?;
        Ok(Vector2::from_x_and_y(size.x, size.y))
    }

    #[must_use]
    pub fn LineSpacing(&self) -> i32 {
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .line_spacing
    }

    pub fn SetLineSpacing(&self, value: i32) -> Result<()> {
        self.state
            .device()
            .state
            .native()
            .set_sprite_font_line_spacing(self.state.require_handle()?, value)?;
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .line_spacing = value;
        Ok(())
    }

    #[must_use]
    pub fn Spacing(&self) -> f32 {
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .spacing
    }

    pub fn SetSpacing(&self, value: f32) -> Result<()> {
        if !value.is_finite() {
            return Err(CnaError::InvalidInput("SpriteFont spacing must be finite"));
        }
        self.state
            .device()
            .state
            .native()
            .set_sprite_font_spacing(self.state.require_handle()?, value)?;
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .spacing = value;
        Ok(())
    }

    #[must_use]
    pub fn DefaultCharacter(&self) -> Option<char> {
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .default_character
    }

    pub fn SetDefaultCharacter(&self, value: Option<char>) -> Result<()> {
        if value.is_some_and(|character| !self.characters.contains(&character)) {
            return Err(CnaError::InvalidInput(
                "SpriteFont default character is absent from the character map",
            ));
        }
        let native = value.map(char16).transpose()?;
        self.state
            .device()
            .state
            .native()
            .set_sprite_font_default_character(self.state.require_handle()?, native)?;
        self.properties
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .default_character = value;
        Ok(())
    }

    #[must_use]
    pub fn Characters(&self) -> &[char] {
        &self.characters
    }
}

fn verify_native_font(
    device: &super::GraphicsDevice,
    handle: sys::CNA_Handle,
    glyphs: &[sys::CNA_SpriteFontGlyph],
    characters: &[sys::CNA_Char16],
) -> Result<()> {
    let mut info = sys::CNA_SpriteFontInfo {
        struct_size: size_of::<sys::CNA_SpriteFontInfo>() as u32,
        struct_version: 1,
        ..sys::CNA_SpriteFontInfo::default()
    };
    device.state.native().sprite_font_info(handle, &mut info)?;
    if usize::try_from(info.character_count).ok() != Some(characters.len()) {
        return Err(CnaError::InvalidInput(
            "native SpriteFont character count disagrees with its source table",
        ));
    }
    let mut copied_characters = vec![0; characters.len()];
    let mut copied_count = 0;
    device.state.native().copy_sprite_font_characters(
        handle,
        &mut copied_characters,
        &mut copied_count,
    )?;
    if copied_count != info.character_count || copied_characters != characters {
        return Err(CnaError::InvalidInput(
            "native SpriteFont character table did not round-trip",
        ));
    }
    let mut copied_glyphs = vec![sys::CNA_SpriteFontGlyph::default(); glyphs.len()];
    device
        .state
        .native()
        .copy_sprite_font_glyphs(handle, &mut copied_glyphs, &mut copied_count)?;
    if copied_count != info.character_count || copied_glyphs != glyphs {
        return Err(CnaError::InvalidInput(
            "native SpriteFont glyph table did not round-trip",
        ));
    }
    Ok(())
}

fn char16(value: char) -> Result<sys::CNA_Char16> {
    u16::try_from(u32::from(value)).map_err(|_| {
        CnaError::InvalidInput("SpriteFont characters must fit XNA's UTF-16 char representation")
    })
}

const fn native_rectangle(value: Rectangle) -> sys::CNA_Rectangle {
    sys::CNA_Rectangle {
        x: value.X,
        y: value.Y,
        width: value.Width,
        height: value.Height,
    }
}

impl ContentDisposable for SpriteFont {
    fn DisposeContent(&self) -> Result<()> {
        self.state.dispose_native()
    }
}

impl ContentLoadable for SpriteFont {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}

impl Drop for SpriteFont {
    fn drop(&mut self) {
        let _ = self.state.dispose_native();
    }
}
