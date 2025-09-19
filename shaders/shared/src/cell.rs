use crate::Material;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable)]
pub struct Cell {
    pub material: Material,
}
const CELL_UNDEF: Cell = Cell::UNDEFINED;
impl Cell {
    pub const UNDEFINED: Self = Self {
        material: Material::Undefined,
    };

    pub fn new_empty() -> Self {
        Self {
            material: Material::Empty,
        }
    }

    pub fn new_undefined() -> Self {
        Self {
            material: Material::Undefined,
        }
    }

    pub fn new_material(material: Material) -> Self {
        Self { material }
    }
}
unsafe impl bytemuck::Pod for Cell {}
