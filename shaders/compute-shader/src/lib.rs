#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
//#![deny(warnings)]
#![allow(unused_imports)]

use core::{
    arch::asm,
    ops::{Index, Rem},
};

use shared::{
    glam::{self, IVec2, USizeVec2, UVec2, UVec3, Vec3Swizzles},
    *,
};
use spirv_std::{
    Image,
    image::{Image2d, StorageImage2d},
    macros::{debug_printf, debug_printfln},
    spirv,
};

mod grid;
use grid::Grid;

// TODO: implement way to make sure that cells never move out of the tile (because even with shifting the tiles each frame, the shared memory is still that one tile)
#[inline(always)]
fn cell_update(params: &ShaderParams, mut grid: Grid, _global_pos: Pos, pos: Pos, pos_idx: usize) {
    let cell = grid.get_idx(pos_idx);

    if cell.material == Material::Sand {
        let below_pos = get_pos(pos, Offset::Down);
        if grid.clamp_pos(pos, below_pos) == below_pos {
            let below = grid.get(below_pos);
            if below.material.is_empty() {
                grid.move_cell(pos, below_pos, true);
                return;
            }
        }
        let right_pos = get_pos(pos, Offset::Right);
        if grid.clamp_pos(pos, right_pos) != right_pos {
            return;
        }
        let left_pos = get_pos(pos, Offset::Left);
        if grid.clamp_pos(pos, left_pos) != left_pos {
            return;
        }
        let right = grid.get(right_pos);
        let left = grid.get(left_pos);

        let (diag_neigh_order, horiz_neigh_order) = if params.frame % 2 == 0 {
            ([Offset::DownRight, Offset::DownLeft], [right, left])
        } else {
            ([Offset::DownLeft, Offset::DownRight], [left, right])
        };

        let diag_pos = get_pos(pos, diag_neigh_order[0]);
        if grid.clamp_pos(pos, diag_pos) == diag_pos {
            let diag_neigh = grid.get(diag_pos);
            if horiz_neigh_order[0].material.is_empty() && diag_neigh.material.is_empty() {
                grid.move_cell(pos, diag_pos, true);
                return;
            }
        }
        let diag_pos = get_pos(pos, diag_neigh_order[1]);
        if grid.clamp_pos(pos, diag_pos) == diag_pos {
            let diag_neigh = grid.get(diag_pos);
            if horiz_neigh_order[1].material.is_empty() && diag_neigh.material.is_empty() {
                grid.move_cell(pos, diag_pos, true);
                return;
            }
        }
    }
}

// Must match SIM_TILE_SIZE!!!!
#[spirv(compute(threads(32, 32, 1)))]
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
    let sim_size = USizeVec2::new(params.sim_width as usize, params.sim_height as usize);
    let mut base_pos = wid.xy().as_usizevec2() * SIM_TILE_SIZE;

    // Shift the grid each even frame to remove problems at tile borders
    base_pos += (SIM_TILE_SIZE_VEC / 2) * (params.frame % 2) as usize;

    let local_pos = lid.xy().as_usizevec2();
    let global_pos = (base_pos + local_pos).rem(sim_size);
    let global_idx = global_pos.y * sim_size.x + global_pos.x;
    let local_idx = local_pos.y * SIM_TILE_SIZE + local_pos.x;

    // init shared workgroup memory from global memory
    shared[local_idx] = input[global_idx];
    if base_pos == Pos::new(496, 400) {
        unsafe {
            debug_printfln!(
                "global: (%u, %u)  local: (%u %u)  grid: (%u, %u)",
                global_pos.x as u32,
                global_pos.y as u32,
                local_pos.x as u32,
                local_pos.y as u32,
                base_pos.x as u32,
                base_pos.y as u32,
            )
        };
    }
    let grid_topleft = base_pos;
    let grid = Grid::new(shared, grid_topleft, sim_size);
    spirv_std::arch::workgroup_memory_barrier_with_group_sync();

    if params.time > 1.0 {
        cell_update(params, grid, global_pos, local_pos, local_idx);
    }

    spirv_std::arch::workgroup_memory_barrier_with_group_sync();

    // write back into global memory from shared memory
    output[global_idx] = shared[local_idx];
}
