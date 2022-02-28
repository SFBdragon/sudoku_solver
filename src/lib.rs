#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;


#[derive(Debug, Clone)]
pub struct Grid {
    /// Digit of each cell, zero if blank.
    cells: [[u8; 9]; 9],
    /// Bitmaps of possible digits per cell.
    maybe: [[u16; 9]; 9],
    
    /// Digit counts per row, indexed by rows then by digits.
    row_digit_counters: [[u8; 9]; 9],
    /// Digit counts per columns, indexed by columns then by digits.
    col_digit_counters: [[u8; 9]; 9],
    /// Digit counts per blocks, indexed by blocks then by digits.
    blk_digit_counters: [[u8; 9]; 9],
    
    /// Stack of 'maybe' deletions to take to take.
    del_maybes: Vec<(usize, (usize, usize))>,
    /// Stack of cell sets to take to take.
    set_cells: Vec<(usize, (usize, usize))>,
}

impl Default for Grid {
    /// Returns an empty grid.
    fn default() -> Self {
        Self {
            cells: [[0u8; 9]; 9],
            maybe: [[Grid::MAYBE_ALL; 9]; 9],

            row_digit_counters: [[9u8; 9]; 9],
            col_digit_counters: [[9u8; 9]; 9],
            blk_digit_counters: [[9u8; 9]; 9],

            del_maybes: Vec::new(),
            set_cells: Vec::new(),
        }
    }
}

impl Grid {
    const MAYBE_ALL: u16 = 0x1ff;

    /// Parses an 81-character string of ASCII digits from 0 to 9, returning a Grid on success.
    pub fn from_str(str: String) -> Self {
        let str = str.trim();
        assert_eq!(str.len(), 81, "Grid string not 81 characters long, instead {}.", str.len());

        let mut grid = Grid::default();

        for (i, char) in str.chars().enumerate() {
            let digit = char.to_digit(10).expect("Unexpected character in grid string.");
            if digit != 0 {
                let y = i / 9;
                let x = i - y * 9;
                
                grid.set_cells.push((digit as usize, (x, y)))
            }
        }

        grid
    }
    /// Serialize grid into an 81-character string of ASCII digits from 0 to 9.
    pub fn to_str(&self) -> String {
        let mut str = String::with_capacity(81);

        for y in 0..9 {
            for x in 0..9 {
                str.push(char::from_digit(self.cells[x][y] as u32, 10)
                    .expect("grid contains invalid cell?"));
            }
        }

        str
    }

    /// Check whether the grid is in a valid solved state or not.
    pub fn verify_solution(&self) -> bool {

        // Check for exhaustion of maybes/available blocks
        if self.maybe              != [[0u16; 9]; 9]
        || self.row_digit_counters != [[0u8; 9]; 9]
        || self.col_digit_counters != [[0u8; 9]; 9] 
        || self.blk_digit_counters != [[0u8; 9]; 9] {
            return false;
        }

        // Ensure there is one of each digit in every row, column, and block
        let mut row_digit_counters = [[0u8; 9]; 9];
        let mut col_digit_counters = [[0u8; 9]; 9];
        let mut blk_digit_counters = [[0u8; 9]; 9];
        
        for y in 0..9 {
            for x in 0..9 {
                if self.cells[x][y] == 0 { return false; }

                row_digit_counters[y                ][self.cells[x][y] as usize - 1] += 1;
                col_digit_counters[x                ][self.cells[x][y] as usize - 1] += 1;
                blk_digit_counters[x / 3 + y / 3 * 3][self.cells[x][y] as usize - 1] += 1;
            }
        }

        row_digit_counters == [[1u8; 9]; 9] &&
        col_digit_counters == [[1u8; 9]; 9] &&
        blk_digit_counters == [[1u8; 9]; 9]
    }

    /// Attempt to solve the grid, returning `Ok(())` on success and `Err(())` on failure.
    /// 
    /// `Err(())` leaves the grid in an undefined state, however `verify_solution` will still give an accurate result.
    /// 
    /// It may be desired to call `verify_solution` on the grid hereafter, however this shouldn't be necessary.
    pub fn solve(&mut self) -> Result<(), ()> {
        loop {
            loop { // Begin solving through elimination and hidden singles in a loop.
                if let Some((digit, index)) = self.del_maybes.pop() {
                    self.del_maybe(digit, index)?;
                } else if let Some((digit, index)) = self.set_cells.pop() {
                    self.set_cell(digit, index)?;
                } else {
                    // Attempt to detect any cells where it is the only possible option of a row/column/block,
                    // even if it itself has multiple possibilities.
                    self.find_hidden_singles();

                    if self.set_cells.len() == 0 {
                        // Solver has exhausted its capabilities
                        break;
                    }
                }
            }

            if self.maybe == [[0; 9]; 9] {
                // Grid has been solved, return
                return Ok(());
            } else {
                // Make a binary guess and use process of elimination to pick the correct one (binary tree nav style)
                let (pair_digit, pair_indecies) = self.find_maybe_pair();

                let mut hypothetical = self.clone();
                hypothetical.set_cells.push((pair_digit, pair_indecies[0]));

                if let Err(_) = hypothetical.solve() {
                    // Hypothetical guess failed, thus the other of the binary possibility must be correct.
                    self.set_cells.push((pair_digit, pair_indecies[1]));
                    continue;
                } else {
                    // Guess was correct and a solution was found, return.
                    *self = hypothetical;
                    return Ok(());
                }
            }
        }
    }

    fn set_cell(&mut self, digit: usize, index: (usize, usize)) -> Result<(), ()> {
        // Repeat check: check if already set
        if self.cells[index.0][index.1] != 0 {
            return if self.cells[index.0][index.1] != digit as u8 {
                Err(()) // Already set to something different, this is a contradiction.
            } else {
                Ok(()) // Already set to the correct value, no action necessary.
            };
        }

        // Contradiction check: if attempt to set a cell that is not maybe the digit, return Err
        if self.maybe[index.0][index.1] & 1 << digit - 1 == 0 {
            return Err(());
        }


        for x in 0..9 { // Remove maybes for each cell in row
            if self.maybe[x][index.1] & 1 << digit - 1 != 0 {
                self.del_maybes.push((digit, (x, index.1)));
            }
        }
        for y in 0..9 { // Remove maybes for each cell in column
            if self.maybe[index.0][y] & 1 << digit - 1 != 0 {
                self.del_maybes.push((digit, (index.0, y)));
            }
        }

        let blk_x = index.0 / 3 * 3;
        let blk_y = index.1 / 3 * 3;
        for y in blk_y..(blk_y + 3) { // Remove maybes for each cell in block
            for x in blk_x..(blk_x + 3) {
                if self.maybe[x][y] & 1 << digit - 1 != 0 {
                    self.del_maybes.push((digit, (x, y)));
                }
            }
        }

        // Set cell
        self.cells[index.0][index.1] = digit as u8;

        // Erase maybes
        let mut maybes = self.maybe[index.0][index.1];
        while maybes != 0 {
            let di = maybes.trailing_zeros();
            self.update_counters(di as usize + 1, index);
            maybes ^= 1 << di;
        }
        self.maybe[index.0][index.1] = 0;
        
        Ok(())
    }
    fn del_maybe(&mut self, digit: usize, index: (usize, usize)) -> Result<(), ()> {
        // If already unmaybed, return early
        if self.maybe[index.0][index.1] & 1 << digit - 1 == 0 {
            return Ok(());
        }

        // Delete maybe
        self.maybe[index.0][index.1] &= !(1 << digit - 1);

        // If there is only one remaining digit that may be set, set the cell.
        if self.maybe[index.0][index.1].count_ones() == 1 {
            self.set_cells.push((self.maybe[index.0][index.1].trailing_zeros() as usize + 1, index));
        }

        self.update_counters(digit, index);

        Ok(())
    }
    fn update_counters(&mut self, digit: usize, index: (usize, usize)) {
        //! Decrement the row, column, and block counters according to the digit.
        
        self.row_digit_counters[index.1                      ][digit - 1] -= 1;
        self.col_digit_counters[index.0                      ][digit - 1] -= 1;
        self.blk_digit_counters[index.0 / 3 + index.1 / 3 * 3][digit - 1] -= 1;// 5, 6   1 + 6 = 7
    }

    fn find_hidden_singles(&mut self) {
        for row in 0..9 {
            for di in 0..9 {
                if self.row_digit_counters[row][di] == 1 {
                    // hidden single located, find and set
                    for x in 0..9 {
                        if self.maybe[x][row] & 1 << di != 0 {
                            self.set_cells.push((di + 1, (x, row)));
                        }
                    }
                }
            }
        }
        for col in 0..9 {
            for di in 0..9 {
                if self.col_digit_counters[col][di] == 1 {
                    // hidden single located, find and set
                    for y in 0..9 {
                        if self.maybe[col][y] & 1 << di != 0 {
                            self.set_cells.push((di + 1, (col, y)));
                        }
                    }
                }
            }
        }
        for blk in 0..9 {
            for di in 0..9 {
                if self.blk_digit_counters[blk][di] == 1 {
                    // hidden single located, find and set
                    let blk_y = blk / 3 * 3;
                    let blk_x = (blk - blk_y) * 3;

                    for y in blk_y..(blk_y + 3) {
                        for x in blk_x..(blk_x + 3) {
                            if self.maybe[x][y] & 1 << di != 0 {
                                self.set_cells.push((di + 1, (x, y)));
                            }
                        }
                    }
                }
            }
        }
    }
    fn find_maybe_pair(&self) -> (usize, [(usize, usize); 2]) {
        //! Search the grid for a binary maybe and return the two possibilities as `(digit, [(x index, y index); 2])`.
        // searches do not terminate early on counter == 2 such that they error if state is invalid

        let mut cell_index = 0;
        let mut cells = [(usize::MAX, usize::MAX); 2];

        for row in 0..9 {
            for di in 0..9 {
                if self.row_digit_counters[row][di] == 2 {
                    // pair of digit has been found in row; locate the cells
                    for x in 0..9 {
                        if self.maybe[x][row] & 1 << di != 0 {
                            cells[cell_index] = (x, row);
                            cell_index += 1;
                        }
                    }
                    return (di + 1, cells);
                }
            }
        }
        for col in 0..9 {
            for di in 0..9 {
                if self.col_digit_counters[col][di] == 2 {
                    // pair of digit has been found in col; locate the cells
                    for y in 0..9 {
                        if self.maybe[col][y] & 1 << di != 0 {
                            cells[cell_index] = (col, y);
                            cell_index += 1;
                        }
                    }
                    return (di + 1, cells);
                }
            }
        }
        for blk in 0..9 {
            for di in 0..9 {
                if self.blk_digit_counters[blk][di] == 2 {
                    // pair of digit has been found in blk; locate the cells
                    let blk_y = blk / 3 * 3;
                    let blk_x = (blk - blk_y) * 3;

                    for y in blk_y..(blk_y + 3) {
                        for x in blk_x..(blk_x + 3) {
                            if self.maybe[x][y] & 1 << di != 0 {
                                cells[cell_index] = (x, y);
                                cell_index += 1;
                            }
                        }
                    }
                    return (di + 1, cells);
                }
            }
        }

        panic!("binary possiblity could not be found, this is likely an implementation error")
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    #[test]
    fn test_solver() {
        let mut grid = super::Grid::from_str("600008940900006100070040000200610000000000200089002000000060005000000030800001600".to_string());
        assert!(grid.solve().is_ok());
        assert_eq!(grid.to_str(), "625178943948326157371945862257619384463587291189432576792863415516294738834751629");

        let mut grid2 = super::Grid::from_str("100007090030020008009600500005300900010080002600004000300000010040000007007000300".to_string());
        assert!(grid2.solve().is_ok());
        assert_eq!(grid2.to_str(), "162857493534129678789643521475312986913586742628794135356478219241935867897261354");

        
        let mut grid3 = super::Grid::from_str("234500200000023040000030400000600000300000000000230040040000654300000010203000004".to_string());
        assert!(grid3.solve().is_err());
    }
}
