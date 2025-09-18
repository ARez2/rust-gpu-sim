#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]
#![allow(unused_imports)]

use glam::UVec3;
use shared::{
    BIND_SIM_INPUT, BIND_SIM_OUTPUT, Cell, Material, SIM_TILE_SIZE, ShaderParams,
    glam::{IVec2, UVec2},
};
use spirv_std::{
    Image, glam,
    image::{Image2d, StorageImage2d},
    spirv,
};

#[spirv(compute(threads(16, 16)))]
#[allow(clippy::too_many_arguments)]
pub fn main_cs(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(local_invocation_id)] lid: UVec3,
    #[spirv(workgroup_id)] _wid: UVec3,
    #[spirv(push_constant)] params: &ShaderParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] input: &[Cell], // read only current frame. Binding: BIND_SIM_INPUT
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] output: &mut [Cell], // output buffer. Binding: BIND_SIM_OUTPUT
    // #[spirv(descriptor_set = 0, binding = 3, non_readable)] output_image: &Image!(
    //     2D,
    //     format = rgba32f,
    //     sampled = false
    // ), // writable output image. Binding: BIND_SIM_OUTPUT_IMG
    #[spirv(workgroup)] shared: &mut [Cell; SIM_TILE_SIZE as usize * SIM_TILE_SIZE as usize],
) {
    let global_idx = gid.y as usize * params.width as usize + gid.x as usize;
    // init shared workgroup memory from global memory
    shared[(lid.y * SIM_TILE_SIZE + lid.x) as usize] = input[global_idx];
    workgroup_barrier();
    let local_idx = (lid.y * SIM_TILE_SIZE + lid.x) as usize;
    let cell = &shared[local_idx];

    // falling sand step
    if cell.material == Material::Sand {
        let below_idx = ((lid.y + 1) * SIM_TILE_SIZE + lid.x) as usize;
        if lid.y as usize + 1 < SIM_TILE_SIZE as usize
            && shared[below_idx].material == Material::Empty
        {
            shared[below_idx] = shared[local_idx];
            shared[local_idx] = Cell::new_empty();
        }
    }
    shared[local_idx] = Cell::new_material(Material::Empty);

    workgroup_barrier();

    // write back into global memory from shared memory
    output[global_idx] = shared[(lid.y * SIM_TILE_SIZE + lid.x) as usize];
    // let color = output[global_idx].material.color();
    // let coord = UVec2::new(gid.x, gid.y);
    // unsafe { output_image.write(coord, color) };
}

fn workgroup_barrier() {
    //spirv_std::arch::workgroup_memory_barrier();
    spirv_std::arch::workgroup_memory_barrier_with_group_sync();
}
