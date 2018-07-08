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

const GROUP_SIZE_LIMIT: usize = 18;

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

/// Check each neighbour tile to determine if we can be sure it is or isn't a mine.
/// This is called straight after the given tile is flipped, as the new information
/// gained by this tiles value could help solve neighbour tiles.
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

    // If this tile is satisfied, flip all neighbouring unflipped tiles (via convenience flip on the one tile).
    if flagged == map.get_tile(index).value && unflipped - flagged > 0 {
        let position: Point = point::from_index(index, map.get_width());
        map.flip(&position);
        moves.push_back(Move {
            position,
            move_type: MoveType::Flip,
        });
    // If the number of unflipped tiles equals this tiles value, they must all be mines.
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

/// Find and solve each discovered group one at a time.
/// Note: this can be improved by considering distinct groups seperately
/// that way multiple uncertain moves can be made with one pass because
/// we know tiles from separate groups won't affect each others solution.
fn enumerate_groups(map: &mut Map) -> VecDeque<Move> {
    let map_size = map.get_size();
    let mut candidates: HashSet<(usize, usize)>;
    let mut visited = HashSet::<usize>::new();
    let mut group_visited = HashSet::<usize>::new();

    // If the number of remaining tiles is less than `GROUP_SIZE_LIMIT`,
    // just compute the permutations as one group.
    if map_size - map.get_tiles_flipped() < GROUP_SIZE_LIMIT as u32 {
        let mut border_unflipped = HashSet::<usize>::new();

        for i in 0..map_size as usize {
            if !map.get_tile(i).flipped && !map.get_tile(i).flagged {
                border_unflipped.insert(i);
            }
        }
        candidates = evaluate_group(map, &border_unflipped);
    } else {
        candidates = HashSet::new();
        // Loop over each tile and consider it's group.
        for i in 0..map_size as usize {
            // Skip flipped tiles and tiles that have already been considered.
            if visited.contains(&i) || map.get_tile(i).flipped {
                continue;
            }

            let groups: Vec<HashSet<usize>> =
                recursive_border_search(map, i, &mut visited, &mut group_visited);

            // Evaluate each group
            for group in groups {
                if group.len() < GROUP_SIZE_LIMIT {
                    candidates.extend(evaluate_group(map, &group));
                }
            }
        }
    }

    let mut moves: VecDeque<Move> = VecDeque::new();

    // Sort the candidates
    let mut candidates_sorted = Vec::from_iter(candidates.iter());
    candidates_sorted.sort_by(|a, b| a.1.cmp(&b.1));

    let mut min_risk_tuple = (0, 0);
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
    if moves.len() == 0 && min_risk_tuple_found {
        map.flip(&position);
        moves.push_back(Move {
            position,
            move_type: MoveType::Flip,
        });
    }

    moves
}

/// Recursively search the for the border tiles of a cohesive group of tiles.
/// A group of tiles is defined as a collection of tiles such that their configuration
/// is solvable by considering only tiles within the group.
fn recursive_border_search(
    map: &Map,
    index: usize,
    visited: &mut HashSet<usize>,
    mut group_visited: &mut HashSet<usize>,
) -> Vec<HashSet<usize>> {
    // Stop recursion if this tile is flipped, flagged or already visited.
    if visited.contains(&index) || map.get_tile(index).flipped || map.get_tile(index).flagged {
        return vec![];
    }

    // Add to visited list
    visited.insert(index);

    let mut found_borders: Vec<HashSet<usize>> = Vec::new();

    // Loop over the neighbours to determine if this is a border tile and to recurse.
    let neighbours: HashSet<Point> = point::get_neighbours(
        &point::from_index(index, map.get_width()),
        map.get_width(),
        map.get_height(),
    );
    for neighbour in &neighbours {
        let neighbour_index = neighbour.to_index(map.get_width());

        // Skip this tile if it's already been visited.
        if visited.contains(&neighbour_index) {
            continue;
        }

        // Check if this is a border tile.
        if map.get_tile(neighbour_index).flipped {
            // Skip this tile if it's already in a group.
            if group_visited.contains(&neighbour_index) {
                continue;
            }
            // Find the full border group
            let mut group_members: HashSet<usize> = HashSet::new();
            recursive_border_grok_flipped(
                map,
                &mut group_visited,
                &mut group_members,
                neighbour_index,
            );
            found_borders.push(group_members);
        } else {
            // Continue the search
            found_borders.append(&mut recursive_border_search(
                map,
                neighbour_index,
                visited,
                group_visited,
            ));
        }
    }

    found_borders
}

/// Recursively find all members of the group.
fn recursive_border_grok_flipped(
    map: &Map,
    visited: &mut HashSet<usize>,
    mut members: &mut HashSet<usize>,
    flipped_index: usize,
) {
    // Loop over the neighbours of the flipped tile to find unflipped members of the group.
    let neighbours: HashSet<Point> = point::get_neighbours(
        &point::from_index(flipped_index, map.get_width()),
        map.get_width(),
        map.get_height(),
    );
    for neighbour in &neighbours {
        let neighbour_index = neighbour.to_index(map.get_width());

        // Skip this tile if it's already been visited.
        if visited.contains(&neighbour_index) {
            continue;
        }

        // Check if this neighbour is unflipped and unflagged.
        if !map.get_tile(neighbour_index).flipped && !map.get_tile(neighbour_index).flagged {
            visited.insert(neighbour_index);

            // Add this neighbour to the group.
            members.insert(neighbour_index);

            // Recurse
            recursive_border_grok_unflipped(map, visited, &mut members, neighbour_index);
        }
    }
}

/// Recursively find all members of the group.
fn recursive_border_grok_unflipped(
    map: &Map,
    visited: &mut HashSet<usize>,
    mut members: &mut HashSet<usize>,
    unflipped_index: usize,
) {
    // Loop over the neighbours of the unflipped tile to find flipped members of the group.
    let neighbours: HashSet<Point> = point::get_neighbours(
        &point::from_index(unflipped_index, map.get_width()),
        map.get_width(),
        map.get_height(),
    );
    for neighbour in &neighbours {
        let neighbour_index = neighbour.to_index(map.get_width());

        // Skip this tile if it's already been visited.
        if visited.contains(&neighbour_index) {
            continue;
        }

        // Check if this is a border tile.
        if map.get_tile(neighbour_index).flipped {
            visited.insert(neighbour_index);

            // Recurse
            recursive_border_grok_flipped(map, visited, &mut members, neighbour_index);
        }
    }
}

/// Compute possible permutations within the given group to find tiles that either must
/// be flagged or must be a mine. Produces a list of tile nominations with a risk value associated.
fn evaluate_group(map: &mut Map, tiles_unflipped: &HashSet<usize>) -> HashSet<(usize, usize)> {
    let mut staging_map: Map = map.clone();
    let unflipped_count: usize = min(GROUP_SIZE_LIMIT, tiles_unflipped.len());
    let max_mines: u32 = min(staging_map.get_mines_remaining(), unflipped_count as u32);
    let mut tallies = HashMap::<usize, u32>::new();
    let mut valid_permutations = 0;
    let map_width = staging_map.get_width();
    let mut tiles_flipped: HashSet<usize> = HashSet::new();

    // Sort so that results are deterministic.
    let mut tiles_unflipped_sorted = Vec::from_iter(tiles_unflipped.iter());
    tiles_unflipped_sorted.sort();

    // Find all the neibouring flipped tiles.
    for index in &tiles_unflipped_sorted {
        let neighbours: HashSet<Point> = point::get_neighbours(
            &point::from_index(**index, map.get_width()),
            map.get_width(),
            map.get_height(),
        );
        for neighbour in neighbours {
            let neighbour_index = neighbour.to_index(map.get_width());
            if map.get_tile(neighbour_index).flipped {
                tiles_flipped.insert(neighbour_index);
            }
        }

        // Initialise flag tallies
        tallies.insert(**index, 0);
    }

    // Initialise the valid flag tally map.
    for index in tiles_unflipped {
        tallies.insert(*index, 0);
    }

    // Loop for each possible permutation of flag positions.
    'outer: for i in 0..(1 << unflipped_count) {
        // Skip early if this permutation contains too many mines.
        if (i as usize).count_ones() > max_mines {
            continue;
        }

        let mut j: usize = 0;
        for index in &tiles_unflipped_sorted {
            // Use the permutation index to determine if this tile is flagged or not
            // using i as a mitmask.
            if i & (1 << j) > 0 {
                if !staging_map.get_tile(**index).flagged {
                    staging_map.flag(&point::from_index(**index, map_width));
                }
            } else {
                if staging_map.get_tile(**index).flagged {
                    staging_map.flag(&point::from_index(**index, map_width));
                }
            }
            j += 1;
        }

        // Check if the flipped tiles are satisfied by this permutation.
        for index in &tiles_flipped {
            if !staging_map.is_tile_satisfied(&point::from_index(*index, map_width)) {
                continue 'outer;
            }
        }
        valid_permutations += 1;

        // Increment the valid flag tally for each unflipped tile.
        for index in tiles_unflipped {
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
    if nominations.len() == 0 && valid_permutations > 0 {
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
        // Define mine positions
        let mines: HashSet<point::Point> = [
            point::Point { x: 3, y: 1 },
            point::Point { x: 4, y: 2 },
            point::Point { x: 1, y: 1 },
            point::Point { x: 2, y: 2 },
            point::Point { x: 4, y: 4 },
        ].iter()
            .cloned()
            .collect();

        // Generate map with these mines.
        let mut map = map::generate_map_with_mines(5, 5, mines);

        // Flip and flag a couple of tiles.
        map.flip(&point::Point { x: 0, y: 4 });
        map.flag(&point::Point { x: 3, y: 1 });

        // Map should be in progrsss
        assert_eq!(map::Status::InProgress, *map.get_status());

        // Solve the map.
        let moves = solver::solve(&map);

        // Should have taken 14 moves.
        assert_eq!(14, moves.len());

        // Apply the moves to the map.
        map.apply_moves(&moves);

        // Map should be completed
        assert_eq!(map::Status::Complete, *map.get_status());
    }

    #[test]
    fn test_hard_solve() {
        // Define mine positions.
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

        // Create a map with these mines.
        let mut map = map::generate_map_with_mines(10, 10, mines);

        // Flip a safe tile.
        map.flip(&point::Point { x: 6, y: 9 });

        // Solve the map.
        let moves = solver::solve(&map);

        // Apply the moves to the map.
        map.apply_moves(&moves);

        // Map should be solved.
        assert_eq!(map::Status::Complete, *map.get_status());

        // Should have taken 61 moves
        assert_eq!(61, moves.len());
    }

    #[test]
    fn test_random_move() {
        // Define mine positions.
        let mines: HashSet<point::Point> = [
            point::Point { x: 3, y: 0 },
            point::Point { x: 3, y: 1 },
            point::Point { x: 3, y: 2 },
            point::Point { x: 3, y: 3 },
            point::Point { x: 3, y: 4 },
            point::Point { x: 3, y: 5 },
            point::Point { x: 3, y: 6 },
            point::Point { x: 3, y: 7 },
            point::Point { x: 3, y: 8 },
            point::Point { x: 3, y: 9 },
            point::Point { x: 4, y: 9 },
            point::Point { x: 5, y: 9 },
            point::Point { x: 6, y: 9 },
            point::Point { x: 7, y: 9 },
            point::Point { x: 7, y: 8 },
            point::Point { x: 7, y: 7 },
            point::Point { x: 7, y: 6 },
            point::Point { x: 7, y: 5 },
            point::Point { x: 7, y: 4 },
            point::Point { x: 7, y: 3 },
            point::Point { x: 7, y: 2 },
            point::Point { x: 7, y: 1 },
            point::Point { x: 7, y: 0 },
        ].iter()
            .cloned()
            .collect();

        // Create a map with these mines.
        let mut map = map::generate_map_with_mines(10, 10, mines);

        // Flip a tile that would require the next move be random.
        map.flip(&point::Point { x: 5, y: 4 });

        // Solve the map.
        let moves = solver::solve(&map);

        // Apply the moves to the map.
        map.apply_moves(&moves);

        // Map should be completed.
        assert!(*map.get_status() != map::Status::InProgress);
    }
}
