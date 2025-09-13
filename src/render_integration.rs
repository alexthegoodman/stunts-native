use std::sync::Arc;
use stunts_engine::{
    // editor_manager,
    editor::Editor,
    // editor_state::EditorState,
    // editor_utilities::EditorUtilities,
    gpu_resources::GpuResources,
    editor::ControlMode,
};
use vello;

/// Engine handle that owns both the editor and render pipeline separately
/// to avoid borrowing conflicts during rendering
pub struct EngineHandle {
    pub editor: Arc<std::sync::Mutex<Editor>>,
    pub render_pipeline: Arc<wgpu::RenderPipeline>,
}

impl EngineHandle {
    pub fn new(editor: Arc<std::sync::Mutex<Editor>>) -> Self {
        // Extract render_pipeline from editor and store it separately
        let render_pipeline = {
            let editor_lock = editor.lock().unwrap();
            // Clone the Arc<RenderPipeline>
            editor_lock.render_pipeline.as_ref()
                .expect("Render pipeline must be initialized")
                .clone()
        };
        
        Self {
            editor,
            render_pipeline,
        }
    }

    pub fn try_new(editor: Arc<std::sync::Mutex<Editor>>) -> Option<Self> {
        // Try to extract render_pipeline, return None if not ready
        let render_pipeline = {
            let editor_lock = editor.lock().unwrap();
            editor_lock.render_pipeline.as_ref()?.clone()
        };
        
        Some(Self {
            editor,
            render_pipeline,
        })
    }
}

// /// Initialize the editor utilities (state is passed around directly)
// pub fn initialize_editor_utilities(utilities: EditorUtilities) {
//     editor_manager::set_editor_utilities(utilities);
// }

// /// Create editor state (this is just a helper)
// pub fn create_editor_state(state: EditorState) -> std::sync::Arc<std::sync::Mutex<EditorState>> {
//     editor_manager::create_editor_state(state)
// }

/// Legacy function for backward compatibility
// pub fn set_gpu_resources(gpu_resources: Arc<GpuResources>) {
//     // Update the utilities with GPU resources
//     editor_manager::with_editor_utilities_mut(|utilities| {
//         utilities.gpu_resources = Some(gpu_resources);
//     });
// }

/// Custom render function that integrates with Vello's rendering pipeline
/// This gets called by Vello and should add commands to the existing encoder
pub fn render_stunts_content(
    engine_handle: &EngineHandle,
    device: &wgpu::Device,
    queue: &wgpu::Queue, 
    encoder: &mut wgpu::CommandEncoder,
    external_resources: &[vello::ExternalResource<'_>],
    // texture / view
    view: &wgpu::TextureView,
    // sidebar visibility for scissor clipping
    sidebar_visible: bool,
    sidebar_width: f32,
) -> Result<(), vello::Error> {
    let mut editor_lock = engine_handle.editor
        .lock()
        .unwrap();

    // If canvas is hidden, skip all rendering
    if editor_lock.canvas_hidden {
        return Ok(());
    }

    let camera = editor_lock.camera.expect("Couldn't get camera");
    let window_size = &camera.window_size;

    // Update animations before rendering
    editor_lock.step_video_animations(&camera, None);
    editor_lock.step_motion_path_animations(&camera, None);

    // Update camera binding if needed (move this before render pass to avoid borrowing conflicts)
    if editor_lock.control_mode == ControlMode::Pan && editor_lock.is_panning {
        editor_lock.update_camera_binding();
    }

    let depth_view = editor_lock.depth_view.as_ref().expect("Couldn't get depth view");

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Stunts Content Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load, // Don't clear, we want to composite over Vello's content
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_view, // This is the depth texture view
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0), // Clear to max depth
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None, // Set this if using stencil
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    // Apply scissor clipping if sidebar is visible
    if sidebar_visible {
        let canvas_x = sidebar_width;
        let canvas_width = window_size.width as f32 - sidebar_width;
        let canvas_height = window_size.height as f32;
        
        // Set viewport to exclude sidebar area
        render_pass.set_viewport(
            canvas_x,
            0.0,
            canvas_width,
            canvas_height,
            0.0,
            1.0
        );
        
        // Set scissor rect to clip canvas content
        render_pass.set_scissor_rect(
            sidebar_width as u32,
            0,
            (window_size.width as f32 - sidebar_width) as u32,
            window_size.height
        );
    }
    
    // Keep editor_lock alive for the entire render pass duration
    // This is the simplest solution - we accept that the editor is locked during rendering
    
    // Use the render pipeline from engine_handle (no borrowing conflicts!)
    render_pass.set_pipeline(&engine_handle.render_pipeline);

    let camera_binding = editor_lock.camera_binding.as_ref()
        .expect("Couldn't get camera binding");

    render_pass.set_bind_group(0, &camera_binding.bind_group, &[]);
    render_pass.set_bind_group(
        2,
        editor_lock
            .window_size_bind_group
            .as_ref()
            .expect("Couldn't get window size group"),
        &[],
    );

    // draw static (internal) polygons
    for (poly_index, polygon) in editor_lock.static_polygons.iter().enumerate() {
        // uniform buffers are pricier, no reason to over-update when idle
        if let Some(dragging_id) = editor_lock.dragging_path_handle {
            if dragging_id == polygon.id {
                polygon
                    .transform
                    .update_uniform_buffer(&queue, &camera.window_size);
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
    for (path_index, path) in editor_lock.motion_paths.iter().enumerate() {
        // println!("motion path layer {:?}", path.transform.layer);
        // uniform buffers are pricier, no reason to over-update when idle
        if let Some(dragging_id) = editor_lock.dragging_path {
            if dragging_id == path.id {
                path.transform
                    .update_uniform_buffer(&queue, &camera.window_size);
            }
        }

        render_pass.set_bind_group(3, &path.bind_group, &[]);

        for (poly_index, polygon) in path.static_polygons.iter().enumerate() {
            // println!("motion polygon layer {:?}", polygon.transform.layer);
            // uniform buffers are pricier, no reason to over-update when idle
            if let Some(dragging_id) = editor_lock.dragging_path_handle {
                if dragging_id == polygon.id {
                    polygon.transform.update_uniform_buffer(
                        &queue,
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
    for (poly_index, polygon) in editor_lock.polygons.iter().enumerate() {
        if !polygon.hidden {
            // uniform buffers are pricier, no reason to over-update when idle
            // also need to remember to update uniform buffers after changes like scale, rotation, position
            if let Some(dragging_id) = editor_lock.dragging_polygon {
                if dragging_id == polygon.id {
                    polygon.transform.update_uniform_buffer(
                        &queue,
                        &camera.window_size,
                    );
                }
            } else if editor_lock.is_playing {
                // still need to be careful of playback performance
                polygon
                    .transform
                    .update_uniform_buffer(&queue, &camera.window_size);
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
    for (text_index, text_item) in editor_lock.text_items.iter().enumerate() {
        if !text_item.hidden {
            if !text_item.background_polygon.hidden {
                // uniform buffers are pricier, no reason to over-update when idle
                // also need to remember to update uniform buffers after changes like scale, rotation, position
                if let Some(dragging_id) = editor_lock.dragging_text {
                    if dragging_id == text_item.background_polygon.id {
                        text_item
                            .background_polygon
                            .transform
                            .update_uniform_buffer(
                                &queue,
                                &camera.window_size,
                            );
                    }
                } else if editor_lock.is_playing {
                    // still need to be careful of playback performance
                    text_item
                        .background_polygon
                        .transform
                        .update_uniform_buffer(
                            &queue,
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
            if let Some(dragging_id) = editor_lock.dragging_text {
                if dragging_id == text_item.id {
                    text_item.transform.update_uniform_buffer(
                        &queue,
                        &camera.window_size,
                    );
                }
            } else if editor_lock.is_playing {
                // still need to be careful of playback performance
                text_item
                    .transform
                    .update_uniform_buffer(&queue, &camera.window_size);
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
    for (image_index, st_image) in editor_lock.image_items.iter().enumerate() {
        if !st_image.hidden {
            // uniform buffers are pricier, no reason to over-update when idle
            if let Some(dragging_id) = editor_lock.dragging_image {
                if dragging_id.to_string() == st_image.id {
                    st_image.transform.update_uniform_buffer(
                        &queue,
                        &camera.window_size,
                    );
                }
            } else if editor_lock.is_playing {
                // still need to be careful of playback performance
                st_image
                    .transform
                    .update_uniform_buffer(&queue, &camera.window_size);
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
    for (video_index, st_video) in editor_lock.video_items.iter().enumerate() {
        if !st_video.hidden {
            // uniform buffers are pricier, no reason to over-update when idle
            if let Some(dragging_id) = editor_lock.dragging_video {
                if dragging_id.to_string() == st_video.id {
                    st_video.transform.update_uniform_buffer(
                        &queue,
                        &camera.window_size,
                    );
                }
            } else if editor_lock.is_playing {
                // still need to be careful of playback performance
                st_video
                    .transform
                    .update_uniform_buffer(&queue, &camera.window_size);
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

    // draw motion arrows
    for motion_arrow in editor_lock.motion_arrows.iter() {
        if !motion_arrow.hidden {
            if editor_lock.motion_mode {
                motion_arrow.transform.update_uniform_buffer(
                    &queue,
                    &camera.window_size,
                );
            }

            render_pass.set_bind_group(1, &motion_arrow.bind_group, &[]);
            render_pass.set_bind_group(3, &motion_arrow.group_bind_group, &[]);
            render_pass.set_vertex_buffer(0, motion_arrow.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                motion_arrow.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..motion_arrow.indices.len() as u32, 0, 0..1);
        }
    }

    // draw resize handles (always on top)
    for resize_handle in editor_lock.resize_handles.iter() {
        if editor_lock.dragging_polygon.is_some() ||
                editor_lock.dragging_text.is_some() ||
                editor_lock.dragging_video.is_some() ||
                editor_lock.dragging_image.is_some() {
            resize_handle.polygon.transform.update_uniform_buffer(
                &queue,
                &camera.window_size,
            );
        }

        render_pass.set_bind_group(1, &resize_handle.polygon.bind_group, &[]);
        render_pass.set_bind_group(3, &resize_handle.polygon.group_bind_group, &[]);
        render_pass.set_vertex_buffer(0, resize_handle.polygon.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            resize_handle.polygon.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..resize_handle.polygon.indices.len() as u32, 0, 0..1);
    }

    if let Some(dot) = &editor_lock.cursor_dot {
        dot.transform
            .update_uniform_buffer(&queue, &camera.window_size);
        render_pass.set_bind_group(1, &dot.bind_group, &[]);
        render_pass.set_bind_group(3, &dot.group_bind_group, &[]);
        render_pass.set_vertex_buffer(0, dot.vertex_buffer.slice(..));
        render_pass
            .set_index_buffer(dot.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..dot.indices.len() as u32, 0, 0..1);
    }

    drop(render_pass); // End the render pass

    Ok(())
}