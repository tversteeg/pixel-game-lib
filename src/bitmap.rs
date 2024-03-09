//! 2D map of booleans.
//!
//! This is not an image.
//! Can be used for various effects such as masks for destructible terrain, pixel outlines etc.

use bitvec::vec::BitVec;
use vek::{Extent2, Vec2};

use crate::{
    canvas::Canvas,
    sprite::{Sprite, SpriteOffset},
};

/// 2D map of boolean values.
///
/// Not an image!
/// Every 'pixel' is a simple `true`/`false` value.
#[derive(Debug, Clone, PartialEq)]
pub struct BitMap {
    /// Amount of pixels in both dimensions.
    size: Extent2<usize>,
    /// All pixel values in a packed vector.
    map: BitVec,
}

impl BitMap {
    /// Create a new empty map.
    ///
    /// # Arguments
    ///
    /// * `size` - Amount of pixels in both dimensions.
    pub fn empty(size: Extent2<usize>) -> Self {
        let map = BitVec::repeat(false, size.product());

        Self { size, map }
    }

    /// Set a value at a coordinate.
    ///
    /// # Arguments
    ///
    /// * `position` - Coordinate inside the map to set, if outside nothing is done.
    /// * `value` - Boolean to set at the coordinate.
    pub fn set(&mut self, position: impl Into<Vec2<usize>>, value: bool) {
        let position = position.into();
        if position.x >= self.size.w || position.y >= self.size.h {
            return;
        }

        let index = position.x + position.y * self.size.w;
        self.set_at_index(index, value);
    }

    /// Convert the value to a image where every `true` value is replaced by the color.
    ///
    /// # Arguments
    ///
    /// * `color` - Draw every boolean `true` value as a colored pixel on the image.
    /// * `offset` - Pixel offset of where the sprite will be drawn.
    pub fn to_sprite(&self, color: u32, offset: SpriteOffset) -> Sprite {
        // Convert each binary value to a pixel
        let pixels = self
            .map
            .iter()
            .map(|bit| if *bit { color } else { 0 })
            .collect::<Vec<_>>();

        // Create a sprite from it
        Sprite::from_buffer(&pixels, self.size, offset)
    }

    /// Width of the map.
    pub fn width(&self) -> usize {
        self.size.w
    }

    /// Height of the map.
    pub fn height(&self) -> usize {
        self.size.h
    }

    /// Size of the map.
    pub fn size(&self) -> Extent2<usize> {
        self.size
    }

    /// Set a pixel at index of the map.
    fn set_at_index(&mut self, index: usize, value: bool) {
        self.map.set(index, value);
    }
}
