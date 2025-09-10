use std::sync::{Arc, Mutex};
use stunts_engine::camera::{Camera, CameraBinding};
use stunts_engine::dot::RingDot;
use stunts_engine::editor::{Editor, Point, Viewport, WindowSize, WindowSizeShader};
use stunts_engine::vertex::Vertex;
use stunts_engine::gpu_resources::GpuResources;
use wgpu::util::DeviceExt;
use stunts_engine::polygon::{Polygon, Stroke};
use stunts_engine::editor::rgb_to_wgpu;
use uuid::Uuid;

pub fn init_pipeline(
    viewport: Arc<Mutex<Viewport>>,
    editor: Arc<Mutex<Editor>>, 
    gpu_resources: Arc<stunts_engine::gpu_resources::GpuResources>,
) {
    println!("Initializing Stunts Native Pipeline...");

    // Get window size from viewport
    let window_size = {
        let vp = viewport.lock().unwrap();
        WindowSize {
            width: vp.width as u32,
            height: vp.height as u32,
        }
    };

    println!("Initializing pipeline...");

    let mut editor_lock = editor.lock().unwrap();

    let camera = Camera::new(window_size);
    let camera_binding = CameraBinding::new(&gpu_resources.device);

    editor_lock.camera = Some(camera);
    editor_lock.camera_binding = Some(camera_binding);

    let sampler = gpu_resources
        .device
        .create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

    // gpu_helper.recreate_depth_view(&gpu_resources, window_size.width, window_size.height);

    let depth_stencil_state = wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24Plus,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };

    let depth_texture = gpu_resources
        .device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            // width: window_size.width.clone(),
            // height: window_size.height.clone(),
            width: window_size.width,
            height: window_size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1, // used in a multisampled environment
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("Stunts Engine Depth Texture"),
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let camera_binding_ref = editor_lock
        .camera_binding
        .as_ref()
        .expect("Couldn't get camera binding");

    let model_bind_group_layout = gpu_resources.device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Existing uniform buffer binding
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture binding
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler binding
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("model_bind_group_layout"),
        },
    );

    let model_bind_group_layout = Arc::new(model_bind_group_layout);

    let group_bind_group_layout = gpu_resources.device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Existing uniform buffer binding
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("group_bind_group_layout"),
        },
    );

    let group_bind_group_layout = Arc::new(group_bind_group_layout);

    let window_size_buffer =
        gpu_resources
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Window Size Buffer"),
                contents: bytemuck::cast_slice(&[WindowSizeShader {
                    width: window_size.width as f32,
                    height: window_size.height as f32,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

    let window_size_buffer = Arc::new(window_size_buffer);

    let window_size_bind_group_layout = gpu_resources.device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        },
    );

    let window_size_bind_group_layout = Arc::new(window_size_bind_group_layout);

    let window_size_bind_group =
        gpu_resources
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &window_size_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: window_size_buffer.as_entire_binding(),
                }],
                label: None,
            });

    // Define the layouts
    let pipeline_layout =
        gpu_resources
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                // bind_group_layouts: &[&bind_group_layout],
                bind_group_layouts: &[
                    &camera_binding_ref.bind_group_layout,
                    &model_bind_group_layout,
                    &window_size_bind_group_layout,
                    &group_bind_group_layout,
                ], // No bind group layouts
                push_constant_ranges: &[],
            });

    // Load the shaders
    let shader_module_vert_primary =
        gpu_resources
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Primary Vert Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/vert_primary.wgsl").into(),
                ),
            });

    let shader_module_frag_primary =
        gpu_resources
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Primary Frag Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/frag_primary.wgsl").into(),
                ),
            });

    // let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // hardcode for now
    let swapchain_format = wgpu::TextureFormat::Rgba8Unorm;

    // Configure the render pipeline
    let render_pipeline =
        gpu_resources
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Common Vector Primary Render Pipeline"),
                layout: Some(&pipeline_layout),
                multiview: None,
                // cache: None,
                vertex: wgpu::VertexState {
                    module: &shader_module_vert_primary,
                    entry_point: "vs_main", // name of the entry point in your vertex shader
                    buffers: &[Vertex::desc()], // Make sure your Vertex::desc() matches your vertex structure
                    // compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module_frag_primary,
                    entry_point: "fs_main", // name of the entry point in your fragment shader
                    targets: &[Some(wgpu::ColorTargetState {
                        format: swapchain_format,
                        // blend: Some(wgpu::BlendState::REPLACE),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    // compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                // primitive: wgpu::PrimitiveState::default(),
                // depth_stencil: None,
                // multisample: wgpu::MultisampleState::default(),
                primitive: wgpu::PrimitiveState {
                    conservative: false,
                    topology: wgpu::PrimitiveTopology::TriangleList, // how vertices are assembled into geometric primitives
                    // strip_index_format: Some(wgpu::IndexFormat::Uint32),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // Counter-clockwise is considered the front face
                    // none cull_mode
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Other properties such as conservative rasterization can be set here
                    unclipped_depth: false,
                },
                depth_stencil: Some(depth_stencil_state), // Optional, only if you are using depth testing
                multisample: wgpu::MultisampleState {
                    // count: 4, // effect performance
                    count: 1, // decreases visual fidelity
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

    // Store the render pipeline in the editor
    editor_lock.render_pipeline = Some(Arc::new(render_pipeline));

    println!("Initialized...");

    let canvas_polygon = Polygon::new(
        &window_size,
        &gpu_resources.device,
        &gpu_resources.queue,
        &model_bind_group_layout,
        &group_bind_group_layout,
        &camera,
        vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 1.0 },
        ],
        (1000.0 as f32, 600.0 as f32),
        Point { x: 500.0, y: 350.0 },
        0.0,
        0.0,
        [0.8, 0.8, 0.8, 1.0],
        Stroke {
            thickness: 0.0,
            fill: rgb_to_wgpu(0, 0, 0, 1.0),
        },
        0.0,
        -89, // camera far is -100
        "canvas_background".to_string(),
        Uuid::new_v4(),
        Uuid::nil(),
    );

    editor_lock.static_polygons.push(canvas_polygon);

    let cursor_ring_dot = RingDot::new(
        &gpu_resources.device,
        &gpu_resources.queue,
        &model_bind_group_layout,
        &group_bind_group_layout,
        &window_size,
        Point { x: 600.0, y: 300.0 },
        stunts_engine::editor::rgb_to_wgpu(250, 20, 10, 255.0 / 2.0),
        &camera,
    );

    editor_lock.cursor_dot = Some(cursor_ring_dot);
    editor_lock.gpu_resources = Some(Arc::clone(&gpu_resources));
    editor_lock.model_bind_group_layout = Some(model_bind_group_layout);
    editor_lock.group_bind_group_layout = Some(group_bind_group_layout);
    editor_lock.window_size_bind_group = Some(window_size_bind_group);
    editor_lock.window_size_bind_group_layout = Some(window_size_bind_group_layout);
    editor_lock.window_size_buffer = Some(window_size_buffer);
    editor_lock.depth_view = Some(depth_view);

    editor_lock.update_camera_binding();
}