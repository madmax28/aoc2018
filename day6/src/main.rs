use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::iter;
use std::result;
use std::str::FromStr;

type Result<T> = result::Result<T, Box<Error>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Coord {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Bound {
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
}

impl Coord {
    fn distance(&self, other: &Coord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn closest(&self, others: &[Coord]) -> Option<Coord> {
        let min = others.iter().map(|o| self.distance(o)).min()?;
        let candidates: Vec<&Coord> = others.iter().filter(|o| self.distance(o) == min).collect();

        if candidates.len() == 1 {
            Some(candidates[0].clone())
        } else {
            None
        }
    }

    fn on_bound(&self, b: &Bound) -> bool {
        self.x == b.xmin || self.y == b.ymin || self.x == b.xmax || self.y == b.ymax
    }
}

impl FromStr for Coord {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        let coords: Vec<_> = s.split(' ').collect();

        Ok(Coord {
            x: coords[0].replace(",", "").parse()?,
            y: coords[1].parse()?,
        })
    }
}

#[derive(Debug)]
struct Grid {
    coords: Vec<Coord>,
    bound: Bound,
}

impl Grid {
    fn new(coords: &[Coord]) -> Self {
        let (xs, ys): (Vec<_>, Vec<_>) = coords.iter().map(|p| (p.x, p.y)).unzip();
        Grid {
            coords: coords.to_vec(),
            bound: Bound {
                xmin: *xs.iter().min().unwrap_or(&0),
                xmax: *xs.iter().max().unwrap_or(&0),
                ymin: *ys.iter().min().unwrap_or(&0),
                ymax: *ys.iter().max().unwrap_or(&0),
            },
        }
    }

    fn on_edge(&self, p: &Coord) -> bool {
        p.on_bound(&self.bound)
    }

    fn part1(&self) -> usize {
        let (edge, center): (Vec<_>, Vec<_>) = (self.bound.xmin..=self.bound.xmax)
            .flat_map(|x| iter::repeat(x).zip(self.bound.ymin..=self.bound.ymax))
            .map(|(x, y)| Coord { x, y })
            .partition(|c| self.on_edge(c));

        let finites: HashSet<Coord> = {
            let infinites: HashSet<Coord> = edge
                .iter()
                .filter_map(|c| {
                    if let Some(c) = c.closest(&self.coords) {
                        Some(c)
                    } else {
                        None
                    }
                })
                .collect();
            self.coords
                .iter()
                .filter(|c| !infinites.contains(c))
                .cloned()
                .collect()
        };

        let closest: Vec<Coord> = center
            .iter()
            .filter_map(|c| {
                if let Some(c) = c.closest(&self.coords) {
                    if finites.contains(&c) {
                        Some(c)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut largest_area = 0;
        for c1 in &self.coords {
            largest_area =
                std::cmp::max(largest_area, closest.iter().filter(|c2| c1 == *c2).count());
        }
        largest_area
    }

    fn part2(&self) -> usize {
        let start = Coord {
            x: (self.bound.xmax + self.bound.xmin) / 2,
            y: (self.bound.ymax + self.bound.ymin) / 2,
        };

        let mut area = 0;
        for inc in 0.. {
            let bound = Bound {
                xmin: start.x - inc,
                xmax: start.x + inc,
                ymin: start.y - inc,
                ymax: start.y + inc,
            };

            let cs: Vec<_> = (start.x - inc..=start.x + inc)
                .flat_map(|x| std::iter::repeat(x).zip(start.y - inc..=start.y + inc))
                .filter_map(|(x, y)| {
                    let c = Coord { x, y };
                    if c.on_bound(&bound) {
                        if self
                            .coords
                            .iter()
                            .map(|coord| c.distance(coord))
                            .sum::<i32>()
                            < 10000
                        {
                            Some(c)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            area += cs.len();

            if cs.is_empty() && inc > start.x + 1 {
                break;
            }
        }
        area
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let coords: Vec<Coord> = input.lines().map(|l| l.parse()).collect::<Result<_>>()?;

    let grid = Grid::new(&coords);
    println!("Part1 area: {}", grid.part1());
    println!("Part2 area: {}", grid.part2());

    Ok(())
}
