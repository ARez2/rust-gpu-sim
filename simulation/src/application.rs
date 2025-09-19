use std::{error::Error, time::Instant};

use futures::executor::block_on;
use ouroboros::self_referencing;
use shared::{BIND_SHADER_PARAMS_WORKAROUND, ShaderParams};
use wgpu::{BindGroup, Buffer, Device, InstanceDescriptor, ShaderModuleDescriptorPassthrough};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::NamedKey,
    window::{WindowAttributes, WindowId},
};

use crate::{load_spirv_module, simulation::Simulation};

#[self_referencing]
struct WindowSurface {
    window: Box<winit::window::Window>,
    #[borrows(window)]
    #[covariant]
    surface: wgpu::Surface<'this>,
}

pub struct Application {
    device: Option<Device>,
    queue: Option<wgpu::Queue>,
    window_surface: Option<WindowSurface>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    shader_module: Option<wgpu::ShaderModule>,
    bind_group: Option<BindGroup>,
    push_constants_ssbo_workaround: Option<Buffer>,
    close_requested: bool,
    start: Instant,
    simulation: Option<Simulation>,
    params: ShaderParams,
    mouse_button_press_since_last_frame: u32,
}
impl Default for Application {
    fn default() -> Self {
        Self {
            device: None,
            queue: None,
            window_surface: None,
            config: None,
            render_pipeline: None,
            shader_module: None,
            bind_group: None,
            push_constants_ssbo_workaround: None,
            close_requested: false,
            start: Instant::now(),
            params: ShaderParams::default(),
            mouse_button_press_since_last_frame: 0,
            simulation: None,
        }
    }
}
impl Application {
    pub async fn init(
        &mut self,
        emulate_push_constants_with_storage_buffer: bool,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Box<dyn Error>> {
        let window_attributes = WindowAttributes::default()
            .with_title("Rust GPU - wgpu")
            .with_inner_size(LogicalSize::new(512.0, 512.0));
        let window_box = event_loop.create_window(window_attributes)?;
        let mut instance_flags = wgpu::InstanceFlags::default();
        // Turn off validation as the shaders are trusted.
        //instance_flags.remove(wgpu::InstanceFlags::VALIDATION);
        // Disable debugging info to speed things up.
        //instance_flags.remove(wgpu::InstanceFlags::DEBUG);
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            flags: instance_flags,
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let window_surface = WindowSurfaceBuilder {
            window: Box::new(window_box),
            surface_builder: |window| {
                instance
                    .create_surface(window)
                    .expect("Failed to create surface")
            },
        }
        .build();

        let window_size = window_surface.borrow_window().inner_size();
        let surface = window_surface.borrow_surface();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await?;
        let mut required_features = wgpu::Features::PUSH_CONSTANTS;
        // Timestamping may not be supported
        let timestamping = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
            && adapter
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if timestamping {
            required_features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES;
        };

        if adapter
            .features()
            .contains(wgpu::Features::SPIRV_SHADER_PASSTHROUGH)
        {
            required_features |= wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
        }
        let required_limits = wgpu::Limits {
            max_push_constant_size: 256,
            ..Default::default()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits,
                ..Default::default()
            })
            .await?;

        let spirv_passthrough = device
            .features()
            .contains(wgpu::Features::SPIRV_SHADER_PASSTHROUGH);
        // #[cfg(not(target_arch = "wasm32"))]
        // let shader_reload = {
        //     let proxy = event_loop.create_proxy();
        //     Some(Box::new(move |res| match proxy.send_event(res) {
        //         Ok(it) => it,
        //         // ShaderModuleDescriptor is not `Debug`, so can't use unwrap/expect
        //         Err(_err) => panic!("Event loop dead"),
        //     }))
        // };
        let module = crate::compile_and_watch("mouse-shader", spirv_passthrough, None);
        let shader_module = if spirv_passthrough {
            unsafe {
                device.create_shader_module_passthrough(ShaderModuleDescriptorPassthrough::SpirV(
                    module,
                ))
            }
        } else {
            let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::SpirV(source),
            })
        };
        let mut bind_group_layout_entries = vec![];
        let mut bind_group_entries = vec![];
        const PUSH_CONSTANTS_SIZE: usize = std::mem::size_of::<ShaderParams>();
        let stages = wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE;

        let push_constants_ssbo_workaround = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: PUSH_CONSTANTS_SIZE as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ssbo_entry = wgpu::BindGroupLayoutEntry {
            binding: BIND_SHADER_PARAMS_WORKAROUND,
            visibility: stages,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some((PUSH_CONSTANTS_SIZE as u64).try_into().unwrap()),
            },
            count: None,
        };
        let push_constant_ranges = if emulate_push_constants_with_storage_buffer {
            vec![]
        } else {
            vec![wgpu::PushConstantRange {
                stages,
                range: 0..PUSH_CONSTANTS_SIZE as u32,
            }]
        };

        let simulation = Simulation::new(
            &device,
            &queue,
            timestamping,
            emulate_push_constants_with_storage_buffer,
            spirv_passthrough,
            (ssbo_entry, &push_constants_ssbo_workaround),
        );
        if emulate_push_constants_with_storage_buffer {
            bind_group_layout_entries.push(ssbo_entry);
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: BIND_SHADER_PARAMS_WORKAROUND,
                resource: push_constants_ssbo_workaround.as_entire_binding(),
            });
        };
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 1,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Storage { read_only: true },
            },
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bind_group_layout_entries,
        });
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: 1,
            resource: simulation.output_buffer.as_entire_binding(),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &push_constant_ranges,
        });
        let swapchain_format = surface.get_capabilities(&adapter).formats[0];
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("main_vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("main_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: Default::default(),
        };
        surface.configure(&device, &config);

        self.device = Some(device);
        self.queue = Some(queue);
        self.window_surface = Some(window_surface);
        self.config = Some(config);
        self.render_pipeline = Some(render_pipeline);
        self.shader_module = Some(shader_module);
        self.bind_group = Some(bind_group);
        if emulate_push_constants_with_storage_buffer {
            self.push_constants_ssbo_workaround = Some(push_constants_ssbo_workaround);
        }
        self.start = web_time::Instant::now();
        self.params.width = window_size.width;
        self.params.height = window_size.height;
        self.simulation = Some(simulation);
        Ok(())
    }

    pub fn render(&mut self) {
        let window_surface = match &self.window_surface {
            Some(ws) => ws,
            None => return,
        };

        let window = window_surface.borrow_window();
        let current_size = window.inner_size();
        self.params.width = current_size.width;
        self.params.height = current_size.height;
        let surface = window_surface.borrow_surface();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire texture: {:?}", e);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_viewport(
                0.0,
                0.0,
                current_size.width as f32,
                current_size.height as f32,
                0.0,
                1.0,
            );
            let time = self.start.elapsed().as_secs_f32();
            for (i, press_time) in self.params.mouse_button_press_time.iter_mut().enumerate() {
                if (self.mouse_button_press_since_last_frame & (1 << i)) != 0 {
                    *press_time = time;
                }
            }
            self.params.time = time;
            self.params.frame += 1;
            self.mouse_button_press_since_last_frame = 0;
            self.params.mouse_button_pressed = 0;
            rpass.set_bind_group(0, self.bind_group.as_ref(), &[]);
            rpass.set_pipeline(self.render_pipeline.as_ref().unwrap());

            let (push_constant_offset, push_constant_bytes) = (0, bytemuck::bytes_of(&self.params));
            if let Some(buffer) = &self.push_constants_ssbo_workaround {
                queue.write_buffer(buffer, push_constant_offset, push_constant_bytes);
            } else {
                rpass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    push_constant_offset as u32,
                    push_constant_bytes,
                )
            }
            rpass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = block_on(self.init(false, event_loop)) {
            eprintln!("Initialization error: {e}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::Resized(new_size) => {
                if let Some(config) = self.config.as_mut() {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    if let Some(ws) = &self.window_surface {
                        let surface = ws.borrow_surface();
                        if let Some(device) = self.device.as_ref() {
                            surface.configure(device, config);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.params.cursor_x = position.x as f32;
                self.params.cursor_y = position.y as f32;
                if self.params.mouse_button_pressed != 0 {
                    self.params.drag_end_x = self.params.cursor_x;
                    self.params.drag_end_y = self.params.cursor_y;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.params.mouse_button_pressed = 1;
                    self.params.drag_start_x = self.params.cursor_x;
                    self.params.drag_start_y = self.params.cursor_y;
                    self.params.drag_end_x = self.params.cursor_x;
                    self.params.drag_end_y = self.params.cursor_y;
                    //if self.mouse_left_pressed {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                    self.params.drag_end_x = x * 0.1;
                    self.params.drag_end_y = y * 0.1;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == NamedKey::Escape && event.state == ElementState::Pressed {
                    self.close_requested = true;
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(sim) = &mut self.simulation {
                    sim.execute(
                        self.device.as_ref().unwrap(),
                        self.queue.as_ref().unwrap(),
                        &self.params,
                    )
                };
                self.render();
            }
            _ => {}
        }
        if self.close_requested {
            event_loop.exit();
        } else if let Some(ws) = &self.window_surface {
            ws.borrow_window().request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
        } else if let Some(ws) = &self.window_surface {
            ws.borrow_window().request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}
