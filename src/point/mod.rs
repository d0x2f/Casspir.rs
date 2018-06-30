//! This module contains tools for manipulating a 2d point vector.

use std::collections::HashSet;

/// Represents a 2d point.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    /// Create an array index from this point and the given width.
    ///
    /// ```
    /// use casspir::point;
    /// let point = point::Point { x: 3, y: 7 };
    /// assert_eq!(point.to_index(10), 73);
    /// ```
    pub fn to_index(&self, width: u16) -> usize {
        (self.y as usize * width as usize) + self.x as usize
    }
}

/// Create a `Point` from an array `index` and puzzle `width`.
///
/// ```
/// use casspir::point;
/// assert_eq!(point::from_index(100, 20), point::Point { x: 0, y: 5 });
/// assert_eq!(point::from_index(22, 6), point::Point { x: 4, y: 3 });
/// assert_eq!(point::from_index(54, 8), point::Point { x: 6, y: 6 });
///
/// let result = std::panic::catch_unwind(|| point::from_index(65536, 1));
/// assert!(result.is_err());
/// ```
pub fn from_index(index: usize, width: u16) -> Point {
    if index / width as usize > (1 << 16) - 1 {
        panic!("Unsupported puzzle dimensions.");
    }
    Point {
        x: (index % width as usize) as u16,
        y: (index / width as usize) as u16,
    }
}

/// Get an array of points representing adjacent tiles.
///
/// ```
/// use casspir::point;
/// use std::collections::HashSet;
///
/// let mut expected = [
///     point::Point { x: 5, y: 4 },
///     point::Point { x: 5, y: 6 },
///     point::Point { x: 4, y: 5 },
///     point::Point { x: 4, y: 4 },
///     point::Point { x: 4, y: 6 },
/// ].iter().cloned().collect();
/// assert_eq!(
///     point::get_neighbours(&point::Point{ x: 5, y: 5 }, 6, 10),
///     expected
/// );
/// ```
pub fn get_neighbours(position: &Point, width: u16, height: u16) -> HashSet<Point> {
    let mut neighbours = HashSet::new();

    let u: bool = position.y > 0;
    let d: bool = position.y < (height - 1);
    let l: bool = position.x > 0;
    let r: bool = position.x < (width - 1);

    if u {
        neighbours.insert(Point {
            x: position.x,
            y: position.y - 1,
        });
    }

    if d {
        neighbours.insert(Point {
            x: position.x,
            y: position.y + 1,
        });
    }

    if l {
        neighbours.insert(Point {
            x: position.x - 1,
            y: position.y,
        });
    }

    if r {
        neighbours.insert(Point {
            x: position.x + 1,
            y: position.y,
        });
    }

    if u && l {
        neighbours.insert(Point {
            x: position.x - 1,
            y: position.y - 1,
        });
    }

    if u && r {
        neighbours.insert(Point {
            x: position.x + 1,
            y: position.y - 1,
        });
    }

    if d && l {
        neighbours.insert(Point {
            x: position.x - 1,
            y: position.y + 1,
        });
    }

    if d && r {
        neighbours.insert(Point {
            x: position.x + 1,
            y: position.y + 1,
        });
    }

    neighbours
}
