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

    /// Takes over a font handle CNA created, and the atlas it draws from.
    ///
    /// `from_parts` goes the other way: it takes Rust's tables and *builds* a
    /// native font from them. A font the content pipeline loaded already
    /// exists, so there is nothing to build -- and the tables this type keeps
    /// for `Characters`, `MeasureString` and the glyph accessors have to come
    /// back out of it. CNA publishes exactly what is needed:
    /// `cna_sprite_font_get_info` for the layout properties and the character
    /// count, `cna_sprite_font_copy_characters` for the map, and
    /// `cna_sprite_font_copy_glyphs` for the bounds, cropping and kerning.
    ///
    /// # Ownership
    ///
    /// Exactly one owner, and the order matters. This value owns the font
    /// handle; the `Arc<Texture2D>` owns the atlas. CNA refuses
    /// `cna_texture2d_destroy` while a font still uses the atlas, so the font
    /// has to go first -- which it does, because `state` is declared before
    /// `_texture` and Rust drops fields in declaration order. The same rule
    /// already governs a caller-built font, so a loaded one needs no special
    /// case.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the refusal for a font whose
    /// tables disagree with the count it reports.
    pub(crate) fn adopt(texture: Arc<Texture2D>, handle: sys::CNA_Handle) -> Result<Self> {
        let Some(device) = texture.GraphicsDevice().cloned() else {
            // Nothing else owns the handle yet, so this is the one place it
            // can be released without stranding it -- and with no device there
            // is nothing to release it *through*, which is the refusal.
            return Err(CnaError::InvalidInput(
                "SpriteFont atlas has no graphics device",
            ));
        };
        let adopted = Self::read_back(&device, texture, handle);
        if adopted.is_err() {
            let _ = device.state.native().destroy_sprite_font(handle);
        }
        adopted
    }

    fn read_back(
        device: &super::GraphicsDevice,
        texture: Arc<Texture2D>,
        handle: sys::CNA_Handle,
    ) -> Result<Self> {
        let mut info = sys::CNA_SpriteFontInfo {
            struct_size: size_of::<sys::CNA_SpriteFontInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_SpriteFontInfo::default()
        };
        device.state.native().sprite_font_info(handle, &mut info)?;
        let count = usize::try_from(info.character_count)
            .map_err(|_| CnaError::InvalidInput("native SpriteFont reports too many characters"))?;
        if count == 0 {
            return Err(CnaError::InvalidInput(
                "a SpriteFont with no characters cannot draw anything",
            ));
        }

        let mut native_characters = vec![0; count];
        let mut copied = 0;
        device.state.native().copy_sprite_font_characters(
            handle,
            &mut native_characters,
            &mut copied,
        )?;
        let mut native_glyphs = vec![sys::CNA_SpriteFontGlyph::default(); count];
        let mut glyph_count = 0;
        device
            .state
            .native()
            .copy_sprite_font_glyphs(handle, &mut native_glyphs, &mut glyph_count)?;
        if copied != info.character_count || glyph_count != info.character_count {
            return Err(CnaError::InvalidInput(
                "native SpriteFont tables disagree with the count it reports",
            ));
        }

        let characters = native_characters
            .iter()
            .map(|value| {
                char::from_u32(u32::from(*value)).ok_or(CnaError::InvalidInput(
                    "native SpriteFont character is not a Unicode scalar value",
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let default_character = if info.has_default_character == sys::CNA_FALSE {
            None
        } else {
            Some(
                char::from_u32(u32::from(info.default_character)).ok_or(CnaError::InvalidInput(
                    "native SpriteFont default character is not a Unicode scalar value",
                ))?,
            )
        };
        let glyphs = native_glyphs
            .iter()
            .map(|glyph| rust_rectangle(glyph.glyph_bounds))
            .collect();
        let cropping = native_glyphs
            .iter()
            .map(|glyph| rust_rectangle(glyph.cropping))
            .collect();
        let kerning = native_glyphs
            .iter()
            .map(|glyph| Vector3::from_x_and_y_and_z(glyph.kerning.x, glyph.kerning.y, glyph.kerning.z))
            .collect();

        Ok(Self {
            state: ResourceState::new(device, handle, ResourceKind::SpriteFont),
            _texture: texture,
            characters,
            _glyphs: glyphs,
            _cropping: cropping,
            _kerning: kerning,
            properties: Mutex::new(FontProperties {
                line_spacing: info.line_spacing,
                spacing: info.spacing,
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

const fn rust_rectangle(value: sys::CNA_Rectangle) -> Rectangle {
    Rectangle::new(value.x, value.y, value.width, value.height)
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


impl crate::extensions::graphics_resource::HasResourceState for SpriteFont {
    fn resource_state(&self) -> &super::resource::ResourceState {
        &self.state
    }
}
