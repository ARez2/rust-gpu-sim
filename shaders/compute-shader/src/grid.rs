use core::ops::Rem;
use shared::{
    glam::{IVec2, UVec2},
    *,
};
use spirv_std::macros::debug_printfln;

pub struct Grid<'a> {
    tile: &'a mut Tile,
    /// top left of the shared workgroup tile
    topleft: Pos,
    sim_size: UVec2,
}
#[allow(dead_code)]
impl<'a> Grid<'a> {
    pub fn new(tile: &'a mut Tile, topleft: Pos, sim_size: UVec2) -> Self {
        Self {
            tile,
            topleft,
            sim_size,
        }
    }

    pub fn clamp_pos(&self, own_pos: Pos, pos: Pos) -> Pos {
        let local_clamp = pos.min(SIM_TILE_SIZE_VEC - 1);

        let delta = pos.as_ivec2() - own_pos.as_ivec2();
        let own_global_pos =
            (self.topleft.as_ivec2() + own_pos.as_ivec2()).rem_euclid(self.sim_size.as_ivec2());
        let cell_global_pos = own_global_pos + delta;

        let cell_global_pos_clamped =
            cell_global_pos.clamp(IVec2::ZERO, (self.sim_size - 1).as_ivec2());

        if cell_global_pos_clamped == cell_global_pos {
            local_clamp
        } else {
            // Compute correction in signed space, then clamp to [0, TILE-1]
            let diff = cell_global_pos - cell_global_pos_clamped;
            let corrected = (local_clamp.as_ivec2() - diff)
                .clamp(IVec2::ZERO, (SIM_TILE_SIZE_VEC - 1).as_ivec2());
            corrected.as_uvec2()
        }
    }

    /// Returns a reference to a cell in the grid without validity checks.
    #[inline(always)]
    pub fn get(&self, pos: Pos) -> &Cell {
        &self.tile[pos_to_idx(pos)]
    }

    /// Returns a reference to a cell in the grid without validity checks.
    #[inline(always)]
    pub fn get_idx(&self, idx: usize) -> &Cell {
        &self.tile[idx]
    }

    /// Returns a mutable reference to a cell in the grid without validity checks.
    #[inline(always)]
    pub fn get_idx_mut(&mut self, idx: usize) -> &mut Cell {
        &mut self.tile[idx]
    }

    /// Moves a [`Cell`] from_pos to_pos. If swap == true, it will swap with the target
    /// position, instead of replacing it. This is faster and can be used for empty target cells.
    pub fn move_cell(&mut self, from_pos: Pos, to_pos: Pos, swap: bool) {
        // spirv_std::arch::workgroup_memory_barrier();
        if from_pos == to_pos {
            return;
        }

        let to_idx = pos_to_idx(to_pos);
        let from_idx = pos_to_idx(from_pos);
        if swap {
            #[allow(clippy::manual_swap)]
            {
                let tmp = self.tile[to_idx];
                self.tile[to_idx] = self.tile[from_idx];
                self.tile[from_idx] = tmp;
            }
        } else {
            self.tile[to_idx] = self.tile[from_idx];
            self.tile[from_idx] = Cell::new_empty();
        }
    }

    pub fn set_cell(&mut self, pos: Pos, cell: Cell) {
        self.tile[pos_to_idx(pos)] = cell;
    }
}
