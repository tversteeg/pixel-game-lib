//! Spawn a winit window and run the game loop.

#[cfg(feature = "in-game-profiler")]
pub(crate) mod in_game_profiler;
#[cfg(target_arch = "wasm32")]
mod web;

// Allow passing the profiler without having to change function signatures
#[cfg(feature = "in-game-profiler")]
pub(crate) use in_game_profiler::InGameProfiler;
use kira::manager::{AudioManager, AudioManagerSettings};
#[cfg(not(feature = "in-game-profiler"))]
pub(crate) type InGameProfiler = ();

use std::{marker::PhantomData, sync::Arc};

use glamour::Vector2;
use miette::{IntoDiagnostic, Result, WrapErr};
use web_time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    assets::{AssetSource, EmbeddedAssets},
    graphics::state::MainRenderState,
    Context, GameConfig,
};

/// How fast old FPS values decay in the smoothed average.
const FPS_SMOOTHED_AVERAGE_ALPHA: f32 = 0.8;

/// Update and render tick function signature.
pub(crate) trait TickFn<G>: FnMut(&mut G, Context) {}

impl<G, T: FnMut(&mut G, Context)> TickFn<G> for T {}

/// Window with game loop.
struct App<'window, U, R, G>
where
    U: TickFn<G>,
    R: TickFn<G>,
{
    /// Winit window handle.
    window: Option<Arc<Window>>,
    /// Game state.
    game_state: G,
    /// Game configuration.
    game_config: GameConfig,
    /// Game state passed around.
    ctx: Context,
    /// Game render state.
    render_state: MainRenderState<'window>,
    /// User update function.
    update: U,
    /// User render function.
    render: R,
    /// Time of the previous frame.
    last_time: Instant,
    /// Timestep accumulator for calculating how many times the `update` function should be called.
    accumulator: f32,
    /// In game profiler state.
    #[cfg(feature = "in-game-profiler")]
    in_game_profiler: InGameProfiler,
}

impl<'window, U, R, G> App<'window, U, R, G>
where
    U: TickFn<G>,
    R: TickFn<G>,
{
    /// Construct the app and initialize everything.
    pub(crate) async fn new(
        game_state: G,
        game_config: GameConfig,
        update: U,
        render: R,
        assets: EmbeddedAssets,
    ) -> Result<Self> {
        // Construct the asset source based on where it comes from
        // Needed to be called here because the render state will consume the atlas
        #[cfg(feature = "embed-assets")]
        let asset_source = AssetSource::new(
            assets.assets,
            assets.atlas.texture_id_to_atlas_id_map(),
            assets.atlas.texture_id_to_size_map(),
        );
        #[cfg(not(feature = "embed-assets"))]
        let asset_source = AssetSource::new(assets.0);

        // Create a surface on the window and setup the render state to it
        let mut render_state = MainRenderState::new(&game_config, assets.atlas())
            .await
            .wrap_err("Error setting up the rendering pipeline")?;

        // Start the audio
        let audio_manager = AudioManager::new(AudioManagerSettings::default())
            .into_diagnostic()
            .wrap_err("Error setting up audio manager")?;

        // Setup the context passed to the tick function implemented by the user
        let mut ctx = Context::new(&game_config, audio_manager, asset_source);

        // Setup the in-game profiler
        #[cfg(feature = "in-game-profiler")]
        let mut in_game_profiler = in_game_profiler::InGameProfiler::new(render_state.device());

        log::debug!("Opening window with game loop");

        // Current time
        let mut last_time = Instant::now();
        // Timestep accumulator
        let mut accumulator = 0.0;

        let window = None;

        Ok(Self {
            window,
            game_state,
            game_config,
            ctx,
            render_state,
            update,
            render,
            last_time,
            accumulator,
            #[cfg(feature = "in-game-profiler")]
            in_game_profiler,
        })
    }
}

impl<'window, U, R, G> ApplicationHandler for App<'window, U, R, G>
where
    U: TickFn<G>,
    R: TickFn<G>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Configure the window
        let window_attributes = Window::default_attributes()
            .with_title(&self.game_config.title)
            // Apply scaling for the requested size
            .with_inner_size(LogicalSize::new(
                self.game_config.buffer_size.width * self.game_config.scaling,
                self.game_config.buffer_size.height * self.game_config.scaling,
            ))
            // Don't allow the game to be smaller than the pixel size
            .with_min_inner_size(LogicalSize::new(
                self.game_config.buffer_size.width,
                self.game_config.buffer_size.height,
            ));

        // Apply web specific window settings
        #[cfg(target_arch = "wasm32")]
        let window_attributes =
            web::window_attributes(window_attributes).expect("Error setting up Web platform");

        // Create the window
        self.window = Some(Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Error inserting window into page"),
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window else {
            // Window is not configured yet
            return;
        };

        if window.id() != window_id {
            // Some other window is accessed
            return;
        }

        // Update egui inside in-game-profiler
        #[cfg(feature = "in-game-profiler")]
        self.in_game_profiler.handle_window_event(&window, &event);

        // Redraw the window when requested
        match event {
            // Resize render surface if window is resized on the desktop, on the web the size is always the same
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(new_size) => {
                // Resize GPU surface
                self.render_state
                    .resize(glamour::Size2::new(new_size.width, new_size.height));

                // On MacOS the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            // Render the frame
            WindowEvent::RedrawRequested => {
                // Set the updated state for the context
                self.ctx.write(|ctx| {
                    // Set the mouse position
                    ctx.mouse = ctx
                        .input
                        .cursor()
                        .and_then(|(x, y)| self.render_state.map_coordinate(Vector2::new(x, y)));
                });

                // Update the timestep
                let current_time = Instant::now();
                let frame_time = (current_time - self.last_time).as_secs_f32();
                self.last_time = current_time;

                self.accumulator += frame_time
                    // Ensure the frametime will never surpass this amount
                    .min(self.game_config.max_frame_time_secs);

                // Call the update tick function with the context
                while self.accumulator >= self.game_config.update_delta_time {
                    let should_exit = self.ctx.write(|ctx| {
                        // Handle the accumulated window events
                        todo!();

                        // Exit when the window is destroyed or closed
                        let should_exit =
                            ctx.input.close_requested() || ctx.input.destroyed() || ctx.exit;

                        if should_exit {
                            event_loop.exit();
                        }

                        should_exit
                    });

                    if should_exit {
                        // Exit was requested
                        return;
                    }

                    profiling::scope!("Update");

                    // Profile the allocations
                    #[cfg(feature = "in-game-profiler")]
                    let profile_region = InGameProfiler::start_profile_heap();

                    // Call the implemented update function on the 'PixelGame' trait
                    (self.update)(&mut self.game_state, self.ctx.clone());

                    #[cfg(feature = "in-game-profiler")]
                    self.in_game_profiler
                        .finish_profile_heap("Update", profile_region);

                    // Mark this tick as executed
                    self.accumulator -= self.game_config.update_delta_time;
                }

                // Set the blending factor for the render loop
                self.ctx.write(|ctx| {
                    // Set the blending factor
                    ctx.blending_factor = self.accumulator / self.game_config.update_delta_time;

                    // Set the FPS with a smoothed average function
                    ctx.frames_per_second = FPS_SMOOTHED_AVERAGE_ALPHA * ctx.frames_per_second
                        + (1.0 - FPS_SMOOTHED_AVERAGE_ALPHA) * frame_time.recip();

                    // Reset the renderable instances
                    ctx.instances.clear();
                });

                // Call the render tick function with the context
                {
                    profiling::scope!("Render");

                    // Profile the allocations
                    #[cfg(feature = "in-game-profiler")]
                    let profile_region = InGameProfiler::start_profile_heap();

                    // Call the implemented render function on the 'PixelGame' trait
                    (self.render)(&mut self.game_state, self.ctx.clone());

                    #[cfg(feature = "in-game-profiler")]
                    self.in_game_profiler
                        .finish_profile_heap("Render", profile_region);
                }

                // Render the saved render state
                {
                    profiling::scope!("Render Internal");

                    // Upload assets to the GPU
                    self.ctx.write(|ctx| {
                        self.render_state
                            .upload(ctx.assets.take_images_for_uploading());
                    });

                    // Render everything
                    #[cfg(feature = "in-game-profiler")]
                    self.render_state
                        .render(&mut self.ctx, &mut self.in_game_profiler, &window);
                    #[cfg(not(feature = "in-game-profiler"))]
                    self.render_state.render(&mut self.ctx, &mut (), &window);
                }

                // Tell the profiler we've executed a tick
                profiling::finish_frame!();
            }
            _ => (),
        }
    }
}

/// Manually create a new window with an event loop and run the game.
pub(crate) fn window<G, U, R>(
    game_state: G,
    game_config: GameConfig,
    update: U,
    render: R,
    assets: EmbeddedAssets,
) -> Result<()>
where
    G: 'static,
    U: TickFn<G> + 'static,
    R: TickFn<G> + 'static,
{
    // Setup the main event loop
    let event_loop = EventLoop::new()
        .into_diagnostic()
        .wrap_err("Error setting up event loop")?;

    // Create the future but run it inside the platform specific async context later
    let app_future = App::new(game_state, game_config, update, render, assets);

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Enable environment logger for winit
        env_logger::init();

        pollster::block_on(async {
            let mut app = app_future.await?;

            event_loop.run_app(&mut app);

            Ok(())
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Show logs
        console_log::init_with_level(log::Level::Warn).expect("Error setting up logger");

        // Show panics in the browser console log
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Web window function is async, so we need to spawn it into a local async runtime
        wasm_bindgen_futures::spawn_local(async {
            let mut app = app_future.await.expect("Error setting up game");

            event_loop.run_app(&mut app);
        });

        Ok(())
    }
}
