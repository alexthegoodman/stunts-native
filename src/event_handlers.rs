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
use std::time::Duration;
use serde::{Deserialize, Serialize};
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
use crate::editor_state::EditorState;

// NOTE: these handlers are tied to winit events, the other ones are tied to the editor
pub fn handle_cursor_moved(
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
            }
        },
    ))
}

pub fn handle_mouse_input(
    mut editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    record: Arc<Mutex<Record<crate::editor_state::ObjectEdit>>>,
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

pub fn handle_window_resize(
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
        camera_binding.update_3d(&gpu_resources.queue, &camera);

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

pub fn handle_mouse_wheel(
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

pub fn handle_modifiers_changed(
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

pub fn handle_keyboard_input(
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