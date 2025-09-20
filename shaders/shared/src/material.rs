use crate::glam::{Vec4, vec4};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable)]
pub enum Material {
    Undefined,
    Empty,
    Sand,
    Red,
    Green,
    Blue,
}
impl Material {
    pub fn color(&self) -> Vec4 {
        match self {
            Self::Undefined => vec4(1.0, 0.0, 1.0, 1.0),
            Self::Empty => vec4(0.0, 0.0, 0.0, 1.0),
            Self::Sand => vec4(1.0, 1.0, 0.0, 1.0),
            Self::Red => vec4(1.0, 0.0, 0.0, 1.0),
            Self::Green => vec4(0.0, 1.0, 0.0, 1.0),
            Self::Blue => vec4(0.0, 0.0, 1.0, 1.0),
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Material::Empty
    }
}
