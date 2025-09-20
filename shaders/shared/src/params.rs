use spirv_std::glam::{USizeVec2, UVec2};

use crate::MouseButtonPressed;

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ShaderParams {
    // window inner size
    pub window_size: UVec2,
    // simulation grid size
    pub sim_size: UVec2,
    // time since app start in secs
    pub time: f32,
    // frame number
    pub frame: u32,

    // cursor in simulation grid space
    pub cursor: UVec2,
    pub drag_start: UVec2,
    pub drag_end: UVec2,
    pub mouse_button_pressed: MouseButtonPressed,
    pub mouse_radius: u32,
}
impl Default for ShaderParams {
    fn default() -> Self {
        Self {
            window_size: UVec2::new(512, 512),
            sim_size: UVec2::new(512, 512),
            time: 0.0,
            frame: 0,
            mouse_radius: 10,
            cursor: UVec2::new(0, 0),
            drag_start: UVec2::new(0, 0),
            drag_end: UVec2::new(0, 0),
            mouse_button_pressed: MouseButtonPressed::None,
        }
    }
}
