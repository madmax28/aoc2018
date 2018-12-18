use std::cmp::{max, min};
use std::fs;
use std::str;

const SPRING: (usize, usize) = (500, 0);

#[derive(Debug)]
enum Error {
    ParseGrid,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Flow,
    Fill,
}

#[derive(Debug)]
struct Grid {
    width: usize,
    height: usize,
    buf: Vec<char>,

    state: State,
    sstack: Vec<State>,
    cursor: (i32, i32),
    cstack: Vec<(i32, i32)>,
}

impl Grid {
    fn new(width: usize, height: usize, xmin: usize, buf: Vec<char>) -> Self {
        Grid {
            width,
            height,
            buf,

            state: State::Flow,
            sstack: Vec::new(),
            cursor: ((SPRING.0 - xmin) as i32, 0),
            cstack: Vec::new(),
        }
    }

    fn set(&mut self, x: usize, y: usize, c: char) {
        self.buf[x + y * self.width] = c;
    }

    fn set_cursor(&mut self, c: char) {
        self.set(self.cursor.0 as usize, self.cursor.1 as usize, c);
    }

    fn peek(&self, dx: i32, dy: i32) -> char {
        let (x, y) = (
            (self.cursor.0 as i32 + dx) as usize,
            (self.cursor.1 as i32 + dy) as usize,
        );
        self.buf[x + y * self.width]
    }

    fn mv(&mut self, dx: i32, dy: i32) {
        self.cursor.0 += dx;
        self.cursor.1 += dy;

        match self.state {
            State::Flow => self.set_cursor('|'),
            State::Fill => self.set_cursor('~'),
        }
    }

    fn push(&mut self) {
        self.cstack.push(self.cursor);
        self.sstack.push(self.state);
    }

    fn pop(&mut self) {
        self.cursor = self.cstack.pop().expect("stack empty");
        self.state = self.sstack.pop().expect("stack empty");
    }

    fn fall(&mut self) {
        self.push();
        self.state = State::Flow;

        if '.' == self.peek(0, 0) {
            self.mv(0, 0);
        }

        loop {
            if self.cursor.1 as usize == self.height - 1 {
                self.pop();
                return;
            }

            match self.peek(0, 1) {
                '.' => self.mv(0, 1),
                '#' | '~' => break,
                '|' => {
                    self.pop();
                    return;
                }
                _ => unimplemented!(),
            }
        }

        self.flow();

        self.pop();
    }

    fn flow(&mut self) {
        self.push();
        self.state = State::Flow;

        let mut should_fill = true;
        for d in &[-1, 1] {
            self.push();
            loop {
                match (self.peek(*d, 0), self.peek(*d, 1)) {
                    (_, '.') | ('#', _) => break,
                    ('~', _) => {
                        self.pop();
                        self.pop();
                        return;
                    }
                    _ => self.mv(*d, 0),
                }
            }

            match (self.peek(*d, 0), self.peek(*d, 1)) {
                (_, '.') => {
                    should_fill = false;
                    self.mv(*d, 0);
                    self.fall();
                }
                ('#', _) => (),
                _ => unimplemented!(),
            }
            self.pop();
        }

        if should_fill {
            self.fill();
            self.mv(0, -1);
            self.flow();
        }

        self.pop();
    }

    fn fill(&mut self) {
        self.push();
        self.state = State::Fill;

        self.mv(0, 0);
        for d in &[-1, 1] {
            self.push();
            while '#' != self.peek(*d, 0) {
                self.mv(*d, 0);
            }
            self.pop();
        }

        self.pop();
    }
}

impl str::FromStr for Grid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let mut xranges: Vec<(usize, usize, usize)> = Vec::new();
        let mut yranges: Vec<(usize, usize, usize)> = Vec::new();
        let (mut xmin, mut xmax) = (usize::max_value(), 0);
        let (mut ymin, mut ymax) = (usize::max_value(), 0);

        for l in s.lines() {
            let mut token = l.split_whitespace();

            let single = token.next().ok_or(Error::ParseGrid)?;
            let coord: usize = single
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .map_err(|_| Error::ParseGrid)?;

            let double = token.next().ok_or(Error::ParseGrid)?;
            let range: Vec<usize> = double
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .collect::<String>()
                .as_str()
                .split("..")
                .map(|s| s.parse::<usize>())
                .collect::<Result<_, _>>()
                .map_err(|_| Error::ParseGrid)?;

            let (from, to) = (
                *range.get(0).ok_or(Error::ParseGrid)?,
                *range.get(1).ok_or(Error::ParseGrid)?,
            );
            if single.contains('x') {
                xmin = min(xmin, coord);
                xmax = max(xmax, coord);
                ymin = min(ymin, from);
                ymax = max(ymax, to);
                yranges.push((coord, from, to));
            } else {
                ymin = min(ymin, coord);
                ymax = max(ymax, coord);
                xmin = min(xmin, from);
                xmax = max(xmax, to);
                xranges.push((coord, from, to));
            }
        }

        assert!(xmin > 0);
        xmin -= 1;
        xmax += 1;

        let (width, height) = (xmax - xmin + 1, ymax - ymin + 1);
        let buf: Vec<char> = vec!['.'; width * height];
        let mut g = Grid::new(width, height, xmin, buf);

        for (y, from, to) in xranges {
            for x in from..=to {
                g.set(x - xmin, y - ymin, '#');
            }
        }
        for (x, from, to) in yranges {
            for y in from..=to {
                g.set(x - xmin, y - ymin, '#');
            }
        }

        Ok(g)
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;
    let mut grid: Grid = input.parse().expect("paring failed");

    grid.fall();
    println!(
        "Part 1: {}",
        grid.buf.iter().filter(|c| **c == '|' || **c == '~').count()
    );
    println!("Part 2: {}", grid.buf.iter().filter(|c| **c == '~').count());

    Ok(())
}
