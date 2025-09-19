#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
//#![deny(warnings)]
#![allow(unused_imports)]

use core::{arch::asm, ops::Index};

use shared::{
    glam::{self, IVec2, USizeVec2, UVec2, UVec3, Vec3Swizzles},
    *,
};
use spirv_std::{
    Image,
    image::{Image2d, StorageImage2d},
    spirv,
};

mod grid;
use grid::Grid;

// TODO: implement way to make sure that cells never move out of the tile (because even with shifting the tiles each frame, the shared memory is still that one tile)
#[inline(always)]
fn cell_update(params: &ShaderParams, mut grid: Grid, pos: Pos, pos_idx: usize) {
    let cell = grid.get_idx(pos_idx);

    if params.time > 1.0 && cell.material == Material::Sand {
        let below_pos = get_pos(pos, Offset::Down);
        if grid.pos_valid(below_pos) {
            let below = grid.get(below_pos);
            if below.material == Material::Empty {
                grid.move_cell(pos, below_pos, true);
                return;
            }
        }
    }
}

// Must match SIM_TILE_SIZE!!!!
#[spirv(compute(threads(16, 16, 1)))]
#[allow(clippy::too_many_arguments)]
pub fn main_cs(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(local_invocation_id)] lid: UVec3,
    #[spirv(workgroup_id)] wid: UVec3,
    #[spirv(push_constant)] params: &ShaderParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] input: &[Cell], // read only current frame. Binding: BIND_SIM_INPUT
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] output: &mut [Cell], // output buffer. Binding: BIND_SIM_OUTPUT
    // #[spirv(descriptor_set = 0, binding = 3, non_readable)] output_image: &Image!(
    //     2D,
    //     format = rgba32f,
    //     sampled = false
    // ), // writable output image. Binding: BIND_SIM_OUTPUT_IMG
    #[spirv(workgroup)] shared: &mut Tile,
) {
    let mut base_pos = wid.xy().as_usizevec2() * SIM_TILE_SIZE;

    // Shift the grid each even frame to remove problems at tile borders
    if params.frame % 2 == 0 {
        base_pos += SIM_TILE_SIZE / 2;
    }
    let sim_size = USizeVec2::new(params.sim_width as usize, params.sim_height as usize);
    let local_pos = lid.xy().as_usizevec2();
    let global_pos = (base_pos + local_pos).min(sim_size - 1);
    let global_idx = global_pos.y * sim_size.x + global_pos.x;
    let local_idx = local_pos.y * SIM_TILE_SIZE + local_pos.x;

    // init shared workgroup memory from global memory
    shared[local_idx] = input[global_idx];
    let grid = Grid::new(shared);
    workgroup_barrier();

    cell_update(params, grid, local_pos, local_idx);

    workgroup_barrier();

    // write back into global memory from shared memory
    output[global_idx] = shared[local_idx];
}

fn workgroup_barrier() {
    //spirv_std::arch::workgroup_memory_barrier();
    spirv_std::arch::workgroup_memory_barrier_with_group_sync();
}
