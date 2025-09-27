#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
//#![deny(warnings)]

#[allow(unused_imports)]
use glam::{Vec2, Vec4, vec2, vec3, vec4};
use shared::*;
use spirv_std::spirv;

// // Cheap mipmap blur from Michael Moroz
// // https://www.shadertoy.com/view/WsVGWV
// float weight(float t, float log2radius, float gamma)
// {
//     return exp(-gamma*pow(log2radius-t,2.));
// }

// vec4 sampleBlurred(sampler2D ch, vec2 uv, float radius, float gamma)
// {
//     vec4 pix = vec4(0.);
//     float norm = 0.;
//     // Weighted integration over mipmap levels
//     for(float i = 0.; i < 10.; i += 1.0)
//     {
//         float k = weight(i, log2(radius), gamma);
//         pix += k*texture(ch, uv, i);
//         norm += k;
//     }

//     return pix / norm;
// }

// float occ = sampleBlurred(iChannel0, fragCoord / RES, 16.0, 0.5).y;
//     occ = saturate((1.0 - occ) / 0.25);

//     col *= 0.2 + 0.8 * occ;

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
    if cell.material.is_empty() && (cell.color.0 > 0.0 || cell.color.1 > 0.0) {
        *output = vec4(1.0, 0.0, 0.0, 1.0);
    }
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
