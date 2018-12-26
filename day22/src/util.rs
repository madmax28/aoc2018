use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }

    pub fn nb_iter(self) -> impl Iterator<Item = Point> {
        let mut ns = Vec::with_capacity(4);
        if self.x > 0 {
            ns.push(Point::new(self.x - 1, self.y));
        }
        if self.y > 0 {
            ns.push(Point::new(self.x, self.y - 1));
        }
        ns.push(Point::new(self.x + 1, self.y));
        ns.push(Point::new(self.x, self.y + 1));
        ns.into_iter()
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
        write!(f, "({}, {})", self.x, self.y)
    }
}
