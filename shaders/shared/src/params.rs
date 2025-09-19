#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ShaderParams {
    pub width: u32,
    pub height: u32,
    pub sim_width: u32,
    pub sim_height: u32,
    pub time: f32,
    pub frame: u32,

    pub cursor_x: f32,
    pub cursor_y: f32,
    pub drag_start_x: f32,
    pub drag_start_y: f32,
    pub drag_end_x: f32,
    pub drag_end_y: f32,

    /// Bit mask of the pressed buttons (0 = Left, 1 = Middle, 2 = Right).
    pub mouse_button_pressed: u32,

    /// The last time each mouse button (Left, Middle or Right) was pressed,
    /// or `f32::NEG_INFINITY` for buttons which haven't been pressed yet.
    ///
    /// If this is the first frame after the press of some button, that button's
    /// entry in `mouse_button_press_time` will exactly equal `time`.
    pub mouse_button_press_time: [f32; 3],
}
impl Default for ShaderParams {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            sim_width: 512,
            sim_height: 512,
            time: 0.0,
            frame: 0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            drag_end_x: 0.0,
            drag_end_y: 0.0,
            mouse_button_pressed: 0,
            mouse_button_press_time: [f32::NEG_INFINITY; 3],
        }
    }
}
