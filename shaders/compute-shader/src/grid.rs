use shared::*;

pub struct Grid<'a> {
    pub tile: &'a mut Tile,
}
impl<'a> Grid<'a> {
    pub fn new(tile: &'a mut Tile) -> Self {
        Self { tile }
    }

    #[inline(always)]
    pub fn pos_valid(&self, pos: Pos) -> bool {
        let clamped = clamp_pos(pos);
        clamped == pos
    }

    #[inline(always)]
    pub fn idx_valid(&self, idx: usize) -> bool {
        let clamped = clamp_idx(idx);
        clamped == idx
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
    pub fn move_cell(&mut self, from_pos: Pos, mut to_pos: Pos, swap: bool) {
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
}
