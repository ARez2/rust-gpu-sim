//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>

#![cfg_attr(target_arch = "spirv", no_std)]

pub use spirv_std::glam;
use spirv_std::glam::USizeVec2;

// this binding is used when the SSBO workaround for push constants is used
pub const BIND_SHADER_PARAMS_WORKAROUND: u32 = 0;
pub const BIND_SIM_INPUT: u32 = 1;
pub const BIND_SIM_OUTPUT: u32 = 2;
pub const BIND_SIM_OUTPUT_IMG: u32 = 3;

pub const BIND_FRAG_TEX: u32 = 1;
pub const BIND_FRAG_SAMPLER: u32 = 2;

pub const SIM_TILE_SIZE: usize = 32;
pub const SIM_TILE_SIZE_VEC: USizeVec2 = USizeVec2::new(SIM_TILE_SIZE, SIM_TILE_SIZE);

pub type Tile = [Cell; SIM_TILE_SIZE * SIM_TILE_SIZE];
pub type Pos = USizeVec2;

mod helpers;
pub use helpers::*;

mod params;
pub use params::*;

mod material;
pub use material::*;

mod cell;
pub use cell::*;

/// Converts a 2D pos to a 1D index
#[inline(always)]
pub fn pos_to_idx(pos: Pos) -> usize {
    pos.y * SIM_TILE_SIZE + pos.x
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Offset {
    Up,
    Left,
    Right,
    UpRight,
    DownRight,
    DownLeft,
    UpLeft,
    Down,
}
impl Offset {
    pub fn tuple(&self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::UpRight => (1, -1),
            Self::DownRight => (1, 1),
            Self::DownLeft => (-1, 1),
            Self::UpLeft => (-1, -1),
            Self::Down => (0, 1),
        }
    }
}

pub fn get_pos(mut pos: Pos, offset: Offset) -> Pos {
    let (x, y) = offset.tuple();
    if x < 0 {
        pos.x -= ((-x) as usize).min(pos.x);
    } else {
        pos.x += x as usize;
    }
    if y < 0 {
        pos.y -= ((-y) as usize).min(pos.y);
    } else {
        pos.y += y as usize;
    }
    pos
}
pub fn get_pos_custom(mut pos: Pos, x: i32, y: i32) -> Pos {
    if x < 0 {
        pos.x -= ((-x) as usize).min(pos.x);
    } else {
        pos.x += x as usize;
    }
    if y < 0 {
        pos.y -= ((-y) as usize).min(pos.y);
    } else {
        pos.y += y as usize;
    }
    pos
}
