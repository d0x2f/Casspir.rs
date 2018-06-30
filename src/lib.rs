extern crate rand;

pub mod map;
pub mod point;
pub mod solver;

use std::collections::HashSet;
use std::collections::VecDeque;

/// Generate a map based on a given `difficulty` and initial `click`.
pub fn generate_map_with_difficulty(
    width: u16,
    height: u16,
    difficulty: u8,
    click: point::Point,
) -> map::Map {
    map::generate_map_with_difficulty(width, height, difficulty, click)
}

/// Generate a map with the given positions as mines.
pub fn generate_map_with_mines(width: u16, height: u16, mines: HashSet<point::Point>) -> map::Map {
    map::generate_map_with_mines(width, height, mines)
}

/// Solve the given `map`.
/// Produces a Queue of moves needed to solve the puzzle.
pub fn solve_map(map: &map::Map) -> VecDeque<solver::Move> {
    solver::solve(map)
}
