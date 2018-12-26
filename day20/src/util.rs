use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            'N' => Point::new(0, -1),
            'E' => Point::new(1, 0),
            'S' => Point::new(0, 1),
            'W' => Point::new(-1, 0),
            _ => panic!(),
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        *self = Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}) }}", self.x, self.y)
    }
}
