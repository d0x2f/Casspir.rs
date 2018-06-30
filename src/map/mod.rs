//! This module contains tools for manipulating a puzzle map.

use point;
use point::Point;
use rand;
use std::collections::HashSet;
use std::vec::Vec;

/// Represents the completion state of a puzzle.
#[derive(PartialEq, Clone, Debug)]
pub enum Status {
    InProgress,
    Failed,
    Complete,
}

/// Represents the state of a tile.
#[derive(PartialEq, Clone)]
pub struct Tile {
    /// The number of adjacent tiles with mines on them.
    pub value: u8,
    /// If this tile is a mine.
    pub mine: bool,
    /// If this tile has been flagged.
    pub flagged: bool,
    /// If this tile has been flipped.
    pub flipped: bool,
}

/// Represents the state of a map (a game board).
#[derive(PartialEq, Clone)]
pub struct Map {
    /// The width of the map.
    width: u16,
    /// The height of the map.
    height: u16,
    /// The total number of mines (discovered or not) on the map.
    total_mines: u32,
    /// The number of mines left to discover.
    mines_remaining: u32,
    /// The number of tiles flipped.
    tiles_flipped: u32,
    /// The completion state of this map.
    status: Status,
    /// The tiles of the map.
    tiles: Vec<Tile>,
}

impl Map {
    pub fn get_status(&self) -> &Status {
        &self.status
    }
    pub fn get_width(&self) -> u16 {
        self.width
    }
    pub fn get_height(&self) -> u16 {
        self.height
    }
    pub fn get_size(&self) -> u32 {
        self.width as u32 * self.height as u32
    }
    pub fn get_tiles_flipped(&self) -> u32 {
        self.tiles_flipped
    }
    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }
    pub fn get_tile(&self, index: usize) -> &Tile {
        &self.tiles[index]
    }
    pub fn get_mines_remaining(&self) -> u32 {
        self.mines_remaining
    }

    pub fn print(&self, revealed: bool) {
        for i in 0..self.get_tiles().len() {
            if (i % self.width as usize) == 0 {
                print!("\n");
            }
            if self.get_tile(i).flipped || revealed {
                if self.get_tile(i).mine {
                    print!("*");
                } else {
                    print!("{}", self.get_tile(i).value);
                }
            } else if self.get_tile(i).flagged {
                print!("^");
            } else {
                print!("#");
            }
        }
        print!("\n");
    }

    /// Flags or unflags a tile at the given `position`.
    pub fn flag(&mut self, position: &Point) {
        if self.status != Status::InProgress {
            return;
        }

        let index: usize = position.to_index(self.width);

        if self.tiles[index].flipped {
            return;
        }

        if self.tiles[index].flagged {
            self.tiles[index].flagged = false;
            self.mines_remaining += 1;
        } else if self.mines_remaining > 0 {
            self.tiles[index].flagged = true;
            self.mines_remaining -= 1;
        }
    }

    /// Flip the tile at the given `position`.
    /// This can trigger a recursive flip that flips all connected 0 value tiles.
    pub fn flip(&mut self, position: &Point) -> u32 {
        let index: usize = position.to_index(self.width);
        let mut flipped: u32 = 0;

        if self.tiles[index].flipped {
            if self.is_tile_satisfied(position) {
                let neighbours: HashSet<Point> =
                    point::get_neighbours(position, self.width, self.height);
                for neighbour in &neighbours {
                    flipped += self.flip_recurse(neighbour);
                }
            }
        } else if !self.tiles[position.to_index(self.width)].flagged {
            flipped = self.flip_recurse(position);
        }

        self.check_completed();

        flipped
    }

    /// Recursively flip tile neighbours that have a value of 0.
    fn flip_recurse(&mut self, position: &Point) -> u32 {
        if self.status != Status::InProgress {
            return 0;
        }

        let index: usize = position.to_index(self.width);

        if self.tiles[index].flipped || self.tiles[index].flagged {
            return 0;
        }

        self.tiles[index].flipped = true;
        self.tiles_flipped += 1;

        if self.tiles[index].mine {
            self.status = Status::Failed;
            return 1;
        }

        if self.tiles[index].value != 0 {
            return 1;
        }

        let neighbours: HashSet<Point> = point::get_neighbours(position, self.width, self.height);
        let mut flipped: u32 = 0;
        for neighbour in &neighbours {
            flipped += self.flip_recurse(neighbour);
        }

        flipped
    }

    /// Checks if the tile at the given `position` is connected the same number of flags as it's value.
    pub fn is_tile_satisfied(&self, position: &Point) -> bool {
        let tile: &Tile = &self.tiles[position.to_index(self.width)];
        let neighbours: HashSet<Point> = point::get_neighbours(position, self.width, self.height);

        let mut flags: u8 = 0;
        for neighbour in neighbours {
            if self.tiles[neighbour.to_index(self.width)].flagged {
                flags += 1;
            }
        }

        flags == tile.value
    }

    /// Check if the map is completed and update the status if so.
    fn check_completed(&mut self) {
        if self.status != Status::InProgress {
            return;
        }

        if (self.tiles_flipped + self.total_mines) as usize == self.tiles.len() {
            self.status = Status::Complete;
        }
    }
}

/// Generate a map based on a given `difficulty` and initial `click`.
pub fn generate_map_with_difficulty(width: u16, height: u16, difficulty: u8, click: Point) -> Map {
    // Initialise a vector of empty tiles.
    let mut tiles = vec![
        Tile {
            value: 0,
            mine: false,
            flagged: false,
            flipped: false
        };
        (width * height) as usize
    ];
    // Choose a mine probability based on the given difficulty.
    let mine_probability: f32 = ((difficulty as f32) + 20.0) / 512.0;

    // Loop over the tiles and turn into a mine with the calculated probability.
    let mut total_mines: u32 = 0;
    for i in 0..tiles.len() {
        let position = point::from_index(i, width);

        // Don't make the first clicked tile a mine.
        if position != click && rand::random::<f32>() < mine_probability {
            tiles[i].mine = true;
            total_mines += 1;

            // Increment the value of neighbouring tiles.
            for point in point::get_neighbours(&position, width, height) {
                tiles[point.to_index(width)].value += 1;
            }
        }
    }

    // Return the constructed map.
    Map {
        width,
        height,
        total_mines,
        mines_remaining: total_mines,
        tiles_flipped: 0,
        status: Status::InProgress,
        tiles,
    }
}

/// Generate a map with given mine locations.
pub fn generate_map_with_mines(width: u16, height: u16, mines: HashSet<Point>) -> Map {
    // Initialise a vector of empty tiles.
    let mut tiles = vec![
        Tile {
            value: 0,
            mine: false,
            flagged: false,
            flipped: false
        };
        (width * height) as usize
    ];

    // Loop over the tiles and turn into a mine with the calculated probability.
    let total_mines: u32 = mines.len() as u32;
    for mine in &mines {
        // Ensure the mine is within the puzzle size.
        let index: usize = mine.to_index(width);
        if index > ((width * height) - 1) as usize {
            panic!("Cannot place a mine outside the puzzle bounds.");
        }
        // Set as mine.
        tiles[index].mine = true;

        // Increment the value of neighbouring tiles.
        for point in point::get_neighbours(mine, width, height) {
            tiles[point.to_index(width)].value += 1;
        }
    }

    // Return the constructed map.
    Map {
        width,
        height,
        total_mines,
        mines_remaining: total_mines,
        tiles_flipped: 0,
        status: Status::InProgress,
        tiles,
    }
}