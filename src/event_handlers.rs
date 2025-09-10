use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use cgmath::Vector4;
use editor_state::{EditorState, ObjectEdit, RecordState};
use stunts_engine::camera::{Camera, CameraBinding};
use stunts_engine::dot::{draw_dot, RingDot};
use stunts_engine::editor::{
    init_editor_with_model, point_to_ndc, ControlMode, Editor, Point, Viewport, WindowSize,
    WindowSizeShader,
};
use stunts_engine::polygon::{Polygon, Stroke};
use stunts_engine::vertex::Vertex;
use uuid::Uuid;
use views::app::app_view;
use wgpu::util::DeviceExt;

use floem::context::PaintState;
use floem::EngineHandle;
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};
use undo::{Edit, Record};

use std::ops::Not;

use cgmath::InnerSpace;
use cgmath::SquareMatrix;
use cgmath::Transform;
use cgmath::{Matrix4, Point3, Vector3};

fn create_render_callback<'a>() -> Box<RenderCallback<'a>> {
    Box::new(
        move |mut encoder: wgpu::CommandEncoder,
              frame: wgpu::SurfaceTexture,
              view: Arc<wgpu::TextureView>,
              resolve_view: Arc<wgpu::TextureView>,
              //   window_handle: &WindowHandle
              gpu_resources: &Arc<GpuResources>,
              engine_handle: &EngineHandle| {
            // let mut handle = window_handle.borrow();
            let mut editor = get_sensor_editor(engine_handle);
            // let mut engine = editor
            //     .as_mut()
            //     .expect("Couldn't get user engine")
            //     .lock()
            //     .unwrap();

            // if let Some(gpu_resources) = &handle.gpu_resources {
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: Some(&resolve_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    // depth_stencil_attachment: None,
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &engine_handle
                            .gpu_helper
                            .as_ref()
                            .expect("Couldn't get gpu helper")
                            .lock()
                            .unwrap()
                            .depth_view
                            .as_ref()
                            .expect("Couldn't fetch depth view"), // This is the depth texture view
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0), // Clear to max depth
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None, // Set this if using stencil
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // println!("Render frame...");

                // Render partial screen content
                // render_pass.set_viewport(100.0, 100.0, 200.0, 200.0, 0.0, 1.0);
                // render_pass.set_scissor_rect(100, 100, 200, 200);

                render_pass.set_pipeline(
                    &engine_handle
                        .render_pipeline
                        .as_ref()
                        .expect("Couldn't fetch render pipeline"),
                );

                // let editor = handle
                //     .user_editor
                //     .as_ref()
                //     .expect("Couldn't get user editor")
                //     .lock()
                //     .unwrap();
                let editor = get_sensor_editor(engine_handle);
                let mut editor = editor
                    .as_ref()
                    .expect("Couldn't get user engine")
                    .lock()
                    .unwrap();

                let camera = editor.camera.expect("Couldn't get camera");

                editor.step_video_animations(&camera, None);
                editor.step_motion_path_animations(&camera, None);

                let camera_binding = editor
                    .camera_binding
                    .as_ref()
                    .expect("Couldn't get camera binding");

                render_pass.set_bind_group(0, &camera_binding.bind_group, &[]);
                render_pass.set_bind_group(
                    2,
                    editor
                        .window_size_bind_group
                        .as_ref()
                        .expect("Couldn't get window size group"),
                    &[],
                );

                // draw static (internal) polygons
                for (poly_index, polygon) in editor.static_polygons.iter().enumerate() {
                    // uniform buffers are pricier, no reason to over-update when idle
                    if let Some(dragging_id) = editor.dragging_path_handle {
                        if dragging_id == polygon.id {
                            polygon
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }
                    }

                    render_pass.set_bind_group(1, &polygon.bind_group, &[]);
                    render_pass.set_bind_group(3, &polygon.group_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        polygon.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..polygon.indices.len() as u32, 0, 0..1);
                }

                // draw motion path static polygons, using motion path transform
                for (path_index, path) in editor.motion_paths.iter().enumerate() {
                    // uniform buffers are pricier, no reason to over-update when idle
                    if let Some(dragging_id) = editor.dragging_path {
                        if dragging_id == path.id {
                            path.transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }
                    }

                    render_pass.set_bind_group(3, &path.bind_group, &[]);

                    for (poly_index, polygon) in path.static_polygons.iter().enumerate() {
                        // uniform buffers are pricier, no reason to over-update when idle
                        if let Some(dragging_id) = editor.dragging_path_handle {
                            if dragging_id == polygon.id {
                                polygon.transform.update_uniform_buffer(
                                    &gpu_resources.queue,
                                    &camera.window_size,
                                );
                            }
                        }

                        render_pass.set_bind_group(1, &polygon.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            polygon.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..polygon.indices.len() as u32, 0, 0..1);
                    }
                }

                // draw polygons
                for (poly_index, polygon) in editor.polygons.iter().enumerate() {
                    if !polygon.hidden {
                        // uniform buffers are pricier, no reason to over-update when idle
                        // also need to remember to update uniform buffers after changes like scale, rotation, position
                        if let Some(dragging_id) = editor.dragging_polygon {
                            if dragging_id == polygon.id {
                                polygon.transform.update_uniform_buffer(
                                    &gpu_resources.queue,
                                    &camera.window_size,
                                );
                            }
                        } else if editor.is_playing {
                            // still need to be careful of playback performance
                            polygon
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }

                        render_pass.set_bind_group(1, &polygon.bind_group, &[]);
                        render_pass.set_bind_group(3, &polygon.group_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, polygon.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            polygon.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..polygon.indices.len() as u32, 0, 0..1);
                    }
                }

                // draw text items
                for (text_index, text_item) in editor.text_items.iter().enumerate() {
                    if !text_item.hidden {
                        if !text_item.background_polygon.hidden {
                            // uniform buffers are pricier, no reason to over-update when idle
                            // also need to remember to update uniform buffers after changes like scale, rotation, position
                            if let Some(dragging_id) = editor.dragging_text {
                                if dragging_id == text_item.background_polygon.id {
                                    text_item
                                        .background_polygon
                                        .transform
                                        .update_uniform_buffer(
                                            &gpu_resources.queue,
                                            &camera.window_size,
                                        );
                                }
                            } else if editor.is_playing {
                                // still need to be careful of playback performance
                                text_item
                                    .background_polygon
                                    .transform
                                    .update_uniform_buffer(
                                        &gpu_resources.queue,
                                        &camera.window_size,
                                    );
                            }

                            render_pass.set_bind_group(
                                1,
                                &text_item.background_polygon.bind_group,
                                &[],
                            );
                            render_pass.set_bind_group(
                                3,
                                &text_item.background_polygon.group_bind_group,
                                &[],
                            );
                            render_pass.set_vertex_buffer(
                                0,
                                text_item.background_polygon.vertex_buffer.slice(..),
                            );
                            render_pass.set_index_buffer(
                                text_item.background_polygon.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            render_pass.draw_indexed(
                                0..text_item.background_polygon.indices.len() as u32,
                                0,
                                0..1,
                            );
                        }

                        // uniform buffers are pricier, no reason to over-update when idle
                        if let Some(dragging_id) = editor.dragging_text {
                            if dragging_id == text_item.id {
                                text_item.transform.update_uniform_buffer(
                                    &gpu_resources.queue,
                                    &camera.window_size,
                                );
                            }
                        } else if editor.is_playing {
                            // still need to be careful of playback performance
                            text_item
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }

                        render_pass.set_bind_group(1, &text_item.bind_group, &[]);
                        render_pass.set_bind_group(3, &text_item.group_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, text_item.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            text_item.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..text_item.indices.len() as u32, 0, 0..1);
                    }
                }

                // draw image items
                for (image_index, st_image) in editor.image_items.iter().enumerate() {
                    if !st_image.hidden {
                        // uniform buffers are pricier, no reason to over-update when idle
                        if let Some(dragging_id) = editor.dragging_image {
                            if dragging_id.to_string() == st_image.id {
                                st_image.transform.update_uniform_buffer(
                                    &gpu_resources.queue,
                                    &camera.window_size,
                                );
                            }
                        } else if editor.is_playing {
                            // still need to be careful of playback performance
                            st_image
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }

                        render_pass.set_bind_group(1, &st_image.bind_group, &[]);
                        render_pass.set_bind_group(3, &st_image.group_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, st_image.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            st_image.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..st_image.indices.len() as u32, 0, 0..1);
                    }
                }

                // draw video items
                for (video_index, st_video) in editor.video_items.iter().enumerate() {
                    if !st_video.hidden {
                        // uniform buffers are pricier, no reason to over-update when idle
                        if let Some(dragging_id) = editor.dragging_video {
                            if dragging_id.to_string() == st_video.id {
                                st_video.transform.update_uniform_buffer(
                                    &gpu_resources.queue,
                                    &camera.window_size,
                                );
                            }
                        } else if editor.is_playing {
                            // still need to be careful of playback performance
                            st_video
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                        }

                        render_pass.set_bind_group(1, &st_video.bind_group, &[]);
                        render_pass.set_bind_group(3, &st_video.group_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, st_video.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            st_video.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..st_video.indices.len() as u32, 0, 0..1);
                    }
                }

                if let Some(dot) = &editor.cursor_dot {
                    dot.transform
                        .update_uniform_buffer(&gpu_resources.queue, &camera.window_size);
                    render_pass.set_bind_group(1, &dot.bind_group, &[]);
                    render_pass.set_bind_group(3, &dot.group_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, dot.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(dot.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..dot.indices.len() as u32, 0, 0..1);
                }

                // much more efficient than calling on mousemove??
                if editor.control_mode == ControlMode::Pan && editor.is_panning {
                    editor.update_camera_binding();
                }
            }

            (Some(encoder), Some(frame), Some(view), Some(resolve_view))
        },
    )
}

fn handle_cursor_moved(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(f64, f64, f64, f64)>> {
    Some(Box::new(
        move |positionX: f64, positionY: f64, logPosX: f64, logPoxY: f64| {
            let mut editor = editor.lock().unwrap();
            let viewport = viewport.lock().unwrap();
            let window_size = WindowSize {
                width: viewport.width as u32,
                height: viewport.height as u32,
            };

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
        },
    ))
}

fn handle_mouse_input(
    mut editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    record: Arc<Mutex<Record<ObjectEdit>>>,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor_orig = Arc::clone(&editor);
        let mut editor = editor.lock().unwrap();
        let viewport = viewport.lock().unwrap();
        let window_size = WindowSize {
            width: viewport.width as u32,
            height: viewport.height as u32,
        };
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
    }))
}

fn handle_window_resize(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    // window_size: WindowSize, // need newest window size
    gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>> {
    Some(Box::new(move |size, logical_size| {
        let mut editor_g = editor.lock().unwrap();

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

        let mut gpu_helper = gpu_helper.lock().unwrap();

        gpu_helper.recreate_depth_view(&gpu_resources, size.width, size.height);

        drop(gpu_helper);
    }))
}

fn handle_mouse_wheel(
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(MouseScrollDelta)>> {
    Some(Box::new(move |delta: MouseScrollDelta| {
        let mut editor = editor.lock().unwrap();

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
    }))
}

fn handle_modifiers_changed(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(Modifiers)>> {
    Some(Box::new(move |modifiers: Modifiers| {
        let mut editor_state = editor_state.lock().unwrap();
        println!("modifiers changed");
        let modifier_state = modifiers.state();
        editor_state.current_modifiers = modifier_state;
    }))
}

use floem_winit::keyboard::NamedKey;
use floem_winit::keyboard::{Key, SmolStr};

fn handle_keyboard_input(
    // editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(KeyEvent)>> {
    Some(Box::new(move |event: KeyEvent| {
        if event.state != ElementState::Pressed {
            return;
        }

        let mut editor_state = editor_state.lock().unwrap();
        // let editor: MutexGuard<'_, Editor> = editor_state.editor.lock().unwrap();
        // Check for Ctrl+Z (undo)
        let modifiers = editor_state.current_modifiers;

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