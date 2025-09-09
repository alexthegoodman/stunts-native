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

// Usage in render pass:
pub fn render_ray_intersection(
    render_pass: &mut wgpu::RenderPass,
    device: &wgpu::Device,
    window_size: &WindowSize,
    editor: &Editor,
    camera: &Camera,
) {
    // if let ray = visualize_ray_intersection(window_size, editor.last_x, editor.last_y, camera) {
    let (vertices, indices, vertex_buffer, index_buffer) = draw_dot(
        device,
        window_size,
        Point {
            x: editor.ds_ndc_pos.x,
            y: editor.ds_ndc_pos.y,
        },
        rgb_to_wgpu(47, 131, 222, 255.0), // Blue dot
        camera,
    );

    // println!("render ray");
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    // }
}

pub fn get_sensor_editor(handle: &EngineHandle) -> Option<Arc<Mutex<Editor>>> {
    handle.user_editor.as_ref().and_then(|e| {
        // let guard = e.lock().ok()?;
        let cloned = e.downcast_ref::<Arc<Mutex<Editor>>>().cloned();
        // drop(guard);
        cloned
    })
}

type RenderCallback<'a> = dyn for<'b> Fn(
        wgpu::CommandEncoder,
        wgpu::SurfaceTexture,
        Arc<wgpu::TextureView>,
        Arc<wgpu::TextureView>,
        // &WindowHandle,
        &Arc<GpuResources>,
        &EngineHandle,
    ) -> (
        Option<wgpu::CommandEncoder>,
        Option<wgpu::SurfaceTexture>,
        Option<Arc<wgpu::TextureView>>,
        Option<Arc<wgpu::TextureView>>,
    ) + 'a;

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