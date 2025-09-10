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
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use stunts_engine::{
    editor::{Viewport, WindowSize, Editor},
    // editor_state::EditorState,
    // editor_utilities::EditorUtilities,
};

mod primary_canvas;
mod pipeline;
mod render_integration;

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

    // // Create the editor state using the new split architecture
    // let editor_state = EditorState::new(viewport.clone());
    // let editor_utilities = EditorUtilities::new();
    
    // // Create the shared editor state
    // let shared_editor_state = render_integration::create_editor_state(editor_state);
    
    // // Initialize the editor utilities (thread-local)
    // render_integration::initialize_editor_utilities(editor_utilities);

    // let's try with the unified editor.rs
    let editor = Editor::new(viewport.clone());
    let editor = Arc::new(Mutex::new(editor));

    
        
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
        .with_child(primary_canvas::create_render_placeholder()?);
    
    let container = container()
        .with_size(1200.0, 800.0) 
        .with_background_color(Color::rgba8(240, 240, 240, 255))
        // .with_padding(Padding::only(50.0, 0.0, 0.0, 0.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(main_row.into_container_element());
    
    let root = container.into_container_element();

    println!("UI Tree Built! Launching...");
        
    // Start the application with the UI tree
    let app = App::new()
        .with_title("Stunts".to_string())?
        .with_inner_size([window_size.width as i32, window_size.height as i32])?
        .with_root(root)?;
        // .with_custom_render({
        //     let editor_for_render = shared_editor_state.clone();
        //     move |device, queue, view, width, height| {
        //         render_integration::render_stunts_content(&editor_for_render, device, queue, view, width, height)
        //     }
        // })
        // .on_resume(move |device, queue| {
        //     println!("App resumed, initializing pipeline with GPU resources...");
            
        //     // Create GPU resources from commonui device and queue
        //     let gpu_resources = stunts_engine::gpu_resources::GpuResources::from_commonui(device, queue);
        //     let gpu_resources = Arc::new(gpu_resources);
            
        //     // Set up GPU resources for rendering integration
        //     render_integration::set_gpu_resources(gpu_resources);
            
        //     println!("GPU resources initialized for stunts rendering");
        // });

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
            let engine_handle_cache: RefCell<Option<render_integration::EngineHandle>> = RefCell::new(None);
            
            Arc::new(move |device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, external_resources: &[vello::ExternalResource<'_>], view: &wgpu::TextureView| -> Result<(), vello::Error> {
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