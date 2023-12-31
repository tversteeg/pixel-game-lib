//! Render a simple ASCII bitmap font.

use assets_manager::{loader::TomlLoader, AnyCache, Asset, BoxedError, Compound, SharedString};
use blit::{prelude::SubRect, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::canvas::Canvas;

/// Pixel font loaded from an image.
#[derive(Debug)]
pub struct Font {
    /// Image to render.
    pub(crate) sprite: BlitBuffer,
    /// Size of a single character.
    pub(crate) char_size: Extent2<u8>,
}

impl Font {
    /// Construct the font from a bitmap with an alpha channel.
    pub fn from_buffer_with_alpha<B>(buffer: B, alpha: u8, char_size: Extent2<u8>) -> Self
    where
        B: ToBlitBuffer,
    {
        let sprite = buffer.to_blit_buffer_with_alpha(alpha);

        Self { sprite, char_size }
    }

    /// Construct the font from a bitmap where a single color is the alpha mask.
    pub fn from_buffer_with_mask_color<B>(
        buffer: B,
        mask_color: u32,
        char_size: Extent2<u8>,
    ) -> Self
    where
        B: ToBlitBuffer,
    {
        let sprite = buffer.to_blit_buffer_with_mask_color(mask_color);

        Self { sprite, char_size }
    }

    /// Render ASCII text on a pixel buffer.
    ///
    /// Start from the top-left.
    pub fn render(&self, text: &str, position: Vec2<f64>, canvas: &mut Canvas) {
        // First character in the image
        let char_start = '!';
        let char_end = '~';

        let pos: Vec2<i32> = position.as_() - (self.char_size.w as i32, 0);
        let mut x = pos.x;
        let mut y = pos.y;

        // Draw each character from the string
        text.chars().for_each(|ch| {
            // Move the cursor
            x += self.char_size.w as i32;

            // Don't draw characters that are not in the picture
            if ch < char_start || ch > char_end {
                if ch == '\n' {
                    x = pos.x;
                    y += self.char_size.h as i32;
                } else if ch == '\t' {
                    x += self.char_size.w as i32 * 3;
                }
                return;
            }

            // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
            let char_offset = (ch as u8 - char_start as u8) as u32 * self.char_size.w as u32;

            // Draw the character
            self.sprite.blit(
                canvas.buffer,
                canvas.size.into_tuple().into(),
                &BlitOptions::new_position(x, y).with_sub_rect(SubRect::new(
                    char_offset,
                    0,
                    self.char_size.into_tuple(),
                )),
            );
        });
    }

    /// Render ASCII text on a pixel buffer.
    ///
    /// Center the text around the point.
    /// Currently does not support multi-line strings yet.
    pub fn render_centered(&self, text: &str, position: Vec2<f64>, canvas: &mut Canvas) {
        self.render(
            text,
            position
                - Vec2::new(
                    (text.len() as f64 * self.char_size.w as f64) / 2.0,
                    self.char_size.h as f64 / 2.0,
                ),
            canvas,
        )
    }
}

#[cfg(feature = "default-font")]
impl Default for Font {
    fn default() -> Self {
        let image = image::load_from_memory(include_bytes!("../assets/Beachball.png"))
            .expect("Error loading default font image from memory")
            .into_rgba8();

        let sprite = image.to_blit_buffer_with_alpha(127);
        let char_size = Extent2::new(10, 10);

        Self { sprite, char_size }
    }
}

impl Compound for Font {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache
            .load_owned::<crate::sprite::Sprite>(id)?
            .into_blit_buffer();

        // Load the metadata
        let metadata = cache.load::<FontMetadata>(id)?.read();
        let char_size = Extent2::new(metadata.char_width, metadata.char_height);

        Ok(Self { sprite, char_size })
    }
}

/// Font metadata to load from TOML.
#[derive(Deserialize)]
pub(crate) struct FontMetadata {
    /// Width of a single character.
    pub(crate) char_width: u8,
    /// Height of a single character.
    pub(crate) char_height: u8,
}

impl Asset for FontMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}
