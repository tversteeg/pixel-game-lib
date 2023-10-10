use std::any::Any;

use crate::sprite::Sprite;

use super::Widget;

use blit::BlitOptions;
use taffy::prelude::{Layout, Node};
use vek::{Extent2, Rect, Vec2};
use winit_input_helper::WinitInputHelper;

/// A simple button widget.
#[derive(Debug)]
pub struct Button {
    /// Top-left position of the widget in pixels.
    pub offset: Vec2<f64>,
    /// Size of the button in pixels.
    pub size: Extent2<f64>,
    /// Extra size of the click region in pixels.
    ///
    /// Relative to the offset.
    pub click_region: Option<Rect<f64, f64>>,
    /// A custom label with text centered at the button.
    pub label: Option<String>,
    /// Current button state.
    pub state: State,
    /// Taffy layout node.
    pub node: Node,
}

impl Button {
    /// Handle the input.
    ///
    /// Return when the button is released.
    pub fn update(&mut self, input: &WinitInputHelper, mouse_pos: Option<Vec2<usize>>) -> bool {
        let mut rect = Rect::new(self.offset.x, self.offset.y, self.size.w, self.size.h);
        if let Some(mut click_region) = self.click_region {
            click_region.x += self.offset.x;
            click_region.y += self.offset.y;
            rect = rect.union(click_region);
        }

        match self.state {
            State::Normal => {
                if let Some(mouse_pos) = mouse_pos {
                    if !input.mouse_held(0) && rect.contains_point(mouse_pos.as_()) {
                        self.state = State::Hover;
                    }
                }

                false
            }
            State::Hover => {
                if let Some(mouse_pos) = mouse_pos {
                    if !rect.contains_point(mouse_pos.as_()) {
                        self.state = State::Normal;
                    } else if input.mouse_pressed(0) {
                        self.state = State::Down;
                    }
                }

                false
            }
            State::Down => {
                if input.mouse_released(0) {
                    self.state = State::Normal;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Render the slider.
    pub fn render(&self, canvas: &mut [u32], canvas_size: Extent2<f64>) {
        /*
        let button = crate::asset::<Sprite, _>(match self.state {
            State::Normal => "button-normal",
            State::Hover => "button-hover",
            State::Down => "button-down",
        });
        button.render_options(
            canvas,
            &BlitOptions::new_position(self.offset.x, self.offset.y)
                .with_slice9((2, 2, 1, 2))
                .with_area((self.size.w, self.size.h)),
        );

        if let Some(label) = &self.label {
            crate::font().render_centered(
                label,
                self.offset + (self.size.w / 2.0, self.size.h / 2.0),
                canvas,
            );
        }
        */
    }

    /// Update from layout changes.
    pub fn update_layout(&mut self, location: Vec2<f64>, layout: &Layout) {}
}

impl Widget for Button {
    fn update_layout(&mut self, location: Vec2<f64>, size: Extent2<f64>) {
        self.offset = location;
        self.size = size;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self {
            offset: Vec2::zero(),
            size: Extent2::zero(),
            label: None,
            state: State::default(),
            click_region: None,
            node: Node::default(),
        }
    }
}

/// In which state the button can be.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Button is doing nothing.
    #[default]
    Normal,
    /// Button is hovered over by the mouse.
    Hover,
    /// Button is hold down.
    Down,
}
