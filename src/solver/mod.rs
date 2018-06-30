//! This module contains tools for solving a puzzle.

use map::{Map, Status, Tile};
use point;
use point::Point;
use rand;
use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;

#[derive(PartialEq, Clone, Debug)]
pub enum MoveType {
    Flip,
    Flag,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Move {
    pub position: Point,
    pub move_type: MoveType,
}

/// Solve the given map and produce a queue of moves representing the solution.
pub fn solve(map: &Map) -> VecDeque<Move> {
    let mut staging_map: Map = map.clone();
    let mut moves = VecDeque::<Move>::new();
    while *staging_map.get_status() == Status::InProgress {
        let mut new_moves = basic_pass(&mut staging_map);
        if new_moves.len() == 0 {
            new_moves = enumerate_groups(&mut staging_map);
            if new_moves.len() == 0 {
                new_moves.push_back(random_move(&mut staging_map));
            }
        }
        moves.append(&mut new_moves);
    }

    moves
}

fn basic_pass(map: &mut Map) -> VecDeque<Move> {
    let mut moves = VecDeque::<Move>::new();
    for i in 0..map.get_tiles().len() {
        if map.get_tile(i).flipped && map.get_tile(i).value > 0 {
            moves.append(&mut evaluate_neighbours(map, i));
        }
    }

    moves
}

fn evaluate_neighbours(map: &mut Map, index: usize) -> VecDeque<Move> {
    let neighbours: HashSet<Point> = point::get_neighbours(
        &point::from_index(index, map.get_width()),
        map.get_width(),
        map.get_height(),
    );

    let mut flagged: u8 = 0;
    let mut unflipped: u8 = 0;
    for neighbour in &neighbours {
        let neighbour_tile: &Tile = &map.get_tile(neighbour.to_index(map.get_width()));
        if neighbour_tile.flagged {
            flagged += 1;
        }
        if !neighbour_tile.flipped {
            unflipped += 1;
        }
    }

    let mut moves = VecDeque::<Move>::new();

    if !map.get_tile(index).flipped && flagged == map.get_tile(index).value && unflipped > 0 {
        let position: Point = point::from_index(index, map.get_width());
        map.flip(&position);
        moves.push_back(Move {
            position,
            move_type: MoveType::Flip,
        });
    } else if unflipped == map.get_tile(index).value {
        for neighbour in &neighbours {
            let neighbour_index = neighbour.to_index(map.get_width());
            if !map.get_tile(neighbour_index).flagged && !map.get_tile(neighbour_index).flipped {
                let position = point::from_index(neighbour_index, map.get_width());
                map.flag(&position);
                moves.push_back(Move {
                    position,
                    move_type: MoveType::Flag,
                });
            }
        }
    }

    moves
}

fn enumerate_groups(map: &mut Map) -> VecDeque<Move> {
    let map_size = map.get_size();
    let mut candidates: HashSet<(usize, usize)>;
    let mut visited = HashSet::<usize>::new();

    // If the number of remaining tiles is less than 20,
    // just compute the permutations as one group.
    if map_size - map.get_tiles_flipped() < 20 {
        let mut border_unflipped = HashSet::<usize>::new();
        let mut border_flipped = HashSet::<usize>::new();

        for i in 0..map_size as usize {
            if !map.get_tile(i).flipped && !map.get_tile(i).flagged {
                border_unflipped.insert(i);

                let neighbours: HashSet<Point> = point::get_neighbours(
                    &point::from_index(i, map.get_width()),
                    map.get_width(),
                    map.get_height(),
                );
                for neighbour in &neighbours {
                    let neighbour_index = neighbour.to_index(map.get_width());
                    if map.get_tile(neighbour_index).flipped {
                        border_flipped.insert(neighbour_index);
                    }
                }
            }
        }
        candidates = evaluate_group(map, &border_unflipped, &border_flipped);
    } else {
        candidates = HashSet::new();
        // Loop over each tile and consider it's group.
        for i in 0..map_size as usize {
            // Skip flipped tiles and tiles that have already been considered.
            if visited.contains(&i) || map.get_tile(i).flipped {
                continue;
            }

            let mut border_unflipped: HashSet<usize> = HashSet::new();
            let mut border_flipped: HashSet<usize> = HashSet::new();

            recursive_border_search(
                map,
                i,
                &mut border_unflipped,
                &mut border_flipped,
                &mut visited,
            );

            visited.extend(&border_unflipped);

            // If the found group is less that 20 tiles, evaluate it.
            // Larger groups are too computationally time consuming.
            if border_unflipped.len() > 0 && border_unflipped.len() < 20 {
                candidates.extend(evaluate_group(map, &border_unflipped, &border_flipped));
            }
        }
    }

    let mut moves: VecDeque<Move> = VecDeque::new();

    // Sort the candidates
    let mut candidates_sorted = Vec::from_iter(candidates.iter());
    candidates_sorted.sort_by(|a, b| a.1.cmp(&b.1));

    let mut min_risk_tuple = (0, 257);
    let mut min_risk_tuple_found = false;
    for candidate in candidates_sorted {
        let position = point::from_index(candidate.0, map.get_width());
        // Zero risk flip.
        if candidate.1 == 0 {
            map.flip(&position);
            moves.push_back(Move {
                position,
                move_type: MoveType::Flip,
            });
        // Certain mine.
        } else if candidate.1 == 256 {
            map.flag(&position);
            moves.push_back(Move {
                position,
                move_type: MoveType::Flag,
            });
        } else if !min_risk_tuple_found {
            min_risk_tuple = *candidate;
            min_risk_tuple_found = true;
        }
    }

    // If no certain moves were made, do the least risky.
    let position = point::from_index(min_risk_tuple.0, map.get_width());
    if moves.len() == 0 && min_risk_tuple.1 != 257 {
        map.flip(&position);
        moves.push_back(Move {
            position,
            move_type: MoveType::Flip,
        });
    }

    moves
}

fn recursive_border_search(
    map: &Map,
    index: usize,
    border_unflipped: &mut HashSet<usize>,
    border_flipped: &mut HashSet<usize>,
    visited: &mut HashSet<usize>,
) {
    // Stop recursion if this tile is flipped, flagged or already visited.
    if visited.contains(&index) || map.get_tile(index).flipped || map.get_tile(index).flagged {
        return;
    }

    // Add to visited list
    visited.insert(index);

    // Loop over the neighbours to determine if this is a border tile and to recurse.
    let neighbours: HashSet<Point> = point::get_neighbours(
        &point::from_index(index, map.get_width()),
        map.get_width(),
        map.get_height(),
    );
    for neighbour in &neighbours {
        let neighbour_index = neighbour.to_index(map.get_width());
        if map.get_tile(neighbour_index).flipped {
            border_flipped.insert(neighbour_index);
            border_unflipped.insert(index);
        } else {
            recursive_border_search(
                map,
                neighbour_index,
                border_unflipped,
                border_flipped,
                visited,
            );
        }
    }
}

/// Compute possible permutations within the given group to find tiles that either must
/// be flagged or must be a mine. Produces a list of tile nominations with a risk value associated.
fn evaluate_group(
    map: &mut Map,
    border_unflipped: &HashSet<usize>,
    border_flipped: &HashSet<usize>,
) -> HashSet<(usize, usize)> {
    let mut staging_map: Map = map.clone();
    let unflipped_count: usize = border_unflipped.len();
    let max_mines: u32 = min(staging_map.get_mines_remaining(), unflipped_count as u32);
    let mut tallies = HashMap::<usize, u32>::new();
    let mut valid_permutations = 0;
    let map_width = staging_map.get_width();

    // Initialise the valid flag tally map.
    for index in border_unflipped {
        tallies.insert(*index, 0);
    }

    // Loop for each possible permutation of flag positions.
    'outer: for i in 0..(1 << unflipped_count) {
        // Skip early if this permutation contains too many mines.
        if (i as usize).count_ones() > max_mines {
            continue;
        }

        let mut j: usize = 0;
        for index in border_unflipped {
            // Use the permutation index to determine if this tile is flagged or not
            // using i as a mitmask.
            if i & (1 << j) > 0 {
                if !staging_map.get_tile(*index).flagged {
                    staging_map.flag(&point::from_index(*index, map_width));
                }
            } else {
                if staging_map.get_tile(*index).flagged {
                    staging_map.flag(&point::from_index(*index, map_width));
                }
            }
            j += 1;
        }

        // Check if the flipped tiles are satisfied by this permutation.
        for index in border_flipped {
            if !staging_map.is_tile_satisfied(&point::from_index(*index, map_width)) {
                continue 'outer;
            }
        }
        valid_permutations += 1;

        // Increment the valid flag tally for each unflipped tile.
        for index in border_unflipped {
            if staging_map.get_tile(*index).flagged {
                let tally = tallies.entry(*index).or_insert(0);
                *tally += 1;
            }
        }
    }

    let mut nominations: HashSet<(usize, usize)> = HashSet::new();
    let mut min_index: usize = 0;
    let mut min_value: u32 = valid_permutations + 1;
    for (index, tally) in tallies {
        // Nominate all that never had a flag for flipping.
        if tally == 0 {
            nominations.insert((index, 0));
        // Nominate all that always had a flag for flagging.
        } else if tally == valid_permutations {
            nominations.insert((index, 256));
        } else if tally < min_value {
            min_value = tally;
            min_index = index;
        }
    }

    // If no certain moves were found, nominate the least risky.
    if nominations.len() == 0 {
        nominations.insert((min_index, (min_value * (255 / valid_permutations)) as usize));
    }

    nominations
}

/// Perform a random move
fn random_move(map: &mut Map) -> Move {
    let random_index: usize =
        rand::random::<usize>() % (map.get_size() - map.get_tiles_flipped()) as usize;

    let mut unflipped_index: usize = 0;
    for i in 0..map.get_tiles().len() {
        if !map.get_tile(i).flipped {
            if unflipped_index == random_index {
                let position = point::from_index(i, map.get_width());
                map.flip(&position);
                return Move {
                    position,
                    move_type: MoveType::Flip,
                };
            }
            unflipped_index += 1;
        }
    }
    panic!("Failed to find a random tile.");
}

#[cfg(test)]
mod tests {
    use map;
    use point;
    use solver;
    use std::collections::HashSet;

    #[test]
    fn test_simple_solve() {
        let mines: HashSet<point::Point> = [
            point::Point { x: 3, y: 1 },
            point::Point { x: 4, y: 2 },
            point::Point { x: 1, y: 1 },
            point::Point { x: 2, y: 2 },
            point::Point { x: 4, y: 4 },
        ].iter()
            .cloned()
            .collect();
        let mut map = map::generate_map_with_mines(5, 5, mines);
        map.flip(&point::Point { x: 0, y: 4 });
        map.flag(&point::Point { x: 3, y: 1 });
        let moves = solver::solve(&map);
        assert_eq!(14, moves.len());
        for play in &moves {
            if play.move_type == solver::MoveType::Flip {
                map.flip(&play.position);
            } else {
                map.flag(&play.position);
            }
        }
        assert_eq!(map::Status::Complete, *map.get_status());
    }

    #[test]
    fn test_hard_solve() {
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
        let moves = solver::solve(&map);

        // Apply the moves to the map.
        for play in &moves {
            if play.move_type == solver::MoveType::Flip {
                map.flip(&play.position);
            } else {
                map.flag(&play.position);
            }
        }

        // Map should be solved.
        assert_eq!(map::Status::Complete, *map.get_status());

        // Should have taken 78 moves
        assert_eq!(78, moves.len());
    }
}
