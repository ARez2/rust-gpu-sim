use std::time::Duration;

use shared::{
    BIND_SHADER_PARAMS_WORKAROUND, BIND_SIM_INPUT, BIND_SIM_OUTPUT, BIND_SIM_OUTPUT_IMG,
    SIM_TILE_SIZE, ShaderParams,
};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer, ComputePipeline,
    Device, PipelineLayout, PushConstantRange, QuerySet, Queue, Sampler, Texture, TextureView,
    util::DeviceExt,
};

use crate::CompiledShaderModules;

pub struct Simulation {
    sim_width: u32,
    sim_height: u32,
    input_buffer: Buffer,
    pub output_buffer: Buffer,
    // output_img_tex: Texture,
    // output_img_view: TextureView,
    // output_img_sampler: Sampler,
    emulate_push_constants_with_storage_buffer: bool,
    bind_group: BindGroup,
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
        options: &crate::Options,
        compiled_shader_modules: &CompiledShaderModules,
        push_constant_data: (BindGroupLayoutEntry, &Buffer, &Vec<PushConstantRange>),
    ) -> Self {
        // FIXME(eddyb) automate this decision by default.
        let module = compiled_shader_modules.spv_module_for_entry_point("compute-shader");
        let module = if options.force_spirv_passthru {
            unsafe {
                device.create_shader_module_passthrough(
                    wgpu::ShaderModuleDescriptorPassthrough::SpirV(module),
                )
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

        const PUSH_CONSTANTS_SIZE: usize = std::mem::size_of::<ShaderParams>();
        let compute_stage = wgpu::ShaderStages::COMPUTE;

        let mut bind_group_layout_entries = vec![];
        let mut bind_group_entries = vec![];
        if options.emulate_push_constants_with_storage_buffer {
            bind_group_layout_entries.push(push_constant_data.0);
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: BIND_SHADER_PARAMS_WORKAROUND,
                resource: push_constant_data.1.as_entire_binding(),
            });
        };
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: BIND_SIM_INPUT,
            count: None,
            visibility: compute_stage,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Storage { read_only: true },
            },
        });
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: BIND_SIM_OUTPUT,
            count: None,
            visibility: compute_stage,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &push_constant_data.2,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            compilation_options: Default::default(),
            cache: None,
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("main_cs"),
        });

        let mut input = vec![
            shared::Cell::new_material(shared::Material::Empty);
            sim_width as usize * sim_height as usize
        ];
        for y in (sim_height - 100)..(sim_height - 50) {
            for x in (sim_width - 100)..(sim_width - 50) {
                input[(y * sim_width + x) as usize] =
                    shared::Cell::new_material(shared::Material::Sand);
            }
        }

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input buffer"),
            contents: bytemuck::cast_slice(&input),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        let output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output buffer"),
            contents: bytemuck::cast_slice(&input),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        assert_eq!(input_buffer.size(), output_buffer.size());

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

        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_INPUT,
            resource: input_buffer.as_entire_binding(),
        });
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: BIND_SIM_OUTPUT,
            resource: output_buffer.as_entire_binding(),
        });
        // bind_group_entries.push(wgpu::BindGroupEntry {
        //     binding: BIND_SIM_OUTPUT_IMG,
        //     resource: wgpu::BindingResource::TextureView(&output_img_view),
        // });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });

        Self {
            sim_width,
            sim_height,
            input_buffer,
            output_buffer,
            // output_img_tex,
            // output_img_view,
            // output_img_sampler,
            emulate_push_constants_with_storage_buffer: options
                .emulate_push_constants_with_storage_buffer,
            bind_group,
            bind_group_layout,
            compute_pipeline,
            compute_pipeline_layout: pipeline_layout,
            timestamp_data,
        }
    }

    pub fn execute(&self, device: &Device, queue: &Queue, params: &ShaderParams) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.compute_pipeline);
            if self.emulate_push_constants_with_storage_buffer {
                let (push_constant_offset, push_constant_bytes) = (0, bytemuck::bytes_of(params));
                cpass.set_push_constants(push_constant_offset, push_constant_bytes);
            }
            if let Some((_ts_buffer, _ts_readback_buffer, ts_queries, _ts_period)) =
                &self.timestamp_data
            {
                cpass.write_timestamp(ts_queries, 0);
            }
            let num_workgroups_x = self.sim_width / SIM_TILE_SIZE as u32;
            let num_workgroups_y = self.sim_height / SIM_TILE_SIZE as u32;
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
            if let Some((_ts_buffer, _ts_readback_buffer, ts_queries, _ts_period)) =
                &self.timestamp_data
            {
                cpass.write_timestamp(ts_queries, 1);
            }
        }

        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &self.input_buffer,
            0,
            self.output_buffer.size(),
        );

        if let Some((ts_buffer, ts_readback_buffer, ts_queries, _ts_period)) = &self.timestamp_data
        {
            encoder.resolve_query_set(ts_queries, 0..2, ts_buffer, 0);
            encoder.copy_buffer_to_buffer(ts_buffer, 0, ts_readback_buffer, 0, ts_buffer.size());
        }

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
}
