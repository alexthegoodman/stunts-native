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
    editor::{Viewport, WindowSize, Editor, Point, WindowSizeShader},
};
use stunts_engine::polygon::{
    Polygon, PolygonConfig, SavedPoint, SavedPolygonConfig, SavedStroke, Stroke,
};
use uuid::Uuid;
use rand::Rng;
use undo::{Edit, Record};
use stunts_engine::{
    animations::Sequence, timelines::SavedTimelineStateConfig,
};
use winit::event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta};
use winit::dpi::{LogicalSize, PhysicalSize};
use stunts_engine::gpu_resources::GpuResources;
use editor_state::EditorState;

mod primary_canvas;
mod pipeline;
mod render_integration;
mod helpers;
mod editor_state;

#[derive(Debug, Clone)]
enum Command {
    AddSquarePolygon,
}

// NOTE: these handlers are tied to winit events, the other ones are tied to the editor
fn handle_cursor_moved(
    editor: std::sync::Arc<Mutex<Editor>>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(f64, f64, f64, f64)>> {
    Some(Box::new(
        move |positionX: f64, positionY: f64, logPosX: f64, logPoxY: f64| {
            let mut editor = editor.lock().unwrap();
            if let Some(gpu_resources) = editor.gpu_resources.clone() {
                let viewport = viewport.lock().unwrap();
                let window_size = WindowSize {
                    width: viewport.width as u32,
                    height: viewport.height as u32,
                };
                drop(viewport);

                // println!("window size {:?}", window_size);
                // println!("Physical Position {:?} {:?}", positionX, positionY);
                // println!("Logical Position {:?} {:?}", logPosX, logPoxY); // logical position is scaled differently than window_size units

                editor.handle_mouse_move(
                    &window_size,
                    &gpu_resources.device,
                    &gpu_resources.queue,
                    positionX as f32,
                    positionY as f32,
                );
                // TODO: need callback for when cursor is done moving, then add translation to undo stack
            }
        },
    ))
}

fn handle_mouse_input(
    mut editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    record: Arc<Mutex<Record<editor_state::ObjectEdit>>>,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor_orig = Arc::clone(&editor);
        let mut editor = editor.lock().unwrap();
        if let Some(gpu_resources) = editor.gpu_resources.clone() {
            let viewport = viewport.lock().unwrap();
            let window_size = WindowSize {
                width: viewport.width as u32,
                height: viewport.height as u32,
            };
            drop(viewport);
            
            if button == MouseButton::Left {
                let edit_config = match state {
                    ElementState::Pressed => editor.handle_mouse_down(
                        // mouse_position.0,
                        // mouse_position.1,
                        &window_size,
                        &gpu_resources.device,
                    ),
                    ElementState::Released => editor.handle_mouse_up(),
                };

                drop(editor);

                // if (edit_config.is_some()) {
                //     let edit_config = edit_config.expect("Couldn't get polygon edit config");

                //     let mut editor_state = editor_state.lock().unwrap();

                //     let edit = ObjectEdit {
                //         polygon_id: edit_config.polygon_id,
                //         old_value: edit_config.old_value,
                //         new_value: edit_config.new_value,
                //         field_name: edit_config.field_name,
                //         signal: None,
                //     };

                //     let mut record_state = RecordState {
                //         editor: editor_orig,
                //         // record: Arc::clone(&record),
                //     };

                //     let mut record = record.lock().unwrap();
                //     record.edit(&mut record_state, edit);
                // }
            }
        }
    }))
}

fn handle_window_resize(
    editor: std::sync::Arc<Mutex<Editor>>,
    // window_size: WindowSize, // need newest window size
    // gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>> {
    Some(Box::new(move |size, logical_size| {
        let mut editor_g = editor.lock().unwrap();
        if let Some(gpu_resources) = editor_g.gpu_resources.clone() {
            let window_size = WindowSize {
                width: size.width,
                height: size.height,
            };

            println!("new window size {:?}", window_size);

            if window_size.width < 10 || window_size.height < 10 {
                return;
            }

        let mut viewport = viewport.lock().unwrap();

        viewport.width = size.width as f32;
        viewport.height = size.height as f32;

        let mut camera = editor_g
            .camera
            .as_mut()
            .expect("Couldn't get camera on resize");

        camera.window_size.width = size.width;
        camera.window_size.height = size.height;

        drop(editor_g);

        let mut editor_g = editor.lock().unwrap();

        let mut camera = editor_g.camera.expect("Couldn't get camera on resize");

        // println!("window 2 {:?}", camera.window_size);

        let mut camera_binding = editor_g
            .camera_binding
            .as_mut()
            .expect("Couldn't get camera binding");
        camera_binding.update(&gpu_resources.queue, &camera);

        gpu_resources.queue.write_buffer(
            &editor_g
                .window_size_buffer
                .as_ref()
                .expect("Couldn't get window size buffer"),
            0,
            bytemuck::cast_slice(&[WindowSizeShader {
                width: window_size.width as f32,
                height: window_size.height as f32,
            }]),
        );

        drop(editor_g);

        // NOTE: reenable later
        // let mut gpu_helper = gpu_helper.lock().unwrap();

        // gpu_helper.recreate_depth_view(&gpu_resources, size.width, size.height);

        // drop(gpu_helper);
        }
    }))
}

fn handle_mouse_wheel(
    editor: std::sync::Arc<Mutex<Editor>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(MouseScrollDelta)>> {
    Some(Box::new(move |delta: MouseScrollDelta| {
        let mut editor = editor.lock().unwrap();
        if let Some(gpu_resources) = editor.gpu_resources.clone() {
            let mouse_pos = Point {
                x: editor.last_top_left.x,
                y: editor.last_top_left.y,
            };

            match delta {
                MouseScrollDelta::LineDelta(_x, y) => {
                    // y is positive for scrolling up/away from user
                    // negative for scrolling down/toward user
                    // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
                    editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    // Convert pixel delta if needed
                    let y = pos.y as f32;
                    // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
                    editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
                }
            }
        }
    }))
}

fn handle_modifiers_changed(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(Modifiers)>> {
    Some(Box::new(move |modifiers: Modifiers| {
        let mut editor_state = editor_state.lock().unwrap();
        println!("modifiers changed");
        let modifier_state = modifiers.state();
        // editor_state.current_modifiers = modifier_state;
    }))
}

fn handle_keyboard_input(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(KeyEvent)>> {
    Some(Box::new(move |event: KeyEvent| {
        if event.state != ElementState::Pressed {
            return;
        }

        let mut editor_state = editor_state.lock().unwrap();
        // let editor: MutexGuard<'_, Editor> = editor_state.editor.lock().unwrap();
        // Check for Ctrl+Z (undo)
        // let modifiers = editor_state.current_modifiers;

        // match event.logical_key {
        //     Key::Character(c) if c == SmolStr::new("z") => {
        //         if modifiers.control_key() {
        //             if modifiers.shift_key() {
        //                 editor_state.redo(); // Ctrl+Shift+Z
        //             } else {
        //                 println!("undo!");
        //                 editor_state.undo(); // Ctrl+Z
        //             }
        //         }
        //     }
        //     Key::Character(c) if c == SmolStr::new("y") => {
        //         if modifiers.control_key() {
        //             editor_state.redo(); // Ctrl+Y
        //         }
        //     }
        //     _ => {}
        // }
    }))
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
    let mut editor = Editor::new(viewport.clone());

    // dummy project
    let project_id = Uuid::new_v4();
    let destination_view = "scene".to_string();
    let dummy_sequence_id = Uuid::new_v4();

    editor.project_selected = Some(project_id.clone());
    editor.current_view = destination_view.clone();

    let editor = Arc::new(Mutex::new(editor));

    // editor_state holds saved data, not active gpu data
    let cloned_editor = Arc::clone(&editor);
    let record = Arc::new(Mutex::new(Record::new()));
    let mut editor_state = editor_state::EditorState::new(cloned_editor, record.clone());

    println!("Loading saved state...");

    // let saved_state = load_project_state(uuid.clone().to_string())
    //     .expect("Couldn't get Saved State");

    let mut dummy_sequences = Vec::new();

    dummy_sequences.push(Sequence  {
        id: dummy_sequence_id.to_string(),
        name: "Sequence 1".to_string(),
        background_fill: None,
        duration_ms: 20000,
        active_polygons: Vec::new(),
        polygon_motion_paths: Vec::new(),
        active_text_items: Vec::new(),
        active_image_items: Vec::new(),
        active_video_items: Vec::new(),
    });
    
    let saved_state = helpers::saved_state::SavedState {
        id: project_id.to_string(),
        // name: "New Project".to_string(),
        sequences: dummy_sequences,
        timeline_state: SavedTimelineStateConfig {
            timeline_sequences: Vec::new(),
        },
    };
    
    editor_state.record_state.saved_state = Some(saved_state.clone());
    
    let editor_state = Arc::new(Mutex::new(editor_state));

    // Create channel for communicating commands from UI to main thread
    let (command_tx, command_rx) = mpsc::channel::<Command>();

    // NOTE: these handlers get attached to editor (the other ones above get attached to winit events)
    // let handle_polygon_click: Arc<PolygonClickHandler> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(
    //             Box::new(move |polygon_id: Uuid, polygon_data: PolygonConfig| {
    //                 // cannot lock editor here! probably because called from Editor
                    
    //                 // set_polygon_selected(
    //                 //     editor_state.clone(),
    //                 //     text_selected_ref.clone(),
    //                 //     polygon_selected_ref.clone(),
    //                 //     image_selected_ref.clone(),
    //                 //     video_selected_ref.clone(),
    //                 //     selected_text_id_ref.clone(),
    //                 //     selected_polygon_id_ref.clone(),
    //                 //     selected_image_id_ref.clone(),
    //                 //     selected_video_id_ref.clone(),
    //                 //     selected_polygon_data_ref.clone(),
    //                 //     polygon_id,
    //                 //     polygon_data,
    //                 // );

    //                 // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //                 //     let editor_state = editor_state.lock().unwrap();
    //                 //     let saved_state = editor_state
    //                 //         .record_state
    //                 //         .saved_state
    //                 //         .as_ref()
    //                 //         .expect("Couldn't get Saved State");

    //                 //     let saved_animation_data = saved_state
    //                 //         .sequences
    //                 //         .iter()
    //                 //         .flat_map(|s| s.polygon_motion_paths.iter())
    //                 //         .find(|p| p.polygon_id == polygon_id.to_string());

    //                 //     if let Some(polygon_animation_data) = saved_animation_data {
    //                 //         animation_data.update(|c| {
    //                 //             *c = Some(polygon_animation_data.clone());
    //                 //         });
    //                 //     } else {
    //                 //         // polygon is not saved animation data
    //                 //         // polygon_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
    //                 //     }

    //                 //     drop(editor_state);
    //                 // }
    //             }) as Box<dyn FnMut(Uuid, PolygonConfig)>,
    //         )
    //     }
    // });

    // let handle_image_click: Arc<ImageItemClickHandler> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(Box::new(move |image_id: Uuid, image_data: StImageConfig| {
    //             // cannot lock editor here! probably because called from Editor

    //             // set_image_selected(
    //             //     editor_state.clone(),
    //             //     text_selected_ref.clone(),
    //             //     polygon_selected_ref.clone(),
    //             //     image_selected_ref.clone(),
    //             //     video_selected_ref.clone(),
    //             //     selected_text_id_ref.clone(),
    //             //     selected_polygon_id_ref.clone(),
    //             //     selected_image_id_ref.clone(),
    //             //     selected_video_id_ref.clone(),
    //             //     selected_image_data_ref.clone(),
    //             //     image_id,
    //             //     image_data,
    //             // );

    //             // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //             //     let editor_state = editor_state.lock().unwrap();
    //             //     let saved_state = editor_state
    //             //         .record_state
    //             //         .saved_state
    //             //         .as_ref()
    //             //         .expect("Couldn't get Saved State");

    //             //     let saved_animation_data = saved_state
    //             //         .sequences
    //             //         .iter()
    //             //         .flat_map(|s| s.polygon_motion_paths.iter())
    //             //         .find(|p| p.polygon_id == image_id.to_string());

    //             //     if let Some(image_animation_data) = saved_animation_data {
    //             //         animation_data.update(|c| {
    //             //             *c = Some(image_animation_data.clone());
    //             //         });
    //             //     } else {
    //             //         // image is not saved animation data
    //             //         // image_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
    //             //     }

    //             //     drop(editor_state);
    //             // }
    //         }) as Box<dyn FnMut(Uuid, StImageConfig)>)
    //     }
    // });

    // let handle_text_click: Arc<TextItemClickHandler> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(
    //             Box::new(move |text_id: Uuid, text_data: TextRendererConfig| {
    //                 // cannot lock editor here! probably because called from Editor

    //                 // set_text_selected(
    //                 //     editor_state.clone(),
    //                 //     text_selected_ref.clone(),
    //                 //     polygon_selected_ref.clone(),
    //                 //     image_selected_ref.clone(),
    //                 //     video_selected_ref.clone(),
    //                 //     selected_text_id_ref.clone(),
    //                 //     selected_polygon_id_ref.clone(),
    //                 //     selected_image_id_ref.clone(),
    //                 //     selected_video_id_ref.clone(),
    //                 //     selected_text_data_ref.clone(),
    //                 //     text_id,
    //                 //     text_data,
    //                 // );

    //                 // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //                 //     let editor_state = editor_state.lock().unwrap();
    //                 //     let saved_state = editor_state
    //                 //         .record_state
    //                 //         .saved_state
    //                 //         .as_ref()
    //                 //         .expect("Couldn't get Saved State");

    //                 //     let saved_animation_data = saved_state
    //                 //         .sequences
    //                 //         .iter()
    //                 //         .flat_map(|s| s.polygon_motion_paths.iter())
    //                 //         .find(|p| p.polygon_id == text_id.to_string());

    //                 //     if let Some(text_animation_data) = saved_animation_data {
    //                 //         animation_data.update(|c| {
    //                 //             *c = Some(text_animation_data.clone());
    //                 //         });
    //                 //     } else {
    //                 //         // text is not saved animation data
    //                 //         // text_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
    //                 //     }

    //                 //     drop(editor_state);
    //                 // }
    //             }) as Box<dyn FnMut(Uuid, TextRendererConfig)>,
    //         )
    //     }
    // });

    // let handle_video_click: Arc<VideoItemClickHandler> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(Box::new(move |video_id: Uuid, video_data: StVideoConfig| {
    //             // cannot lock editor here! probably because called from Editor

    //             // set_video_selected(
    //             //     editor_state.clone(),
    //             //     text_selected_ref.clone(),
    //             //     polygon_selected_ref.clone(),
    //             //     image_selected_ref.clone(),
    //             //     video_selected_ref.clone(),
    //             //     selected_text_id_ref.clone(),
    //             //     selected_polygon_id_ref.clone(),
    //             //     selected_image_id_ref.clone(),
    //             //     selected_video_id_ref.clone(),
    //             //     selected_video_data_ref.clone(),
    //             //     video_id,
    //             //     video_data,
    //             // );

    //             // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //             //     let editor_state = editor_state.lock().unwrap();
    //             //     let saved_state = editor_state
    //             //         .record_state
    //             //         .saved_state
    //             //         .as_ref()
    //             //         .expect("Couldn't get Saved State");

    //             //     let saved_animation_data = saved_state
    //             //         .sequences
    //             //         .iter()
    //             //         .flat_map(|s| s.polygon_motion_paths.iter())
    //             //         .find(|p| p.polygon_id == video_id.to_string());

    //             //     if let Some(video_animation_data) = saved_animation_data {
    //             //         println!(
    //             //             "Setting animation data {:?}",
    //             //             video_animation_data.polygon_id
    //             //         );
    //             //         animation_data.update(|c| {
    //             //             *c = Some(video_animation_data.clone());
    //             //         });
    //             //     } else {
    //             //         // image is not saved animation data
    //             //         // image_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
    //             //     }

    //             //     drop(editor_state);
    //             // }
    //         }) as Box<dyn FnMut(Uuid, StVideoConfig)>)
    //     }
    // });

    // let on_mouse_up: Arc<OnMouseUp> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(Box::new(move |object_id: Uuid, point: Point| {
    //             // // cannot lock editor here! probably because called from Editor
    //             // // {
    //             // //     let mut editor = new_editor.lock().unwrap();
    //             // //     // Update editor as needed
    //             // // }

    //             // // let value = string_to_f32(&value).map_err(|_| "Couldn't convert string to f32").expect("Couldn't convert string to f32");

    //             // let mut current_animation_data = animation_data
    //             //     .get()
    //             //     .expect("Couldn't get current Animation Data");
    //             // let mut current_keyframe = selected_keyframes.get();

    //             // let mut editor_state = editor_state.lock().unwrap();

    //             // if let Some(current_keyframe) = current_keyframe.get_mut(0) {
    //             //     // let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
    //             //     let mut current_sequence = selected_sequence_data.get();
    //             //     // let current_polygon = selected_polygon_data.read();
    //             //     // let current_polygon = current_polygon.borrow();

    //             //     // update keyframe
    //             //     current_keyframe.value =
    //             //         KeyframeValue::Position([point.x as i32, point.y as i32]);

    //             //     update_keyframe(
    //             //         editor_state,
    //             //         current_animation_data,
    //             //         current_keyframe,
    //             //         current_sequence,
    //             //         selected_keyframes,
    //             //         animation_data,
    //             //         selected_sequence_data,
    //             //         selected_sequence_id,
    //             //         sequence_selected,
    //             //     );

    //             //     println!("Keyframe updated!");
    //             // } else {
    //             //     let last_saved_state = editor_state
    //             //         .record_state
    //             //         .saved_state
    //             //         .as_mut()
    //             //         .expect("Couldn't get Saved State");

    //             //     let object_type = find_object_type(&last_saved_state, &object_id);

    //             //     if let Some(object_type) = object_type.clone() {
    //             //         last_saved_state.sequences.iter_mut().for_each(|s| {
    //             //             if s.id == selected_sequence_id.get() {
    //             //                 match object_type {
    //             //                     ObjectType::Polygon => {
    //             //                         s.active_polygons.iter_mut().for_each(|ap| {
    //             //                             if ap.id == object_id.to_string() {
    //             //                                 ap.position = SavedPoint {
    //             //                                     x: point.x as i32,
    //             //                                     y: point.y as i32,
    //             //                                 }
    //             //                             }
    //             //                         });
    //             //                     }
    //             //                     ObjectType::TextItem => {
    //             //                         s.active_text_items.iter_mut().for_each(|tr| {
    //             //                             if tr.id == object_id.to_string() {
    //             //                                 tr.position = SavedPoint {
    //             //                                     x: point.x as i32,
    //             //                                     y: point.y as i32,
    //             //                                 }
    //             //                             }
    //             //                         });
    //             //                     }
    //             //                     ObjectType::ImageItem => {
    //             //                         s.active_image_items.iter_mut().for_each(|si| {
    //             //                             if si.id == object_id.to_string() {
    //             //                                 si.position = SavedPoint {
    //             //                                     x: point.x as i32,
    //             //                                     y: point.y as i32,
    //             //                                 }
    //             //                             }
    //             //                         });
    //             //                     }
    //             //                     ObjectType::VideoItem => {
    //             //                         s.active_video_items.iter_mut().for_each(|si| {
    //             //                             if si.id == object_id.to_string() {
    //             //                                 si.position = SavedPoint {
    //             //                                     x: point.x as i32,
    //             //                                     y: point.y as i32,
    //             //                                 }
    //             //                             }
    //             //                         });
    //             //                     }
    //             //                 }
    //             //             }
    //             //         });

    //             //         // TODO: probably perf hit with larger files, or does it get released?
    //             //         let new_saved_state = last_saved_state.to_owned();

    //             //         save_saved_state_raw(new_saved_state);

    //             //         // drop(editor_state);

    //             //         println!("Position updated!");
    //             //     }
    //             // }

    //             // // let mut editor = editor_cloned7.lock().unwrap();
    //             // // editor.update_motion_paths(&selected_sequence_data.get());

    //             // // println!("Motion Paths updated!");

    //             // (selected_sequence_data.get(), selected_keyframes.get())

    //             ()
    //         })
    //             as Box<dyn FnMut(Uuid, Point) -> (Sequence, Vec<UIKeyframe>)>)
    //     }
    // });

    // let on_handle_mouse_up: Arc<OnHandleMouseUp> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(
    //             Box::new(move |keyframe_id: Uuid, object_id: Uuid, point: Point| {
    //                 // cannot lock editor here! probably because called from Editor

    //                 // println!("Updating keyframe via handle...");

    //                 // if (!sequence_selected.get()) {
    //                 //     return (selected_sequence_data.get(), selected_keyframes.get());
    //                 // }

    //                 // let selected_sequence = selected_sequence_data.get();

    //                 // let is_polygon = selected_sequence
    //                 //     .active_polygons
    //                 //     .iter()
    //                 //     .find(|p| p.id == object_id.to_string());
    //                 // let is_image = selected_sequence
    //                 //     .active_image_items
    //                 //     .iter()
    //                 //     .find(|i| i.id == object_id.to_string());
    //                 // let is_text = selected_sequence
    //                 //     .active_text_items
    //                 //     .iter()
    //                 //     .find(|t| t.id == object_id.to_string());
    //                 // let is_video = selected_sequence
    //                 //     .active_video_items
    //                 //     .iter()
    //                 //     .find(|t| t.id == object_id.to_string());

    //                 // if let Some(polygon) = is_polygon {
    //                 //     set_polygon_selected(
    //                 //         editor_state.clone(),
    //                 //         text_selected_ref.clone(),
    //                 //         polygon_selected_ref.clone(),
    //                 //         image_selected_ref.clone(),
    //                 //         video_selected_ref.clone(),
    //                 //         selected_text_id_ref.clone(),
    //                 //         selected_polygon_id_ref.clone(),
    //                 //         selected_image_id_ref.clone(),
    //                 //         selected_video_id_ref.clone(),
    //                 //         selected_polygon_data_ref.clone(),
    //                 //         object_id,
    //                 //         PolygonConfig {
    //                 //             id: Uuid::from_str(&polygon.id)
    //                 //                 .expect("Couldn't convert string to uuid"),
    //                 //             name: polygon.name.clone(),
    //                 //             // TODO: support triangles and other shapes by saving points
    //                 //             points: vec![
    //                 //                 Point { x: 0.0, y: 0.0 },
    //                 //                 Point { x: 1.0, y: 0.0 },
    //                 //                 Point { x: 1.0, y: 1.0 },
    //                 //                 Point { x: 0.0, y: 1.0 },
    //                 //             ],
    //                 //             fill: [
    //                 //                 polygon.fill[0] as f32,
    //                 //                 polygon.fill[1] as f32,
    //                 //                 polygon.fill[2] as f32,
    //                 //                 polygon.fill[3] as f32,
    //                 //             ],
    //                 //             dimensions: (
    //                 //                 polygon.dimensions.0 as f32,
    //                 //                 polygon.dimensions.1 as f32,
    //                 //             ),
    //                 //             position: Point {
    //                 //                 x: polygon.position.x as f32,
    //                 //                 y: polygon.position.y as f32,
    //                 //             },
    //                 //             border_radius: polygon.border_radius as f32,
    //                 //             stroke: Stroke {
    //                 //                 thickness: polygon.stroke.thickness as f32,
    //                 //                 fill: [
    //                 //                     polygon.stroke.fill[0] as f32,
    //                 //                     polygon.stroke.fill[1] as f32,
    //                 //                     polygon.stroke.fill[2] as f32,
    //                 //                     polygon.stroke.fill[3] as f32,
    //                 //                 ],
    //                 //             },
    //                 //             layer: polygon.layer.clone(),
    //                 //         },
    //                 //     );
    //                 // }

    //                 // if let Some(image) = is_image {
    //                 //     set_image_selected(
    //                 //         editor_state.clone(),
    //                 //         text_selected_ref.clone(),
    //                 //         polygon_selected_ref.clone(),
    //                 //         image_selected_ref.clone(),
    //                 //         video_selected_ref.clone(),
    //                 //         selected_text_id_ref.clone(),
    //                 //         selected_polygon_id_ref.clone(),
    //                 //         selected_image_id_ref.clone(),
    //                 //         selected_video_id_ref.clone(),
    //                 //         selected_image_data_ref.clone(),
    //                 //         object_id,
    //                 //         StImageConfig {
    //                 //             id: image.id.clone(),
    //                 //             name: image.name.clone(),
    //                 //             dimensions: image.dimensions,
    //                 //             position: Point {
    //                 //                 x: image.position.x as f32,
    //                 //                 y: image.position.y as f32,
    //                 //             },
    //                 //             path: image.path.clone(),
    //                 //             layer: image.layer.clone(),
    //                 //         },
    //                 //     );
    //                 // }

    //                 // if let Some(text) = is_text {
    //                 //     set_text_selected(
    //                 //         editor_state.clone(),
    //                 //         text_selected_ref.clone(),
    //                 //         polygon_selected_ref.clone(),
    //                 //         image_selected_ref.clone(),
    //                 //         video_selected_ref.clone(),
    //                 //         selected_text_id_ref.clone(),
    //                 //         selected_polygon_id_ref.clone(),
    //                 //         selected_image_id_ref.clone(),
    //                 //         selected_video_id_ref.clone(),
    //                 //         selected_text_data_ref.clone(),
    //                 //         object_id,
    //                 //         TextRendererConfig {
    //                 //             id: Uuid::from_str(&text.id)
    //                 //                 .expect("Couldn't convert string to uuid"),
    //                 //             name: text.name.clone(),
    //                 //             text: text.text.clone(),
    //                 //             font_family: text.font_family.clone(),
    //                 //             dimensions: (text.dimensions.0 as f32, text.dimensions.1 as f32),
    //                 //             position: Point {
    //                 //                 x: text.position.x as f32,
    //                 //                 y: text.position.y as f32,
    //                 //             },
    //                 //             layer: text.layer.clone(),
    //                 //             color: text.color.clone(),
    //                 //             font_size: text.font_size.clone(),
    //                 //             background_fill: text
    //                 //                 .background_fill
    //                 //                 .unwrap_or([200, 200, 200, 255]),
    //                 //         },
    //                 //     );
    //                 // }

    //                 // if let Some(video) = is_video {
    //                 //     set_video_selected(
    //                 //         editor_state.clone(),
    //                 //         text_selected_ref.clone(),
    //                 //         polygon_selected_ref.clone(),
    //                 //         image_selected_ref.clone(),
    //                 //         video_selected_ref.clone(),
    //                 //         selected_text_id_ref.clone(),
    //                 //         selected_polygon_id_ref.clone(),
    //                 //         selected_image_id_ref.clone(),
    //                 //         selected_video_id_ref.clone(),
    //                 //         selected_video_data_ref.clone(),
    //                 //         object_id,
    //                 //         StVideoConfig {
    //                 //             id: video.id.clone(),
    //                 //             name: video.name.clone(),
    //                 //             dimensions: video.dimensions,
    //                 //             position: Point {
    //                 //                 x: video.position.x as f32,
    //                 //                 y: video.position.y as f32,
    //                 //             },
    //                 //             path: video.path.clone(),
    //                 //             layer: video.layer.clone(),
    //                 //             mouse_path: video.mouse_path.clone(),
    //                 //         },
    //                 //     );
    //                 // }

    //                 // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //                 //     let editor_state = editor_state.lock().unwrap();
    //                 //     let saved_state = editor_state
    //                 //         .record_state
    //                 //         .saved_state
    //                 //         .as_ref()
    //                 //         .expect("Couldn't get Saved State");

    //                 //     let saved_animation_data = saved_state
    //                 //         .sequences
    //                 //         .iter()
    //                 //         .flat_map(|s| s.polygon_motion_paths.iter())
    //                 //         .find(|p| p.polygon_id == object_id.to_string());

    //                 //     if let Some(object_animation_data) = saved_animation_data {
    //                 //         animation_data.update(|c| {
    //                 //             *c = Some(object_animation_data.clone());
    //                 //         });
    //                 //     } else {
    //                 //         // text is not saved animation data
    //                 //         // text_index,time,width,height,x,y,rotation,scale,perspective_x,perspective_y,opacity
    //                 //     }

    //                 //     drop(editor_state);
    //                 // }

    //                 // let mut current_animation_data = animation_data
    //                 //     .get()
    //                 //     .expect("Couldn't get current Animation Data");

    //                 // let mut data = current_animation_data.clone();

    //                 // let current_keyframe = data.properties.iter_mut().find_map(|a| {
    //                 //     a.keyframes
    //                 //         .iter_mut()
    //                 //         .find(|kf| kf.id == keyframe_id.to_string())
    //                 // });

    //                 // // get current_keyframe from handle

    //                 // let mut editor_state = editor_state.lock().unwrap();

    //                 // if let Some(current_keyframe) = current_keyframe {
    //                 //     println!("Current keyframe found...");

    //                 //     // let mut current_keyframe = current_keyframe.get_mut(0).expect("Couldn't get Selected Keyframe");
    //                 //     let mut current_sequence = selected_sequence_data.get();

    //                 //     // update keyframe
    //                 //     current_keyframe.value =
    //                 //         KeyframeValue::Position([point.x as i32, point.y as i32]);

    //                 //     update_keyframe(
    //                 //         editor_state,
    //                 //         current_animation_data,
    //                 //         current_keyframe,
    //                 //         current_sequence,
    //                 //         selected_keyframes,
    //                 //         animation_data,
    //                 //         selected_sequence_data,
    //                 //         selected_sequence_id,
    //                 //         sequence_selected,
    //                 //     );

    //                 //     println!("Keyframe updated!");
    //                 // } else {
    //                 //     println!("Current keyframe not found!");
    //                 // }

    //                 // (selected_sequence_data.get(), selected_keyframes.get())

    //                 ()
    //             })
    //                 as Box<dyn FnMut(Uuid, Uuid, Point) -> (Sequence, Vec<UIKeyframe>)>,
    //         )
    //     }
    // });

    // let on_path_mouse_up: Arc<OnPathMouseUp> = Arc::new({
    //     let editor_state = editor_state.clone();

    //     move || {
    //         let editor_state = editor_state.clone();

    //         Some(Box::new(move |path_id: Uuid, point: Point| {
    //             // cannot lock editor here! probably because called from Editor

    //             // println!("Updating path... {:?} {:?}", path_id, point);

    //             // if (!sequence_selected.get()) {
    //             //     return (selected_sequence_data.get(), selected_keyframes.get());
    //             // }

    //             // let mut selected_sequence = selected_sequence_data.get();

    //             // // update selected sequence data with new path data
    //             // selected_sequence
    //             //     .polygon_motion_paths
    //             //     .iter_mut()
    //             //     .for_each(|p| {
    //             //         if p.id == path_id.to_string() {
    //             //             p.position = [point.x as i32, point.y as i32];
    //             //         }
    //             //     });

    //             // selected_sequence_data.set(selected_sequence);

    //             // // save to saved state
    //             // if let Ok(mut animation_data) = animation_data_ref.lock() {
    //             //     let mut editor_state = editor_state.lock().unwrap();
    //             //     let saved_state = editor_state
    //             //         .record_state
    //             //         .saved_state
    //             //         .as_ref()
    //             //         .expect("Couldn't get Saved State");

    //             //     let saved_animation_data = saved_state
    //             //         .sequences
    //             //         .iter()
    //             //         .flat_map(|s| s.polygon_motion_paths.iter())
    //             //         .find(|p| p.id == path_id.to_string());

    //             //     if let Some(object_animation_data) = saved_animation_data {
    //             //         let mut updated_animation_data = object_animation_data.clone();

    //             //         updated_animation_data.position = [point.x as i32, point.y as i32];

    //             //         animation_data.set(Some(updated_animation_data));
    //             //     }

    //             //     let mut new_saved_state = saved_state.clone();

    //             //     new_saved_state.sequences.iter_mut().for_each(|s| {
    //             //         if s.id == selected_sequence_id.get() {
    //             //             s.polygon_motion_paths.iter_mut().for_each(|pm| {
    //             //                 if pm.id == path_id.to_string() {
    //             //                     pm.position = [point.x as i32, point.y as i32];
    //             //                 }
    //             //             });
    //             //         }
    //             //     });

    //             //     editor_state.record_state.saved_state = Some(new_saved_state.clone());

    //             //     save_saved_state_raw(new_saved_state);

    //             //     println!("Path updated!");

    //             //     drop(editor_state);
    //             // }

    //             // (selected_sequence_data.get(), selected_keyframes.get())

    //             ()
    //         })
    //             as Box<dyn FnMut(Uuid, Point) -> (Sequence, Vec<UIKeyframe>)>)
    //     }
    // });

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
        .with_size(250.0, 50.0)
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
        
    // Start the application with the UI tree and event handlers
    let app = App::new()
        .with_title("Stunts".to_string())?
        .with_inner_size([window_size.width as i32, window_size.height as i32])?
        .with_root(root)?
        .with_cursor_moved({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |position_x: f64, position_y: f64, log_pos_x: f64, log_pos_y: f64| {
                if let Some(handler) = handle_cursor_moved(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(position_x, position_y, log_pos_x, log_pos_y);
                }
            }
        })
        .with_mouse_input({
            let editor = editor.clone();
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            let record = record.clone();
            move |button, state| {
                if let Some(handler) = handle_mouse_input(
                    editor_state.clone(),
                    editor.clone(),
                    viewport.clone(),
                    record.clone(),
                ) {
                    handler(button, state);
                }
            }
        })
        .with_window_resize({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |size, logical_size| {
                if let Some(mut handler) = handle_window_resize(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(size, logical_size);
                }
            }
        })
        .with_mouse_wheel({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |delta| {
                if let Some(mut handler) = handle_mouse_wheel(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(delta);
                }
            }
        })
        .with_modifiers_changed({
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            move |modifiers| {
                if let Some(mut handler) = handle_modifiers_changed(
                    editor_state.clone(),
                    viewport.clone(),
                ) {
                    handler(modifiers);
                }
            }
        })
        .with_keyboard_input({
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            move |event| {
                if let Some(mut handler) = handle_keyboard_input(
                    editor_state.clone(),
                    viewport.clone(),
                ) {
                    handler(event);
                }
            }
        });

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
            let state_for_render = editor_state.clone();
            let command_rx_for_render = Arc::new(Mutex::new(command_rx));
            let engine_handle_cache: RefCell<Option<render_integration::EngineHandle>> = RefCell::new(None);
            
            Arc::new(move |device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, external_resources: &[vello::ExternalResource<'_>], view: &wgpu::TextureView| -> Result<(), vello::Error> {
                // Process any pending commands from the UI thread
                if let Ok(rx) = command_rx_for_render.try_lock() {
                    while let Ok(command) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            if let Ok(mut editor_state) = state_for_render.try_lock() {
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

                                        
                                        editor.add_polygon(
                                            polygon_config.clone(),
                                            polygon_config.name.clone(),
                                            polygon_config.id,
                                            dummy_sequence_id.to_string(),
                                        );

                                        editor_state.add_saved_polygon(
                                            dummy_sequence_id.to_string(),
                                            SavedPolygonConfig {
                                                id: polygon_config.id.to_string().clone(),
                                                name: polygon_config.name.clone(),
                                                dimensions: (
                                                    polygon_config.dimensions.0 as i32,
                                                    polygon_config.dimensions.1 as i32,
                                                ),
                                                fill: [
                                                    polygon_config.fill[0] as i32,
                                                    polygon_config.fill[1] as i32,
                                                    polygon_config.fill[2] as i32,
                                                    polygon_config.fill[3] as i32,
                                                ],
                                                border_radius: polygon_config.border_radius as i32, // multiply by 100?
                                                position: SavedPoint {
                                                    x: polygon_config.position.x as i32,
                                                    y: polygon_config.position.y as i32,
                                                },
                                                stroke: SavedStroke {
                                                    thickness: polygon_config.stroke.thickness as i32,
                                                    fill: [
                                                        polygon_config.stroke.fill[0] as i32,
                                                        polygon_config.stroke.fill[1] as i32,
                                                        polygon_config.stroke.fill[2] as i32,
                                                        polygon_config.stroke.fill[3] as i32,
                                                    ],
                                                },
                                                layer: polygon_config.layer.clone(),
                                            },
                                        );

                                        let saved_state = editor_state
                                            .record_state
                                            .saved_state
                                            .as_ref()
                                            .expect("Couldn't get saved state");
                                        let updated_sequence = saved_state
                                            .sequences
                                            .iter()
                                            .find(|s| s.id == dummy_sequence_id.to_string())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        // drop(editor_state);

                                        // let mut editor = editor_cloned.lock().unwrap();

                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);
                                        
                                        // drop(editor);

                                        println!("Square polygon added to editor successfully: {}", polygon_config.id);
                                    }
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