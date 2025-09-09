use gui_core::{App, Element};
use gui_core::widgets::canvas::canvas;
use gui_core::widgets::container::container;
use vello::peniko::Color;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine, ExternalResource};
use wgpu::{Device, Queue, Buffer, RenderPipeline, VertexBufferLayout, BufferUsages, VertexAttribute, VertexFormat, VertexStepMode};
use wgpu::util::DeviceExt;

// TODO: replace from here through create_gpu_resources to integrate with pipeline.rs and make sure not to recreate pipeline on every frame

// Vertex structure for our custom triangle
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ]
        }
    }
}

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5], color: [1.0, 0.0, 0.0] },   // Top vertex - Red
    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] }, // Bottom left - Green  
    Vertex { position: [0.5, -0.5], color: [0.0, 0.0, 1.0] },  // Bottom right - Blue
];

const VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}
"#;

const FRAGMENT_SHADER: &str = r#"
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}
"#;

fn create_gpu_resources(device: &Device) -> (Buffer, RenderPipeline) {
    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
        usage: BufferUsages::VERTEX,
    });

    // Create shaders
    let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
    });

    let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(FRAGMENT_SHADER.into()),
    });

    // Create render pipeline
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    
    (vertex_buffer, render_pipeline)
}

pub fn create_advanced_canvas_app() -> Result<Element, Box<dyn std::error::Error>> {
    // Create an advanced canvas that combines Vello graphics with custom vertex rendering
    let advanced_canvas = canvas()
        .with_size(600.0, 400.0)
        .with_position(50.0, 50.0)
        .with_render_func(|scene: &mut Scene, _device: &Device, _queue: &Queue, x, y, width, height| {
            // Part 1: Vello high-level graphics
            // Draw a blue circle using Vello
            let circle_center = vello::kurbo::Point::new((x + width * 0.75) as f64, (y + height * 0.25) as f64);
            let circle = Circle::new(circle_center, 40.0);
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::BLUE,
                None,
                &circle,
            );
            
            // Draw a rounded rectangle using Vello
            let rect = RoundedRect::new(
                (x + width * 0.05) as f64,
                (y + height * 0.7) as f64,
                (x + width * 0.4) as f64,
                (y + height * 0.95) as f64,
                12.0
            );
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::PURPLE,
                None,
                &rect,
            );
            
            Ok(())
        })
        .with_shared_encoder_render_func(|device: &Device, _queue: &Queue, encoder: &mut wgpu::CommandEncoder, _external_resources: &[vello::ExternalResource], _x: f32, _y: f32, _width: f32, _height: f32| {
            // Part 2: Custom vertex rendering using shared encoder - NOW IT WORKS!
            // TODO: bad idea to recreate pipeline on every frame, maybe wont work
            let (vertex_buffer, render_pipeline) = create_gpu_resources(device);
            
            // Get the texture view from external resources
            let texture_view = if let Some(vello::ExternalResource::Image(_proxy, texture_view)) = _external_resources.first() {
                texture_view
            } else {
                return Err("No texture view found in external resources".into());
            };
            
            {
                // Create render pass using Vello's shared encoder - this is the key fix!
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Triangle Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // LOAD existing content (don't clear Vello's background)
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                // // Add scissor clipping to canvas bounds
                // render_pass.set_scissor_rect(
                //     _x as u32,
                //     _y as u32,
                //     _width as u32,
                //     _height as u32
                // );

                // Set a viewport that matches your canvas bounds:
                render_pass.set_viewport(
                    _x,           // x offset
                    _y,           // y offset
                    _width,       // width
                    _height,      // height
                    0.0,          // min_depth
                    1.0           // max_depth
                );
                
                // Set up the render pipeline and vertex buffer
                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                
                // Draw the triangle! 3 vertices, 1 instance
                render_pass.draw(0..3, 0..1);
            }
            
            // No need to submit - Vello will submit the shared encoder!
            // println!("Triangle rendered using shared encoder at position ({}, {}) with size {}x{}", _x, _y, _width, _height);
            Ok(())
        });

    // Create a container to hold our advanced canvas
    let root = container()
        .with_size(800.0, 600.0)
        // .with_background_color(Color::rgba8(200, 200, 200, 255))
        .with_child(Element::new_widget(Box::new(advanced_canvas)))
        .into_container_element();

    Ok(root)

    // let app = App::new()
    //     .with_title("Advanced Canvas - Vello + Custom Rendering".to_string())?
    //     .with_inner_size([800, 600])?
    //     .with_root(root)?;

    // Ok(app)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test] 
//     fn test_advanced_canvas_app_creation() {
//         let app_result = create_advanced_canvas_app();
//         assert!(app_result.is_ok());
//     }
// }