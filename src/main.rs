use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::{Padding, Background};
use gui_core::widgets::text::text_signal;
use gui_reactive::Signal;
use vello::peniko::{Color, Gradient, GradientKind, ColorStops, Extend};
use gui_core::widgets::canvas::canvas;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine};
use wgpu::{Device, Queue};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::sync::mpsc;
use stunts_engine::{
    editor::{Viewport, WindowSize, Editor, Point},
    polygon::{PolygonConfig, Stroke},
    // editor_state::EditorState,
    // editor_utilities::EditorUtilities,
};
use uuid::Uuid;
use rand::Rng;
use undo::{Edit, Record};

mod primary_canvas;
mod pipeline;
mod render_integration;
mod helpers;
mod editor_state;

#[derive(Debug, Clone)]
enum Command {
    AddSquarePolygon,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Stunts Native...");

    let window_size = WindowSize {
        width: 1200,
        height: 800,
    };

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    )));

    // let's try with the unified editor.rs
    let editor = Editor::new(viewport.clone());
    let editor = Arc::new(Mutex::new(editor));

    // editor_state holds saved data, not active gpu data
    let cloned_editor = Arc::clone(&editor);
    let record = Arc::new(Mutex::new(Record::new()));
    let editor_state = Arc::new(Mutex::new(editor_state::EditorState::new(cloned_editor, record)));

    // Create channel for communicating commands from UI to main thread
    let (command_tx, command_rx) = mpsc::channel::<Command>();
    // Create gradients for button1 states
    let button_normal = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(75, 75, 80), Color::rgb8(60, 60, 65)]);
    let button_hover = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(85, 85, 90), Color::rgb8(70, 70, 75)]);
    let button_pressed = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(50, 50, 55), Color::rgb8(65, 65, 70)]);
    
    let button1 = button("Add Asset")
        .with_font_size(10.0)
        .with_width(100.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {                
                tx.send(Command::AddSquarePolygon);
            }
        });
    
    // turns on a mode in the editor so the user can draw arrows by clicking and dragging or dots by just clicking
    let button2 = button("Add Motion")
        .with_font_size(10.0)
        .with_width(100.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal),
            Background::Gradient(button_hover),
            Background::Gradient(button_pressed)
        )
        .on_click({
            let tx = command_tx.clone();
            move || {                
                tx.send(Command::AddSquarePolygon);
            }
        });

    let toolkit = row()
        .with_size(400.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        // FIXED: Added missing RowWidget positioning support in Element::position_child_element_static
        .with_child(Element::new_widget(Box::new(button1)))
        .with_child(Element::new_widget(Box::new(button2)));

    let scaffolding = column()
        .with_size(350.0, 50.0) // FIXED: Improved row layout to use intrinsic child sizes instead of equal distribution
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        // .with_gap(40.0)
        .with_child(toolkit.into_container_element())
        .with_child(primary_canvas::create_render_placeholder()?);
    
    // Create a radial gradient for container
    let container_gradient = Gradient::new_radial((0.0, 0.0), 450.0)
        .with_stops([Color::rgb8(90, 90, 95), Color::rgb8(45, 45, 50)]);
    
    let container = container()
        .with_size(1200.0, 800.0) 
        .with_radial_gradient(container_gradient)
        // .with_padding(Padding::only(50.0, 0.0, 0.0, 0.0))
        .with_padding(Padding::all(20.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(scaffolding.into_container_element());
    
    let root = container.into_container_element();

    println!("UI Tree Built! Launching...");
        
    // Start the application with the UI tree
    let app = App::new()
        .with_title("Stunts".to_string())?
        .with_inner_size([window_size.width as i32, window_size.height as i32])?
        .with_root(root)?;

    // Use the new run_with_editor_state method that avoids Send + Sync constraints
    app.run_with_editor_state(
        editor.clone(),
        {
            let editor_for_init = editor.clone();
            let viewport_for_init = viewport.clone();
            move |device, queue| {
                let gpu_resources = std::sync::Arc::new(stunts_engine::gpu_resources::GpuResources::from_commonui(device, queue));
                pipeline::init_pipeline(viewport_for_init, editor_for_init, gpu_resources.clone());
                // render_integration::set_gpu_resources(gpu_resources);
            }
        },
        {
            let editor_for_render = editor.clone();
            let command_rx_for_render = Arc::new(Mutex::new(command_rx));
            let engine_handle_cache: RefCell<Option<render_integration::EngineHandle>> = RefCell::new(None);
            
            Arc::new(move |device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, external_resources: &[vello::ExternalResource<'_>], view: &wgpu::TextureView| -> Result<(), vello::Error> {
                // Process any pending commands from the UI thread
                if let Ok(rx) = command_rx_for_render.try_lock() {
                    while let Ok(command) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            match command {
                                Command::AddSquarePolygon => {
                                    println!("Processing add square polygon command from channel");
                                    let random_coords = helpers::utilities::get_random_coords(window_size);
                                    let new_id = Uuid::new_v4();

                                    let polygon_config = PolygonConfig {
                                        id: new_id,
                                        name: "Square".to_string(),
                                        points: vec![
                                            Point { x: 0.0, y: 0.0 },
                                            Point { x: 1.0, y: 0.0 },
                                            Point { x: 1.0, y: 1.0 },
                                            Point { x: 0.0, y: 1.0 },
                                        ],
                                        dimensions: (100.0, 100.0),
                                        position: Point {
                                            x: random_coords.0 as f32,
                                            y: random_coords.1 as f32,
                                        },
                                        border_radius: 0.0,
                                        fill: [1.0, 0.0, 0.0, 1.0], // Red color
                                        stroke: Stroke {
                                            fill: [0.0, 0.0, 0.0, 1.0], // Black border
                                            thickness: 2.0,
                                        },
                                        layer: -2,
                                    };

                                    let dummy_sequence_id = Uuid::new_v4();
                                    editor.add_polygon(
                                        polygon_config.clone(),
                                        polygon_config.name.clone(),
                                        polygon_config.id,
                                        dummy_sequence_id.to_string(),
                                    );
                                    println!("Square polygon added to editor successfully: {}", polygon_config.id);
                                }
                            }
                        } else {
                            println!("Could not acquire editor lock during render, command not processed");
                        }
                    }
                }
                
                // Create engine handle lazily on first render (after pipeline is initialized)
                let mut cache = engine_handle_cache.borrow_mut();
                if cache.is_none() {
                    if let Some(handle) = render_integration::EngineHandle::try_new(editor_for_render.clone()) {
                        *cache = Some(handle);
                    } else {
                        // Pipeline not ready yet, skip rendering
                        return Ok(());
                    }
                }
                
                render_integration::render_stunts_content(cache.as_ref().unwrap(), device, queue, encoder, external_resources, view)
            })
        }
    )
}