//! User-facing context used in [`crate::PixelGame::update`] and [`crate::PixelGame::render`].

pub mod sprite;
pub mod text;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use assets_manager::Compound;
use glamour::{Angle, Rect, Size2, Vector2};
use hashbrown::HashMap;
use smol_str::SmolStr;
use winit::{event::MouseButton, keyboard::KeyCode, window::Window};
use winit_input_helper::WinitInputHelper;

use crate::{
    assets::{AssetRef, Assets},
    font::Font,
    graphics::instance::Instances,
    sprite::Sprite,
    GameConfig,
};

use self::{sprite::DrawSpriteContext, text::DrawTextContext};

/// Context containing most functionality for interfacing with the game engine.
///
/// Exposed in [`crate::PixelGame::update`] and [`crate::PixelGame::render`].
///
/// [`Context`] is safe and cheap to clone due to being a `Rc<RefCell<..>>` under the hood.
#[derive(Clone)]
pub struct Context {
    /// Implementation of all non-primitive parts.
    inner: Rc<RefCell<ContextInner>>,
}

/// Render methods.
///
/// All methods use a `path` as the first argument, which is then used to retrieve the assets when they haven't been loaded before with [`crate::assets`].
impl Context {
    /// Handle sprite assets, mostly used for drawing.
    ///
    /// This will load the sprite asset from disk and upload it to the GPU the first time this sprite is referenced.
    /// Check the [`DrawSpriteContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the sprite, see [`crate::assets`] for more information about asset loading and storing.
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to specify the transformations of the sprite.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline(always)]
    pub fn sprite<'path>(&self, path: &'path str) -> DrawSpriteContext<'path, '_> {
        DrawSpriteContext {
            path,
            ctx: self,
            position: Vector2::ZERO,
            rotation: Angle::new(0.0),
        }
    }

    /// Draw some text on the screen at the set position with a rotation of `0`.
    ///
    /// This will load the font asset from disk and upload it to the GPU the first time this font is referenced.
    /// Check the [`DrawTextContext`] documentation for drawing options available.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path of the font, see [`crate::assets`] for more information about asset loading and storing.
    /// * `text` - ASCII text that will be drawn character by character.
    ///
    /// # Panics
    ///
    /// - When asset failed loading.
    #[inline]
    pub fn text<'path, 'text>(
        &self,
        path: &'path str,
        text: &'text str,
    ) -> DrawTextContext<'path, 'text, '_> {
        DrawTextContext {
            path,
            text,
            ctx: self,
            position: Vector2::ZERO,
        }
    }
}

/// State methods.
impl Context {
    /// Tell the game to exit, this will close the window and return from the [`crate::PixelGame::run`] function.
    ///
    /// The rest of the tick function will still be executed.
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{Context, KeyCode};
    ///
    /// # struct Empty; impl Empty {
    /// // In `PixelGame::update` trait implementation
    /// // ..
    /// fn update(&mut self, ctx: Context) {
    ///   // Stop game when 'Escape' is pressed
    ///   if ctx.key_pressed(KeyCode::Escape) {
    ///     ctx.exit();
    ///   }
    /// }
    /// # }
    #[inline]
    pub fn exit(&self) {
        self.write(|ctx| ctx.exit = true)
    }

    /// Get the delta time in seconds for the update tick.
    ///
    /// This is a constant set by [`GameConfig::with_update_delta_time`].
    ///
    /// # Returns
    ///
    /// - Seconds a single update tick took, this is a constant.
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.read(|ctx| ctx.delta_time)
    }

    /// Get the amount of frames drawn in a second.
    ///
    /// This counts the times [`crate::PixelGame::render`] is called.
    ///
    /// # Returns
    ///
    /// - Frames per second drawn.
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{Context, KeyCode, glamour::Vector2};
    ///
    /// # struct Empty; impl Empty {
    /// // In `PixelGame::render` trait implementation
    /// // ..
    /// fn render(&mut self, ctx: Context) {
    ///   // Draw a simple FPS counter on the top-left of the screen
    ///   let fps = ctx.delta_time().recip();
    ///   ctx.text("Beachball", &format!("{:.1}", ctx.frames_per_second())).draw();
    /// }
    /// # }
    #[inline]
    pub fn frames_per_second(&self) -> f32 {
        self.read(|ctx| ctx.frames_per_second)
    }

    /// Get the blending factor between the update states used in the render state.
    ///
    /// This is only set for [`crate::PixelGame::render`].
    ///
    /// Using this number allows you to create smooth animations for slower update loops.
    /// A common way to do this is to keep a previous state and interpolate the current state with the previous one.
    /// For most use cases a basic lerp function suffices for this.
    ///
    /// # Returns
    ///
    /// - Number between `0.0`-`1.0` which is the ratio between the previous state and the current state for interpolating.
    ///
    /// # Example
    ///
    /// ```
    /// use pixel_game_lib::{Context, KeyCode, glamour::Vector2};
    ///
    /// # #[derive(Default)] struct S{position: Vector2, previous_position: Vector2}
    /// # struct Empty; impl Empty {
    /// // In `PixelGame::render` trait implementation
    /// // ..
    /// fn render(&mut self, ctx: Context) {
    /// # let sprite = S::default();
    ///   // Lerp a sprite between it's last position and the current position
    ///   let interpolated_position =
    ///       sprite.position * ctx.blending_factor() +
    ///       sprite.previous_position * (1.0 - ctx.blending_factor());
    ///
    ///   // Draw the sprite with smooth position
    ///   ctx.sprite("sprite").translate(interpolated_position).draw();
    /// }
    /// # }
    #[inline]
    pub fn blending_factor(&self) -> f32 {
        self.read(|ctx| ctx.blending_factor)
    }

    /// Size of the drawable part of the window in pixels.
    ///
    /// This ignores any scaling.
    ///
    /// # Returns
    ///
    /// - Width and height of the drawable part of the window.
    #[inline]
    pub fn size(&self) -> Size2 {
        self.read(|ctx| ctx.size)
    }

    /// Show the OS cursor or hide it.
    ///
    /// # Arguments
    ///
    /// * `visible` - `true` to show the OS cursor, `false` to hide it.
    #[inline]
    pub fn set_cursor_visible(&self, visible: bool) {
        self.write(|ctx| ctx.window.set_cursor_visible(visible));
    }
}

/// Input methods.
impl Context {
    /// Get the position if the mouse is inside the viewport frame.
    ///
    /// This is `Some(..`) if the mouse is inside the viewport frame, not the entire window.
    /// The value of the coordinates corresponds to the pixel, when the frame is scaled this also encodes the subpixel in the fractional part.
    ///
    /// # Returns
    ///
    /// - `None` when the mouse is not on the buffer of pixels.
    /// - `Some(..)` with the coordinates of the pixel if the mouse is on the buffer of pixels.
    #[inline]
    pub fn mouse(&self) -> Option<Vector2> {
        self.read(|ctx| ctx.mouse)
    }

    /// Whether the mouse button goes from "not pressed" to "pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is pressed.
    #[inline]
    pub fn mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_pressed(mouse_button))
    }

    /// Whether the mouse button goes from "pressed" to "not pressed".
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is released.
    #[inline]
    pub fn mouse_released(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_released(mouse_button))
    }

    /// Whether the mouse button is in a "pressed" state.
    ///
    /// # Arguments
    ///
    /// * `mouse_button` - Mouse button to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the mouse is being held down.
    #[inline]
    pub fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        self.read(|ctx| ctx.input.mouse_held(mouse_button))
    }

    /// Whether the key goes from "not pressed" to "pressed".
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is pressed.
    #[inline]
    pub fn key_pressed(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_pressed(keycode))
    }

    /// Whether the key goes from "pressed" to "not pressed".
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is released.
    #[inline]
    pub fn key_released(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_released(keycode))
    }

    /// Whether the key is in a "pressed" state.
    ///
    /// Uses physical keys in the US layout, so for example the W key will be in the same physical key on both US and french keyboards.
    ///
    /// # Arguments
    ///
    /// * `keycode` - Key to check the state of.
    ///
    /// # Returns
    ///
    /// - `true` when the specified key is being held.
    #[inline]
    pub fn key_held(&self, keycode: KeyCode) -> bool {
        self.read(|ctx| ctx.input.key_held(keycode))
    }
}

/// Generic asset loading.
impl Context {
    /// Load a reference to any non-renderable asset.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset<T>(&self, path: impl AsRef<str>) -> AssetRef<T>
    where
        T: Compound,
    {
        self.read(|ctx| ctx.asset(path.as_ref()))
    }

    /// Load a clone of any non-renderable asset.
    ///
    /// Sets up the asset manager once, which can be accessed with the global function in this module.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory structure of the asset file in `assets/` where every `/` is a `.`.
    ///
    /// # Panics
    ///
    /// - When asset with path does not exist.
    /// - When asset could not be loaded due to an invalid format.
    #[inline]
    pub fn asset_owned<T>(&self, path: impl AsRef<str>) -> T
    where
        T: Compound,
    {
        self.read(|ctx| ctx.asset_owned(path.as_ref()))
    }
}

/// Internally used methods.
impl Context {
    /// Create a new empty context.
    #[inline]
    pub(crate) fn new(config: &GameConfig, window: Arc<Window>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner::new(config, window))),
        }
    }

    /// Get a read-only reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn read<R>(&self, reader: impl FnOnce(&ContextInner) -> R) -> R {
        reader(&self.inner.borrow())
    }

    /// Get a mutable reference to the inner struct.
    ///
    /// # Panics
    ///
    /// - When internal [`RwLock`] is poisoned.
    #[inline]
    pub(crate) fn write<R>(&self, writer: impl FnOnce(&mut ContextInner) -> R) -> R {
        writer(&mut self.inner.borrow_mut())
    }
}

/// Internal wrapped implementation for [`Context`].
pub(crate) struct ContextInner {
    /// Whether to exit after the update loop.
    pub(crate) exit: bool,
    /// Mouse position.
    pub(crate) mouse: Option<Vector2>,
    /// Parsed game input.
    ///
    /// Exoses methods for detecting mouse and keyboard events.
    pub(crate) input: WinitInputHelper,
    /// Instances of all sprites drawn this tick, also includes sprites from the fonts.
    pub(crate) instances: Instances,
    /// Asset cache.
    pub(crate) assets: Assets,
    /// All sprite textures to render.
    pub(crate) sprites: HashMap<SmolStr, Sprite>,
    /// All font textures to render.
    pub(crate) fonts: HashMap<SmolStr, Font>,
    /// Portions of textures that need to be re-written.
    pub(crate) texture_update_queue: Vec<(SmolStr, Rect, Vec<u32>)>,
    /// Time in seconds between every update tick.
    pub(crate) delta_time: f32,
    /// Frames per second for the render tick.
    pub(crate) frames_per_second: f32,
    /// Interpolation alpha for the render tick.
    pub(crate) blending_factor: f32,
    /// Size of the inner window in pixels.
    pub(crate) size: Size2,
    /// Reference to the window.
    pub(crate) window: Arc<Window>,
}

impl ContextInner {
    /// Initialize the inner context.
    pub(crate) fn new(config: &GameConfig, window: Arc<Window>) -> Self {
        let exit = false;
        let mouse = None;
        let input = WinitInputHelper::default();
        let instances = Instances::default();
        let assets = Assets::new();
        let sprites = HashMap::new();
        let fonts = HashMap::new();
        let texture_update_queue = Vec::new();
        let delta_time = config.update_delta_time;
        let size = config.buffer_size;
        let frames_per_second = 0.0;
        let blending_factor = 0.0;

        Self {
            exit,
            mouse,
            input,
            instances,
            assets,
            sprites,
            fonts,
            texture_update_queue,
            delta_time,
            frames_per_second,
            blending_factor,
            size,
            window,
        }
    }

    /// Get all sprites from any container with sprites.
    pub(crate) fn sprites_iter_mut(&mut self) -> impl Iterator<Item = &mut Sprite> {
        profiling::scope!("Sprite iterator");
        // PERF: improve performance by removing chain

        self.sprites.values_mut().chain(
            self.fonts
                .values_mut()
                .flat_map(|font| font.sprites.iter_mut()),
        )
    }

    /// Take all updates to textures that need to be done.
    pub(crate) fn take_texture_updates(
        &mut self,
    ) -> impl Iterator<Item = (&'_ Sprite, Rect, Vec<u32>)> + '_ {
        self.texture_update_queue
            .drain(..)
            .map(|(path, rect, pixels)| {
                (
                    self.sprites
                        .get(path.as_str())
                        .expect("Sprite update did't yield proper sprite path in 'sprites'"),
                    rect,
                    pixels,
                )
            })
    }

    /// Load the sprite asset if it doesn't exist yet.
    pub(crate) fn load_sprite_if_not_loaded(&mut self, path: &str) {
        if !self.sprites.contains_key(path) {
            profiling::scope!("Load sprite");

            // Load the sprite from disk
            let sprite = self.asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.sprites.insert(path.into(), sprite);
        }
    }

    /// Load the font asset if it doesn't exist yet.
    pub(crate) fn load_font_if_not_loaded(&mut self, path: &str) {
        if !self.fonts.contains_key(path) {
            profiling::scope!("Load font");

            // Load the font from disk
            let font = self.asset_owned(path);

            // Keep track of it, to see if it needs to be updated or not
            self.fonts.insert(path.into(), font);
        }
    }

    /// Load an asset.
    pub(crate) fn asset<T>(&self, path: &str) -> AssetRef<T>
    where
        T: Compound,
    {
        profiling::scope!("Load asset");

        self.assets.asset(path)
    }

    /// Load a clone of an asset.
    pub(crate) fn asset_owned<T>(&self, path: &str) -> T
    where
        T: Compound,
    {
        profiling::scope!("Load owned asset");

        self.assets.asset_owned(path)
    }
}
