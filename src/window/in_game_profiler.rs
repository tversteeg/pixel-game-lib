//! Show a in-game profiler window.
//!
//! Window is based on Egui.

use std::alloc::System;

use egui::{Grid, Window as EguiWindow};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{
    egui::{FullOutput, ViewportId},
    State,
};
use glamour::Size2;
use hashbrown::HashMap;
use puffin_egui::egui::Context;
use stats_alloc::{Region, Stats, StatsAlloc, INSTRUMENTED_SYSTEM};
use winit::{event::WindowEvent, window::Window};

use crate::graphics::state::PREFERRED_TEXTURE_FORMAT;

/// Use a custom allocator to count all allocations.
#[global_allocator]
pub(crate) static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// State for showing the in-game profiler.
pub(crate) struct InGameProfiler {
    /// Egui WGPU renderer.
    renderer: Renderer,
    /// Egui winit state.
    state: State,
    /// Memory allocations.
    allocations: HashMap<String, Stats>,
}

impl InGameProfiler {
    /// Creates a new render routine to render the in-game profiler.
    pub(super) fn new<W>(device: &wgpu::Device, window: W) -> Self
    where
        W: wgpu::WindowHandle,
    {
        let renderer = Renderer::new(device, PREFERRED_TEXTURE_FORMAT, None, 1);
        let state = State::new(
            Context::default(),
            ViewportId::default(),
            &window,
            None,
            None,
        );

        let allocations = HashMap::new();

        // Enable the profiler
        puffin::set_scopes_on(true);

        Self {
            renderer,
            state,
            allocations,
        }
    }

    /// Render the window.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: Size2<u32>,
    ) {
        profiling::scope!("Render profiling window");

        // Get egui input
        let input = self.state.take_egui_input(window);

        // Render egui frame
        let FullOutput {
            shapes,
            textures_delta,
            pixels_per_point,
            ..
        } = self.state.egui_ctx().run(input, |ctx| {
            // Show a GUI window for the CPU & GPU profilers
            EguiWindow::new("Profiler").show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Draw the memory profiler window
                    for (stat, values) in &self.allocations {
                        // Draw the allocations
                        ui.collapsing(format!("Allocations: {stat}"), |ui| {
                            Grid::new("grid")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("Allocations");
                                    ui.monospace(values.allocations.to_string());
                                    ui.end_row();

                                    ui.label("Deallocations");
                                    ui.monospace(values.deallocations.to_string());
                                    ui.end_row();

                                    ui.label("Bytes Allocated");
                                    ui.monospace(values.bytes_allocated.to_string());
                                    ui.end_row();

                                    ui.label("Bytes Deallocated");
                                    ui.monospace(values.bytes_deallocated.to_string());
                                    ui.end_row();

                                    ui.label("Bytes Reallocated");
                                    ui.monospace(values.bytes_reallocated.to_string());
                                    ui.end_row();
                                });
                        });
                    }
                });

                // CPU profiler
                puffin_egui::profiler_ui(ui);
            });
        });

        for id in textures_delta.free {
            self.renderer.free_texture(&id);
        }

        for (id, image_delta) in textures_delta.set {
            self.renderer
                .update_texture(device, queue, id, &image_delta);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [screen_size.width, screen_size.height],
            pixels_per_point,
        };

        let paint_jobs = self.state.egui_ctx().tessellate(shapes, pixels_per_point);

        self.renderer
            .update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        // Start a new render pass for the egui window
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render the egui window
        self.renderer
            .render(&mut render_pass, &paint_jobs, &screen_descriptor);
    }

    /// Handle a winit event.
    pub(super) fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    /// Start profiling heap allocations within a region.
    pub(crate) fn start_profile_heap() -> Region<'static, System> {
        Region::new(GLOBAL)
    }

    /// Start profiling heap allocations within a region.
    pub(crate) fn finish_profile_heap(
        &mut self,
        name: impl Into<String>,
        region: Region<'static, System>,
    ) {
        let stats = region.change();
        self.allocations.insert(name.into(), stats);
    }
}
