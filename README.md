# Casspir.rs

[![Build Status](https://travis-ci.org/d0x2f/Casspir.rs.svg?branch=master)](https://travis-ci.org/d0x2f/Casspir.rs)
[![Coverage Status](https://coveralls.io/repos/github/d0x2f/Casspir.rs/badge.svg)](https://coveralls.io/github/d0x2f/Casspir.rs)

Minesweeper puzzle generator and solver.

It's not perfect and there are improvements possible, some of which you may be able to notice from the video [here](https://www.youtube.com/watch?v=qlBwNXP5lfM).

## Usage Example

A simple program that uses the library might look like this:

```rust
// Generate a random map.
let mut map = map::generate_map_with_difficulty(5, 5, 100, point::Point { x: 2, y: 3 });

// Print the begining map state.
println!("Puzzle Start:");
map.print(&mut std::io::stdout(), false).unwrap();

// Run the solver
let moves = solver::solve(&map);

// Print each step of the solution.
println!("\nSolution:");
for play in &moves {
    let move_type;
    if play.move_type == solver::MoveType::Flip {
        move_type = "Flip";
    } else {
        move_type = "Flag";
    }
    println!("{} ({},{})", move_type, play.position.x, play.position.y);
}

// Apply the moves to the map.
map.apply_moves(&moves);

// Print the final (hopefully solved) state.
println!("\nPuzzle End:");
map.print(&mut std::io::stdout(), false).unwrap();

if *map.get_status() == map::Status::Complete {
    println!("\nSuccess!");
} else {
    println!("\nFailure :(");
}
```

Which would produce results like so:

```
$ casspir-example
Puzzle Start:

#####
#####
#####
##2##
#####

Solution:
Flip (3,3)
Flag (2,1)
Flip (1,3)
Flip (1,1)
Flip (2,0)
Flag (1,2)
Flag (1,4)
Flag (3,0)
Flag (1,0)
Flip (1,1)
Flip (3,1)
Flip (0,2)
Flip (0,3)

Puzzle End:

1^3^1
23^21
1^210
22200
1^100

Success!
```

---
Ported to rust from [Casspir](https://github.com/d0x2f/Casspir).