use gui_core::{Element};
use gui_core::widgets::container::container;
use vello::peniko::Color;
use std::sync::{Arc, Mutex};

pub fn create_render_placeholder() -> Result<Element, Box<dyn std::error::Error>> {
    // Create a placeholder container that will occupy the space where the render area should be
    // The actual rendering will happen directly on the surface in App::render_widgets()
    let placeholder_container = container()
        .with_size(600.0, 400.0)
        .with_background_color(Color::rgba8(20, 20, 20, 255)) // Dark background to indicate render area
        .with_border_radius(4.0);

    let root = container()
        .with_size(800.0, 600.0)
        .with_child(placeholder_container.into_container_element())
        .into_container_element();

    Ok(root)
}