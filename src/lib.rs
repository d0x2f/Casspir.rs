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
///
/// ```
/// use casspir::map;
/// use casspir::point;
/// use casspir::solver;
/// use std::collections::HashSet;
///
/// let mines: HashSet<point::Point> = [
///     point::Point { x: 3, y: 1 },
///     point::Point { x: 4, y: 2 },
///     point::Point { x: 1, y: 1 },
///     point::Point { x: 2, y: 2 },
///     point::Point { x: 4, y: 4 },
/// ].iter()
///     .cloned()
///     .collect();
/// let mut map = casspir::generate_map_with_mines(5, 5, mines);
/// map.flip(&point::Point { x: 0, y: 4 });
/// map.flag(&point::Point { x: 3, y: 1 });
/// let moves = solver::solve(&map);
/// assert_eq!(14, moves.len());
/// for play in &moves {
///     if play.move_type == solver::MoveType::Flip {
///         map.flip(&play.position);
///     } else {
///         map.flag(&play.position);
///     }
/// }
/// assert_eq!(map::Status::Complete, *map.get_status());
/// ```
pub fn solve_map(map: &map::Map) -> VecDeque<solver::Move> {
    solver::solve(map)
}
