use crate::Material;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, bytemuck::Zeroable)]
pub struct Cell {
    pub material: Material,
    pub color: (f32, f32, f32, f32),
    pad1: u32,
    pad2: u32,
    pad3: u32,
}
impl Cell {
    pub fn new_empty() -> Self {
        Self {
            material: Material::Empty,
            ..Default::default()
        }
    }

    pub fn new_undefined() -> Self {
        Self {
            material: Material::Undefined,
            ..Default::default()
        }
    }

    pub fn new_material(material: Material) -> Self {
        Self {
            material,
            ..Default::default()
        }
    }
}
unsafe impl bytemuck::Pod for Cell {}
