//! Setting up a window for WASM platforms.

use miette::{Context, IntoDiagnostic, Result};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use winit::window::WindowAttributes;

/// Inject canvas.
#[inline(always)]
pub(crate) fn window_attributes(window_attributes: WindowAttributes) -> Result<WindowAttributes> {
    // Create a canvas the winit window can be attached to
    let window = web_sys::window().ok_or_else(|| miette::miette!("Error finding web window"))?;
    let document = window
        .document()
        .ok_or_else(|| miette::miette!("Error finding web document"))?;
    let body = document
        .body()
        .ok_or_else(|| miette::miette!("Error finding web body"))?;

    // Look for a canvas with ID 'chuot', and if not found create it
    let canvas = match document.get_element_by_id("chuot") {
        // Canvas found, use it
        Some(canvas) => canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|err| miette::miette!("Error casting canvas: {err:?}"))?,
        // No canvas found, create the element
        None => {
            log::warn!("No canvas element with ID 'chuot' found, creating a new one and adding it to the body of the page");
            let canvas = document
                .create_element("canvas")
                .map_err(|err| miette::miette!("Error creating canvas: {err:?}"))?
                .dyn_into::<HtmlCanvasElement>()
                .map_err(|err| miette::miette!("Error casting canvas: {err:?}"))?;
            canvas.set_id("chuot");

            body.append_child(&canvas)
                .map_err(|err| miette::miette!("Error appending canvas to body: {err:?}"))?;

            canvas
        }
    };

    // Ensure the pixels are not rendered with wrong filtering and that the size is correct
    canvas
        .style()
        .set_css_text("image-rendering: pixelated; outline: none; border: none;");

    Ok(window_attributes.with_canvas(canvas))
}
