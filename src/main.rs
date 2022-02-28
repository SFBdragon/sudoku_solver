use sudoku_solver::Grid;

fn main() {
    let mut grid = None;
    for arg in std::env::args() {
        if arg.trim().trim_start_matches('-').len() == 81 {
            grid = Some(Grid::from_str(arg));
        }
    }

    if let Some(mut g) = grid {
        if let Err(_) = g.solve() {
            println!("No solution could be found.");
            return;
        }
        if !g.verify_solution() {
            panic!("SOLUTION FOUND WAS INVALID, THIS IS LIKELY A BUG.");
        }

        print!("Solution: {}", g.to_str());
    } else {
        println!("Valid grid string argument not found.");
    }
}

