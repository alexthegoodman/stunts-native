use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::Padding;
use gui_core::widgets::text::text_signal;
use gui_reactive::Signal;
use vello::peniko::Color;
use gui_core::widgets::canvas::canvas;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine};
use wgpu::{Device, Queue};

mod primary_canvas;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Stunts...");
    
    let perc_button_1 = button("50% Width")
        .with_width_perc(50.0)
        .with_height(40.0)
        .with_colors(
            Color::rgba8(255, 100, 100, 255),
            Color::rgba8(255, 120, 120, 255),
            Color::rgba8(200, 80, 80, 255)
        );

    let main_column = column()
        .with_size_perc(30.0, 80.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(perc_button_1)));

    let main_row = row()
        .with_size(1200.0, 800.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_gap(40.0)
        .with_child(main_column.into_container_element())
        .with_child(primary_canvas::create_advanced_canvas_app()?);
    
    let container = container()
        .with_size(1200.0, 800.0) 
        .with_background_color(Color::rgba8(240, 240, 240, 255))
        // .with_padding(Padding::only(50.0, 0.0, 0.0, 0.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(main_row.into_container_element());
    
    let root = container.into_container_element();

    println!("UI Tree Built! Launching...");
    
    // Start the application with the UI tree
    let app = App::new().with_title("Stunts".to_string())?.with_inner_size([1200, 800])?.with_root(root)?;
    app.run()
}