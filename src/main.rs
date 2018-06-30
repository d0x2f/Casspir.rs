extern crate casspir;

use casspir::map;
use casspir::point;
use casspir::solver;
use std::collections::HashSet;

fn main() {
    let mines: HashSet<point::Point> = [
        point::Point { x: 0, y: 0 },
        point::Point { x: 1, y: 0 },
        point::Point { x: 2, y: 0 },
        point::Point { x: 3, y: 1 },
        point::Point { x: 4, y: 1 },
        point::Point { x: 5, y: 1 },
        point::Point { x: 7, y: 1 },
        point::Point { x: 8, y: 1 },
        point::Point { x: 2, y: 2 },
        point::Point { x: 6, y: 2 },
        point::Point { x: 9, y: 2 },
        point::Point { x: 0, y: 4 },
        point::Point { x: 2, y: 5 },
        point::Point { x: 4, y: 5 },
        point::Point { x: 5, y: 5 },
        point::Point { x: 8, y: 5 },
        point::Point { x: 9, y: 5 },
        point::Point { x: 1, y: 6 },
        point::Point { x: 9, y: 6 },
        point::Point { x: 0, y: 7 },
        point::Point { x: 1, y: 7 },
        point::Point { x: 4, y: 7 },
        point::Point { x: 4, y: 8 },
        point::Point { x: 3, y: 9 },
        point::Point { x: 8, y: 9 },
    ].iter()
        .cloned()
        .collect();
    let mut map = map::generate_map_with_mines(10, 10, mines);
    map.flip(&point::Point { x: 6, y: 9 });
    map.print(&mut std::io::stdout(), true).unwrap();
    let moves = solver::solve(&map);

    // Apply the moves to the map.
    for play in &moves {
        if play.move_type == solver::MoveType::Flip {
            map.flip(&play.position);
        } else {
            map.flag(&play.position);
        }
    }
    map.print(&mut std::io::stdout(), false).unwrap();

    // Map should be solved.
    assert_eq!(map::Status::Complete, *map.get_status());

    // Should have taken 78 moves
    assert_eq!(78, moves.len());
}
