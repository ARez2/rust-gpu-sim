#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
//#![deny(warnings)]

use core::f32::consts::PI;
use glam::{Mat2, Vec2, Vec3, Vec4, Vec4Swizzles, vec2, vec3, vec4};
use shared::*;
use spirv_std::spirv;

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] params: &ShaderParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] sim_state: &[Cell],
    output: &mut Vec4,
) {
    let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
    // Normalize to [0,1] range across the window
    let uv = frag_coord / params.window_size.as_vec2();

    // Scale into simulation space
    let mut sim_pos = (uv * params.sim_size.as_vec2()).floor().as_uvec2();

    // Clamp to valid indices
    sim_pos = sim_pos.min(params.sim_size - 1);

    // Fetch cell from 1D array
    let idx = (sim_pos.y * params.sim_size.x + sim_pos.x) as usize;
    let cell = sim_state[idx];

    // Output its color
    *output = cell.material.color();
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
