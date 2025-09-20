use std::time::Duration;

use shared::{
    BIND_SHADER_PARAMS_WORKAROUND, BIND_SIM_INPUT, BIND_SIM_OUTPUT, BIND_SIM_OUTPUT_IMG,
    SIM_TILE_SIZE, ShaderParams,
};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer, ComputePipeline,
    Device, PipelineLayout, PushConstantRange, QuerySet, Queue, Sampler,
    ShaderModuleDescriptorPassthrough, Texture, TextureView, util::DeviceExt,
};

use crate::{CompiledShaderModules, load_spirv_module};

pub struct Simulation {
    pub sim_width: u32,
    pub sim_height: u32,
    input_buffer: Buffer,
    output_buffer: Buffer,
    // output_img_tex: Texture,
    // output_img_view: TextureView,
    // output_img_sampler: Sampler,
    emulate_push_constants_with_storage_buffer: bool,
    bind_group_a: BindGroup,
    bind_group_b: BindGroup,
    current_bind_group_a: bool,
    bind_group_layout: BindGroupLayout,
    compute_pipeline: ComputePipeline,
    compute_pipeline_layout: PipelineLayout,
    // if timestamping is enabled, this tuple is:
    // (ts buffer, ts readback buffer, ts queries, ts period)
    timestamp_data: Option<(Buffer, Buffer, QuerySet, f32)>,
}
impl Simulation {
    pub fn new(
        device: &Device,
        queue: &Queue,
        timestamping: bool,
        emulate_push_constants_with_storage_buffer: bool,
        spirv_passthrough: bool,
        push_constant_data: (BindGroupLayoutEntry, &Buffer),
    ) -> Self {
        // #[cfg(not(target_arch = "wasm32"))]
        // let shader_reload = {
        //     let proxy = event_loop.create_proxy();
        //     Some(Box::new(move |res| match proxy.send_event(res) {
        //         Ok(it) => it,
        //         // ShaderModuleDescriptor is not `Debug`, so can't use unwrap/expect
        //         Err(_err) => panic!("Event loop dead"),
        //     }))
        // };
        let module = crate::compile_and_watch("compute-shader", spirv_passthrough, None);
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

        let sim_width = 512;
        let sim_height = 512;

        let mut bind_group_layout_entries = vec![];
        let mut bind_group_entries_a = vec![];
        if emulate_push_constants_with_storage_buffer {
            bind_group_layout_entries.push(push_constant_data.0);
            bind_group_entries_a.push(wgpu::BindGroupEntry {
                binding: BIND_SHADER_PARAMS_WORKAROUND,
                resource: push_constant_data.1.as_entire_binding(),
            });
        };
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: BIND_SIM_INPUT,
            count: None,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Storage { read_only: true },
            },
        });
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: BIND_SIM_OUTPUT,
            count: None,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Storage { read_only: false },
            },
        });

        // let output_img_format = wgpu::TextureFormat::Rgba32Float;
        // let output_img_tex = device.create_texture(&wgpu::TextureDescriptor {
        //     size: wgpu::Extent3d {
        //         width: sim_width,
        //         height: sim_height,
        //         depth_or_array_layers: 1,
        //     },
        //     view_formats: &[output_img_format],
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: output_img_format,
        //     usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING, // | wgpu::TextureUsages::COPY_SRC???
        //     label: Some("Output image"),
        // });
        // let output_img_view = output_img_tex.create_view(&wgpu::TextureViewDescriptor::default());
        // let output_img_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     ..Default::default()
        // });
        // bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
        //     binding: BIND_SIM_OUTPUT_IMG,
        //     visibility: compute_stage,
        //     ty: wgpu::BindingType::StorageTexture {
        //         access: wgpu::StorageTextureAccess::WriteOnly,
        //         format: output_img_format,
        //         view_dimension: wgpu::TextureViewDimension::D2,
        //     },
        //     count: None,
        // });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bind_group_layout_entries,
        });

        const PUSH_CONSTANTS_SIZE: usize = std::mem::size_of::<ShaderParams>();
        let push_constant_ranges = if emulate_push_constants_with_storage_buffer {
            &vec![]
        } else {
            &vec![wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..PUSH_CONSTANTS_SIZE as u32,
            }]
        };
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            compilation_options: Default::default(),
            cache: None,
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main_cs"),
        });

        let mut input = vec![
            shared::Cell::new_material(shared::Material::Empty);
            sim_width as usize * sim_height as usize
        ];
        // for y in 100..150 {
        for x in 0..400 {
            input[(400 * sim_width + x) as usize] =
                shared::Cell::new_material(shared::Material::Sand);
        }
        for y in 0..256 {
            for x in 350..512 {
                input[(y * sim_width + x) as usize] =
                    shared::Cell::new_material(shared::Material::Sand);
            }
        }

        let output = input.clone();

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input buffer"),
            contents: bytemuck::cast_slice(&input),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output buffer"),
            contents: bytemuck::cast_slice(&output),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        assert_eq!(input_buffer.size(), output_buffer.size());
        // Create 2 bind groups to swap at runtime
        let mut bind_group_entries_b = bind_group_entries_a.clone();
        bind_group_entries_a.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_INPUT,
            resource: input_buffer.as_entire_binding(),
        });
        bind_group_entries_a.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_OUTPUT,
            resource: output_buffer.as_entire_binding(),
        });
        bind_group_entries_b.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_INPUT,
            resource: output_buffer.as_entire_binding(),
        });
        bind_group_entries_b.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_OUTPUT,
            resource: input_buffer.as_entire_binding(),
        });

        // bind_group_entries.push(wgpu::BindGroupEntry {
        //     binding: BIND_SIM_OUTPUT_IMG,
        //     resource: wgpu::BindingResource::TextureView(&output_img_view),
        // });
        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group A"),
            layout: &bind_group_layout,
            entries: &bind_group_entries_a,
        });
        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group B"),
            layout: &bind_group_layout,
            entries: &bind_group_entries_b,
        });

        let timestamp_data = if timestamping {
            let timestamp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timestamps buffer"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let timestamp_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            timestamp_readback_buffer.unmap();

            let timestamp_queries = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                count: 2,
                ty: wgpu::QueryType::Timestamp,
            });
            Some((
                timestamp_buffer,
                timestamp_readback_buffer,
                timestamp_queries,
                queue.get_timestamp_period(),
            ))
        } else {
            None
        };

        Self {
            sim_width,
            sim_height,
            input_buffer,
            output_buffer,
            // output_img_tex,
            // output_img_view,
            // output_img_sampler,
            emulate_push_constants_with_storage_buffer,
            bind_group_a,
            bind_group_b,
            current_bind_group_a: true,
            bind_group_layout,
            compute_pipeline,
            compute_pipeline_layout: pipeline_layout,
            timestamp_data,
        }
    }

    pub fn execute(&mut self, device: &Device, queue: &Queue, params: &ShaderParams) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());

            self.current_bind_group_a = !self.current_bind_group_a;
            if self.current_bind_group_a {
                cpass.set_bind_group(0, &self.bind_group_a, &[]);
            } else {
                cpass.set_bind_group(0, &self.bind_group_b, &[]);
            }

            cpass.set_pipeline(&self.compute_pipeline);
            if !self.emulate_push_constants_with_storage_buffer {
                let (push_constant_offset, push_constant_bytes) = (0, bytemuck::bytes_of(params));
                cpass.set_push_constants(push_constant_offset, push_constant_bytes);
            }
            if let Some((_ts_buffer, _ts_readback_buffer, ts_queries, _ts_period)) =
                &self.timestamp_data
            {
                cpass.write_timestamp(ts_queries, 0);
            }

            let tile = SIM_TILE_SIZE as f32;
            let num_workgroups_x = (self.sim_width as f32 / tile).ceil() as u32;
            let num_workgroups_y = (self.sim_height as f32 / tile).ceil() as u32;
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
            if let Some((_ts_buffer, _ts_readback_buffer, ts_queries, _ts_period)) =
                &self.timestamp_data
            {
                cpass.write_timestamp(ts_queries, 1);
            }
        }

        if let Some((ts_buffer, ts_readback_buffer, ts_queries, _ts_period)) = &self.timestamp_data
        {
            encoder.resolve_query_set(ts_queries, 0..2, ts_buffer, 0);
            encoder.copy_buffer_to_buffer(ts_buffer, 0, ts_readback_buffer, 0, ts_buffer.size());
        }

        // if self.current_bind_group_a {
        //     encoder.copy_buffer_to_buffer(
        //         &self.output_buffer,
        //         0,
        //         &self.input_buffer,
        //         0,
        //         self.output_buffer.size(),
        //     );
        // } else {
        //     encoder.copy_buffer_to_buffer(
        //         &self.input_buffer,
        //         0,
        //         &self.output_buffer,
        //         0,
        //         self.output_buffer.size(),
        //     );
        // }

        queue.submit(Some(encoder.finish()));
        if let Some((_ts_buffer, ts_readback_buffer, _ts_queries, _ts_period)) =
            &self.timestamp_data
        {
            let timestamp_slice = ts_readback_buffer.slice(..);
            timestamp_slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
        }
        // NOTE(eddyb) `poll` should return only after the above callbacks fire
        // (see also https://github.com/gfx-rs/wgpu/pull/2698 for more details).
        device.poll(wgpu::PollType::Wait).unwrap();

        if let Some((_ts_buffer, ts_readback_buffer, _ts_queries, ts_period)) = &self.timestamp_data
        {
            let timing_data = ts_readback_buffer.slice(..).get_mapped_range();
            let timings = timing_data
                .chunks_exact(8)
                .map(|b| u64::from_ne_bytes(b.try_into().unwrap()))
                .collect::<Vec<_>>();

            println!(
                "Took: {:?}",
                Duration::from_nanos(
                    ((timings[1] - timings[0]) as f64 * f64::from(*ts_period)) as u64
                )
            );
            drop(timing_data);
            ts_readback_buffer.unmap();
        }
    }

    /// Returns the most recent buffer thats been written to
    pub fn current_buffer(&self) -> &Buffer {
        println!("{}", self.current_bind_group_a);
        if self.current_bind_group_a {
            &self.output_buffer
        } else {
            &self.input_buffer
        }
    }
}
