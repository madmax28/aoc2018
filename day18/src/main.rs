use gif::{Frame, Encoder, Repeat, SetParameter};

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
struct Map {
    width: usize,
    height: usize,
    buf: Vec<char>,

    generation: usize,
    seen: HashMap<Vec<char>, usize>,

    did_visualize: bool,
}

impl Map {
    fn new(width: usize, height: usize, buf: Vec<char>) -> Self {
        Map { width, height, buf, generation: 0, seen: HashMap::new(), did_visualize: false }
    }

    fn tick(&mut self, n: usize) {
        let target_gen = self.generation + n;
        while self.generation < target_gen {
            let mut tmp_buf = self.buf.clone();

            for y in 0..self.height {
                for x in 0..self.width {
                    let neighbors = (y as i32 - 1..=y as i32 + 1)
                        .flat_map(|y2| (x as i32 - 1..=x as i32 + 1).zip(std::iter::repeat(y2)))
                        .filter(|(x2, y2)| {
                            *x2 >= 0
                                && *x2 < self.width as i32
                                && *y2 >= 0
                                && *y2 < self.height as i32
                                && (*x2 != x as i32 || *y2 != y as i32)
                        })
                    .map(|(x, y)| self.buf[x as usize + y as usize * self.width]);

                    match self.buf[x + y * self.width] {
                        '.' => if neighbors.filter(|c| *c == '|').count() >= 3 {
                            tmp_buf[x + y * self.width] = '|';
                        },
                        '|' => if neighbors.filter(|c| *c == '#').count() >= 3 {
                            tmp_buf[x + y * self.width] = '#';
                        },
                        '#' => {
                            let ns: Vec<_> = neighbors.collect();
                            let num_yards = ns.iter().filter(|&c| *c == '#').count();
                            let num_trees = ns.iter().filter(|&c| *c == '|').count();
                            if num_yards == 0 || num_trees == 0 {
                                tmp_buf[x + y * self.width] = '.';
                            }
                        },
                        _ => panic!("invalid character"),
                    }
                }
            }
            self.generation += 1;

            if let Some(gen) = self.seen.get(&tmp_buf) {
                let recursion_len = self.generation - gen;
                let step = (target_gen - self.generation) / recursion_len;
                self.generation += step * recursion_len;

                if !self.did_visualize {
                    self.did_visualize = true;
                    self.create_gif(tmp_buf.clone(), recursion_len);
                }
            } else {
                self.seen.insert(tmp_buf.clone(), self.generation);
            }

            self.buf = tmp_buf;
        }
    }

    fn create_gif(&self, start: Vec<char>, len: usize) {
        let mut tmp_map = self.clone();
        tmp_map.buf = start;

        let outfile = fs::File::create("output.gif").expect("cant create file");
        let color_map = &[
            33, 130, 64, // green
            109, 54, 33, // brown
            0, 0, 0, //black
        ];
        let mut encoder = Encoder::new(outfile, self.width as u16, self.height as u16, color_map).unwrap();
        encoder.set(Repeat::Infinite).unwrap();

        for _ in 0..len {
            tmp_map.tick(1);

            let buf: Vec<_> = tmp_map.buf
                .iter()
                .map(|c| match c {
                    '|' => 0,
                    '.' => 1,
                    '#' => 2,
                    _ => panic!("invalid char"),
                })
                .collect();
            let mut frame = Frame::default();
            frame.width = tmp_map.width as u16;
            frame.height = tmp_map.height as u16;
            frame.buffer = std::borrow::Cow::Owned(buf);
            frame.delay = 5;
            encoder.write_frame(&frame).unwrap();
        }
    }

    #[allow(dead_code)]
    fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}", self.buf[x + y * self.width]);
            }
            println!();
        }
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut map = {
        let (width, height) = (
            input.lines().next().expect("input empty").len(),
            input.lines().count(),
        );

        let buf: Vec<char> = input.chars().filter(|c| *c != '\n').collect();

        Map::new(width, height, buf)
    };

    map.tick(10);
    let num_trees = map.buf.iter().filter(|&c| *c == '|').count();
    let num_yards = map.buf.iter().filter(|&c| *c == '#').count();
    println!("Part 1: {}", num_trees * num_yards);

    map.tick(1_000_000_000 - 10);
    let num_trees = map.buf.iter().filter(|&c| *c == '|').count();
    let num_yards = map.buf.iter().filter(|&c| *c == '#').count();
    println!("Part 2: {}", num_trees * num_yards);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let mut buf = String::new();
        buf += ".#.#...|#.";
        buf += ".....#|##|";
        buf += ".|..|...#.";
        buf += "..|#.....#";
        buf += "#.#|||#|#|";
        buf += "...#.||...";
        buf += ".|....|...";
        buf += "||...#|.#|";
        buf += "|.||||..|.";
        buf += "...#.|..|.";
        let buf: Vec<char> = buf.chars().collect();

        let mut map = Map::new(10, 10, buf);
        map.print();
        for _ in 0..10 {
            println!();
            map.tick(1);
            map.print();
        }

        let num_trees = map.buf.iter().filter(|&c| *c == '|').count();
        let num_yards = map.buf.iter().filter(|&c| *c == '#').count();
        assert_eq!(num_trees, 37);
        assert_eq!(num_yards, 31);
    }
}
