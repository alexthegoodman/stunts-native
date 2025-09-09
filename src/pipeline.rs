// #[tokio::main]
async fn init_pipeline() {
    println!("Initializing Stunts...");

    let app = Application::new();

    // Get the primary monitor's size
    let monitor = app.primary_monitor().expect("Couldn't get primary monitor");
    let monitor_size = monitor.size();

    // Calculate a reasonable window size (e.g., 80% of the screen size)
    let window_width = (monitor_size.width.into_integer() as f32 * 0.8) as u32;
    let window_height = (monitor_size.height.into_integer() as f32 * 0.8) as u32;

    println!("Window Size {:?}x{:?}", window_width, window_height);

    let window_size = WindowSize {
        width: window_width,
        height: window_height,
    };

    let mut gpu_helper = Arc::new(Mutex::new(GpuHelper::new()));

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_clonsed2 = Arc::clone(&gpu_helper);
    let gpu_cloned3 = Arc::clone(&gpu_helper);

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    )));

    let mut editor = Arc::new(Mutex::new(init_editor_with_model(viewport.clone())));

    let cloned_viewport = Arc::clone(&viewport);
    let cloned_viewport2 = Arc::clone(&viewport);
    let cloned_viewport3 = Arc::clone(&viewport);

    let cloned = Arc::clone(&editor);
    let cloned2 = Arc::clone(&editor);
    let cloned3 = Arc::clone(&editor);
    let cloned4 = Arc::clone(&editor);
    let cloned5 = Arc::clone(&editor);
    let cloned7 = Arc::clone(&editor);
    let cloned11 = Arc::clone(&editor);
    let cloned12 = Arc::clone(&editor);
    let cloned13 = Arc::clone(&editor);

    let record = Arc::new(Mutex::new(Record::new()));

    let record_2 = Arc::clone(&record);

    let editor_state = Arc::new(Mutex::new(EditorState::new(cloned4, record)));

    let state_2 = Arc::clone(&editor_state);
    let state_3 = Arc::clone(&editor_state);
    let state_4 = Arc::clone(&editor_state);
    let state_5 = Arc::clone(&editor_state);

    {
        let app_handle = app.handle.as_mut().expect("Couldn't get handle");
        let window_handle = app_handle
            .window_handles
            .get_mut(&window_id)
            .expect("Couldn't get window handle");

        // Create and set the render callback
        let render_callback = create_render_callback();

        // window_handle.set_render_callback(render_callback);
        window_handle.set_encode_callback(render_callback);
        // window_handle.window_size = Some(window_size);
        window_handle.window_width = Some(window_size.width);
        window_handle.window_height = Some(window_size.height);

        println!("Ready...");

        // window_handle.user_editor = Some(Box::new(cloned));

        // Receive and store GPU resources
        // match &mut window_handle.paint_state {
        //     PaintState::PendingGpuResources { rx, .. } => {
        if let PaintState::PendingGpuResources { rx, .. } = &mut window_handle.paint_state {
            async {
                let gpu_resources = Arc::new(rx.recv().unwrap().unwrap());

                println!("Initializing pipeline...");

                // let mut editor = cloned11.lock().unwrap();
                let mut editor = cloned5.lock().unwrap();

                let camera = Camera::new(window_size);
                let camera_binding = CameraBinding::new(&gpu_resources.device);

                editor.camera = Some(camera);
                editor.camera_binding = Some(camera_binding);

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

                gpu_cloned.lock().unwrap().recreate_depth_view(
                    &gpu_resources,
                    window_size.width,
                    window_size.height,
                );

                let depth_stencil_state = wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                };

                let camera_binding = editor
                    .camera_binding
                    .as_ref()
                    .expect("Couldn't get camera binding");

                // let model_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                //     &wgpu::BindGroupLayoutDescriptor {
                //         entries: &[wgpu::BindGroupLayoutEntry {
                //             binding: 0,
                //             visibility: wgpu::ShaderStages::VERTEX,
                //             ty: wgpu::BindingType::Buffer {
                //                 ty: wgpu::BufferBindingType::Uniform,
                //                 has_dynamic_offset: false,
                //                 min_binding_size: None,
                //             },
                //             count: None,
                //         }],
                //         label: Some("model_bind_group_layout"),
                //     },
                // );

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
                                &camera_binding.bind_group_layout,
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

                // let swapchain_capabilities = gpu_resources
                //     .surface
                //     .get_capabilities(&gpu_resources.adapter);
                // let swapchain_format = swapchain_capabilities.formats[0]; // Choosing the first available format
                let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // hardcode for now - actually must match common-floem's

                // Configure the render pipeline
                let render_pipeline =
                    gpu_resources
                        .device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Common Vector Primary Render Pipeline"),
                            layout: Some(&pipeline_layout),
                            multiview: None,
                            cache: None,
                            vertex: wgpu::VertexState {
                                module: &shader_module_vert_primary,
                                entry_point: "vs_main", // name of the entry point in your vertex shader
                                buffers: &[Vertex::desc()], // Make sure your Vertex::desc() matches your vertex structure
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                                count: 4, // effect performance
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                        });

                // window_handle.render_pipeline = Some(render_pipeline);
                // window_handle.depth_view = gpu_helper.depth_view;

                println!("Initialized...");

                // let canvas_polygon = Polygon::new(
                //     &window_size,
                //     &gpu_resources.device,
                //     &gpu_resources.queue,
                //     &model_bind_group_layout,
                //     &group_bind_group_layout,
                //     &camera,
                //     vec![
                //         Point { x: 0.0, y: 0.0 },
                //         Point { x: 1.0, y: 0.0 },
                //         Point { x: 1.0, y: 1.0 },
                //         Point { x: 0.0, y: 1.0 },
                //     ],
                //     (800.0 as f32, 450.0 as f32),
                //     Point { x: 400.0, y: 225.0 },
                //     0.0,
                //     0.0,
                //     [0.8, 0.8, 0.8, 1.0],
                //     Stroke {
                //         thickness: 0.0,
                //         fill: rgb_to_wgpu(0, 0, 0, 1.0),
                //     },
                //     0.0,
                //     -89, // camera far is -100
                //     "canvas_background".to_string(),
                //     Uuid::new_v4(),
                //     Uuid::nil(),
                // );

                // editor.static_polygons.push(canvas_polygon);

                let cursor_ring_dot = RingDot::new(
                    &gpu_resources.device,
                    &gpu_resources.queue,
                    &model_bind_group_layout,
                    &group_bind_group_layout,
                    &window_size,
                    Point { x: 600.0, y: 300.0 },
                    rgb_to_wgpu(250, 20, 10, 255.0 / 2.0),
                    &camera,
                );

                editor.cursor_dot = Some(cursor_ring_dot);

                window_handle.handle_cursor_moved = handle_cursor_moved(
                    cloned2.clone(),
                    gpu_resources.clone(),
                    cloned_viewport.clone(),
                );
                window_handle.handle_mouse_input = handle_mouse_input(
                    state_4.clone(),
                    cloned3.clone(),
                    gpu_resources.clone(),
                    cloned_viewport2.clone(),
                    record_2.clone(),
                );
                window_handle.handle_window_resized = handle_window_resize(
                    cloned7,
                    gpu_resources.clone(),
                    gpu_cloned3,
                    cloned_viewport3.clone(),
                );
                window_handle.handle_mouse_wheel =
                    handle_mouse_wheel(cloned11, gpu_resources.clone(), cloned_viewport3.clone());
                window_handle.handle_modifiers_changed = handle_modifiers_changed(
                    state_3,
                    gpu_resources.clone(),
                    cloned_viewport3.clone(),
                );
                window_handle.handle_keyboard_input =
                    handle_keyboard_input(state_2, gpu_resources.clone(), cloned_viewport3.clone());

                gpu_clonsed2.lock().unwrap().gpu_resources = Some(Arc::clone(&gpu_resources));
                editor.gpu_resources = Some(Arc::clone(&gpu_resources));
                editor.model_bind_group_layout = Some(model_bind_group_layout);
                editor.group_bind_group_layout = Some(group_bind_group_layout);
                editor.window_size_bind_group = Some(window_size_bind_group);
                editor.window_size_bind_group_layout = Some(window_size_bind_group_layout);
                editor.window_size_buffer = Some(window_size_buffer);
                window_handle.gpu_resources = Some(gpu_resources);
                // window_handle.gpu_helper = Some(gpu_clonsed2);
                editor.window = window_handle.window.clone();
                window_handle.engine_handle = Some(EngineHandle {
                    render_pipeline: Some(render_pipeline),
                    user_editor: Some(Box::new(cloned)),
                    gpu_helper: Some(gpu_cloned),
                    depth_view: None,
                });

                editor.update_camera_binding();
            }
            .await;
        }
        //     PaintState::Initialized { .. } => {
        //         println!("Renderer is already initialized");
        //     }
        // }
    }

}