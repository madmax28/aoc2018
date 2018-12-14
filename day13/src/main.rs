use std::cmp::Ordering;
use std::error;
use std::fmt;
use std::fs;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    InvalidInput,
    InvalidChar,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Dir {
    L,
    R,
    U,
    D,
}

impl Dir {
    fn as_char(self) -> char {
        match self {
            Dir::L => '<',
            Dir::R => '>',
            Dir::U => '^',
            Dir::D => 'v',
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Turn {
    L,
    S,
    R,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Cart {
    pos: (usize, usize),
    dir: Dir,
    turn: Turn,
}

impl Cart {
    fn new(pos: (usize, usize), dir: Dir) -> Self {
        Cart {
            pos,
            dir,
            turn: Turn::L,
        }
    }

    fn turn(&mut self) {
        let (dir, turn) = match (self.dir, self.turn) {
            (Dir::L, Turn::L) => (Dir::D, Turn::S),
            (Dir::L, Turn::R) => (Dir::U, Turn::L),
            (Dir::R, Turn::L) => (Dir::U, Turn::S),
            (Dir::R, Turn::R) => (Dir::D, Turn::L),
            (Dir::U, Turn::L) => (Dir::L, Turn::S),
            (Dir::U, Turn::R) => (Dir::R, Turn::L),
            (Dir::D, Turn::L) => (Dir::R, Turn::S),
            (Dir::D, Turn::R) => (Dir::L, Turn::L),
            (d, Turn::S) => (d, Turn::R),
        };
        self.dir = dir;
        self.turn = turn;
    }

    fn target_pos(&self) -> (usize, usize) {
        match self.dir {
            Dir::L => (self.pos.0 - 1, self.pos.1),
            Dir::R => (self.pos.0 + 1, self.pos.1),
            Dir::U => (self.pos.0, self.pos.1 - 1),
            Dir::D => (self.pos.0, self.pos.1 + 1),
        }
    }

    fn handle_turn(&mut self, c: char) -> Result<()> {
        self.dir = match (self.dir, c) {
            (Dir::L, '\\') => Dir::U,
            (Dir::R, '\\') => Dir::D,
            (Dir::U, '\\') => Dir::L,
            (Dir::D, '\\') => Dir::R,
            (Dir::L, '/') => Dir::D,
            (Dir::R, '/') => Dir::U,
            (Dir::U, '/') => Dir::R,
            (Dir::D, '/') => Dir::L,
            _ => return Err(Box::new(Error::InvalidChar)),
        };
        Ok(())
    }
}

impl PartialOrd for Cart {
    fn partial_cmp(&self, other: &Cart) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cart {
    fn cmp(&self, other: &Cart) -> Ordering {
        if self.pos.1 != other.pos.1 {
            self.pos.1.cmp(&other.pos.1)
        } else {
            self.pos.0.cmp(&other.pos.0)
        }
    }
}

#[derive(Debug, Clone)]
struct Rails {
    size: (usize, usize),
    rails: Vec<char>,
}

impl Rails {
    fn get_unchecked(&self, pos: (usize, usize)) -> char {
        self.rails[pos.0 + pos.1 * self.size.0]
    }

    fn set(&mut self, pos: (usize, usize), c: char) {
        self.rails[pos.0 + pos.1 * self.size.0] = c;
    }
}

#[derive(Debug, Clone)]
struct Grid {
    rails: Rails,
    carts: Vec<Cart>,
}

impl Grid {
    fn tick(&mut self) -> Result<Vec<(usize, usize)>> {
        let mut crashed = Vec::new();
        let mut crashed_pos = Vec::new();

        for i in 0..self.carts.len() {
            if crashed.contains(&i) {
                continue;
            }

            let to = self.carts[i].target_pos();
            if let Some(j) =
                (0..self.carts.len()).find(|j| !crashed.contains(j) && self.carts[*j].pos == to)
            {
                crashed.push(i);
                crashed.push(j);
                crashed_pos.push(to);
                continue;
            }
            self.carts[i].pos = to;

            match self.rails.get_unchecked(to) {
                '+' => self.carts[i].turn(),
                c @ '/' | c @ '\\' => self.carts[i].handle_turn(c)?,
                '-' | '|' => (),
                _ => return Err(Box::new(Error::InvalidChar)),
            }
        }

        crashed.sort();
        for i in crashed.iter().rev() {
            self.carts.remove(*i);
        }

        self.carts.sort();
        Ok(crashed_pos)
    }

    #[allow(dead_code)]
    fn print(&self) {
        let mut rails = self.rails.clone();
        for cart in &self.carts {
            rails.set(cart.pos, cart.dir.as_char());
        }
        for y in 0..rails.size.1 {
            for x in 0..rails.size.0 {
                print!("{}", rails.get_unchecked((x, y)));
            }
            println!();
        }
    }
}

impl FromStr for Grid {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut lines = s.lines();
        let width = lines.next().ok_or(Error::InvalidInput)?.len();
        let height = lines.count() + 1;

        let mut rails = Rails {
            size: (width, height),
            rails: s.chars().filter(|c| *c != '\n').collect(),
        };

        let mut carts = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if let Some((dir, c)) = match rails.get_unchecked((x, y)) {
                    '>' => Some((Dir::R, '-')),
                    '<' => Some((Dir::L, '-')),
                    '^' => Some((Dir::U, '|')),
                    'v' => Some((Dir::D, '|')),
                    _ => None,
                } {
                    carts.push(Cart::new((x, y), dir));
                    rails.set((x, y), c);
                }
            }
        }
        carts.sort();

        Ok(Grid { rails, carts })
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let mut grid: Grid = input.parse()?;

    let mut crash_pos = Vec::new();
    while crash_pos.is_empty() {
        crash_pos = grid.tick()?;
    }

    print!("Part 1: crash at:");
    for p in &crash_pos {
        print!(" {:?}", p);
    }
    println!();

    while grid.carts.len() > 1 {
        grid.tick()?;
    }
    println!("Part 2: {:?}", grid.carts.get(0).expect("no cart left").pos);

    Ok(())
}
