pub mod map;
pub mod point;
pub mod solver;

use std::collections::HashSet;
use std::collections::VecDeque;

/// Generate a map based on a given `difficulty` and initial `click`.
///
/// ```
/// use casspir::{self, map, point};
/// let mut map = map::generate_map_with_difficulty(10, 10, 100, point::Point { x: 2, y: 6 });
/// assert_eq!(map::Status::InProgress, *map.get_status());
/// assert!(map.get_mines_remaining() >= 5);
/// ```
pub fn generate_map_with_difficulty(
    width: u16,
    height: u16,
    difficulty: u8,
    click: point::Point,
) -> map::Map {
    map::generate_map_with_difficulty(width, height, difficulty, click)
}

/// Generate a map with the given positions as mines.
///
/// ```
/// use casspir::{self, map, point};
/// use std::collections::HashSet;
/// let mines: HashSet<point::Point> = [point::Point { x: 0, y: 0 }].iter().cloned().collect();
/// let mut map = map::generate_map_with_mines(1, 1, mines);
/// assert_eq!(map::Status::InProgress, *map.get_status());
/// assert_eq!(1, map.get_mines_remaining());
/// ```
pub fn generate_map_with_mines(width: u16, height: u16, mines: HashSet<point::Point>) -> map::Map {
    map::generate_map_with_mines(width, height, mines)
}

/// Solve the given `map`.
/// Produces a Queue of moves needed to solve the puzzle.
///
/// ```
/// use casspir::{self, map, point};
/// let mut map = casspir::generate_map_with_difficulty(10, 10, 1, point::Point { x: 1, y: 4 });
/// let moves = casspir::solve_map(&map);
/// map.apply_moves(&moves);
/// assert!(*map.get_status() != map::Status::InProgress);
/// ```
pub fn solve_map(map: &map::Map) -> VecDeque<solver::Move> {
    solver::solve(map)
}
