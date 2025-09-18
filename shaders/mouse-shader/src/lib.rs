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
    let ifrag_coord = frag_coord.as_uvec2();
    let y = ifrag_coord.y.min(params.sim_height - 1);
    let x = ifrag_coord.x.min(params.sim_width - 1);
    let cell = sim_state[(y * params.sim_width + x) as usize];
    let uv = frag_coord / vec2(1920.0, 1080.0);
    *output = cell.material.color();
    //*output = vec4(frag_coord.x / 1280.0, frag_coord.y / 720.0, 0.0, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
