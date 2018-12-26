mod util;

use crate::util::Point;

use std::collections::HashMap;
use std::fs;
use std::iter;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tool {
    Neither,
    Torch,
    ClimbingGear,
}

impl Tool {
    fn as_idx(self) -> usize {
        match self {
            Tool::Neither => 0,
            Tool::Torch => 1,
            Tool::ClimbingGear => 2,
        }
    }

    fn iter() -> impl Iterator<Item = Tool> {
        vec![Tool::Neither, Tool::Torch, Tool::ClimbingGear].into_iter()
    }
}

#[derive(Debug, Clone)]
enum RegionType {
    Rocky,
    Wet,
    Narrow,
}

impl RegionType {
    fn visitable_with(&self, tool: Tool) -> bool {
        match (self, tool) {
            (RegionType::Rocky, Tool::Neither)
            | (RegionType::Wet, Tool::Torch)
            | (RegionType::Narrow, Tool::ClimbingGear) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
struct Region {
    pos: Point,
    geo: u64,
    erosion: u64,
    typ: RegionType,

    dists: [u32; 3],
}

#[derive(Debug)]
struct Cave {
    depth: u64,
    target: Point,
    regions: HashMap<Point, Region>,
}

impl Cave {
    fn new(depth: u64, target: Point) -> Self {
        Cave {
            depth,
            target,
            regions: HashMap::new(),
        }
    }

    fn region(&mut self, pos: Point) -> &mut Region {
        if !self.regions.contains_key(&pos) {
            let geo = self.calc_geo(pos);
            let erosion = (geo + self.depth) % 20183;
            let typ = match erosion % 3 {
                0 => RegionType::Rocky,
                1 => RegionType::Wet,
                2 => RegionType::Narrow,
                _ => panic!(),
            };

            self.regions.insert(
                pos,
                Region {
                    pos,
                    geo,
                    erosion,
                    typ,

                    dists: [u32::max_value(); 3],
                },
            );
        }

        self.regions.get_mut(&pos).unwrap()
    }

    fn erosion(&mut self, pos: Point) -> u64 {
        self.region(pos).erosion
    }

    fn danger(&mut self, pos: Point) -> u64 {
        match self.region(pos).typ {
            RegionType::Rocky => 0,
            RegionType::Wet => 1,
            RegionType::Narrow => 2,
        }
    }

    fn calc_geo(&mut self, pos: Point) -> u64 {
        if pos == Point::new(0, 0) || pos == self.target {
            return 0;
        }

        match (pos.x, pos.y) {
            (0, y) => u64::from(y * 48271),
            (x, 0) => u64::from(x * 16807),
            (x, y) => {
                (self.erosion(Point::new(x - 1, y)) * self.erosion(Point::new(x, y - 1))) % 20183
            }
        }
    }

    fn find_min_dist(&mut self) -> u32 {
        let mut frontier: Vec<(u32, Point, Tool)> = vec![(0, Point::new(0, 0), Tool::Torch)];

        let mut time = 0;
        'outer: loop {
            let mut new_frontier = frontier.clone();

            for idx in (0..frontier.len()).rev() {
                let cand = &frontier[idx];

                if cand.0 == time {
                    new_frontier.remove(idx);

                    if (cand.1, cand.2) == (self.target, Tool::Torch) {
                        break 'outer;
                    }

                    for pos in cand.1.nb_iter() {
                        for tool in Tool::iter() {
                            if !self.region(pos).typ.visitable_with(tool)
                                || !self.region(cand.1).typ.visitable_with(tool)
                            {
                                continue;
                            }

                            let d = if tool == cand.2 { time + 1 } else { time + 8 };
                            if d < self.region(pos).dists[tool.as_idx()] {
                                self.region(pos).dists[tool.as_idx()] = d;
                                new_frontier.push((d, pos, tool));
                            }
                        }
                    }
                }
            }

            time += 1;
            frontier = new_frontier;
        }

        time
    }

    #[allow(dead_code)]
    fn print(&mut self, size: u32) {
        for y in 0..=size {
            for x in 0..=size {
                if x == 0 && y == 0 {
                    print!("M");
                } else if Point::new(x, y) == self.target {
                    print!("T");
                } else {
                    print!(
                        "{}",
                        match self.region(Point::new(x, y)).typ {
                            RegionType::Rocky => '.',
                            RegionType::Wet => '=',
                            RegionType::Narrow => '|',
                        }
                    );
                }
            }
            println!();
        }
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let (depth, target) = {
        let mut lines = input.lines();

        let l = lines.next().expect("invalid input");
        let depth: u64 = l
            .chars()
            .skip_while(|c| !c.is_digit(10))
            .collect::<String>()
            .parse()?;

        let l = lines.next().expect("invalid input");
        let x = l
            .chars()
            .skip_while(|c| !c.is_digit(10))
            .take_while(|c| c.is_digit(10))
            .collect::<String>()
            .parse()?;
        let y = l
            .chars()
            .skip_while(|c| *c != ',')
            .skip(1)
            .take_while(|c| c.is_digit(10))
            .collect::<String>()
            .parse()?;

        (depth, Point::new(x, y))
    };
    let mut cave = Cave::new(depth, target);

    let danger: u64 = (0..=target.x)
        .flat_map(|x| iter::repeat(x).zip(0..=target.y))
        .map(|(x, y)| cave.danger(Point::new(x, y)))
        .sum();
    println!("Part 1: {}", danger);

    println!("Part 2: {}", cave.find_min_dist());

    Ok(())
}
