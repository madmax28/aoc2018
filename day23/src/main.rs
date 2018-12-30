use std::cmp::{max, min, Ordering};
use std::fmt;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

impl Coord {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Coord { x, y, z }
    }

    fn origin() -> Self {
        Coord::new(0, 0, 0)
    }

    fn dist(&self, other: &Coord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

#[derive(Debug)]
struct Nanobot {
    p: Coord,
    r: i32,
}

// Finds point in interval closest to given value
fn find_closest_1d(min: i32, max: i32, val: i32) -> i32 {
    if max >= val && min <= val {
        val
    } else if (max - val).abs() < (min - val).abs() {
        max
    } else {
        min
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BoundingBox {
    min: Coord,
    max: Coord,
}

impl BoundingBox {
    fn from_nanobots(bots: &[Nanobot]) -> Self {
        let mut bb = BoundingBox {
            min: Coord::new(i32::max_value(), i32::max_value(), i32::max_value()),
            max: Coord::new(i32::min_value(), i32::min_value(), i32::min_value()),
        };

        for bot in bots {
            bb.min.x = min(bb.min.x, bot.p.x - bot.r);
            bb.max.x = max(bb.max.x, bot.p.x + bot.r);
            bb.min.y = min(bb.min.y, bot.p.y - bot.r);
            bb.max.y = max(bb.max.y, bot.p.y + bot.r);
            bb.min.z = min(bb.min.z, bot.p.z - bot.r);
            bb.max.z = max(bb.max.z, bot.p.z + bot.r);
        }

        assert!(bb.min.x <= bb.max.x);
        assert!(bb.min.y <= bb.max.y);
        assert!(bb.min.z <= bb.max.z);

        bb
    }

    fn split(&self) -> Vec<BoundingBox> {
        let mut res = Vec::new();

        let border_x = (self.max.x + self.min.x) / 2;
        let border_y = (self.max.y + self.min.y) / 2;
        let border_z = (self.max.z + self.min.z) / 2;

        let shift_x = if self.min.x != self.max.x { vec![false, true] } else { vec![false] };
        let shift_y = if self.min.y != self.max.y { vec![false, true] } else { vec![false] };
        let shift_z = if self.min.z != self.max.z { vec![false, true] } else { vec![false] };

        for sz in &shift_z {
            for sy in &shift_y {
                for sx in &shift_x {
                    res.push(BoundingBox {
                        min: Coord::new(
                            if *sx { border_x + 1 } else { self.min.x },
                            if *sy { border_y + 1 } else { self.min.y },
                            if *sz { border_z + 1 } else { self.min.z },
                        ),
                        max: Coord::new(
                            if *sx { self.max.x } else { border_x },
                            if *sy { self.max.y } else { border_y },
                            if *sz { self.max.z } else { border_z },
                        ),
                    });
                }
            }
        }

        res
    }

    fn find_closest(&self, c: Coord) -> Coord {
        Coord::new(
            find_closest_1d(self.min.x, self.max.x, c.x),
            find_closest_1d(self.min.y, self.max.y, c.y),
            find_closest_1d(self.min.z, self.max.z, c.z),
        )
    }

    fn intersects(&self, bot: &Nanobot) -> bool {
        self.find_closest(bot.p).dist(&bot.p) <= bot.r
    }

    fn count_intersections(&self, bots: &[Nanobot]) -> usize {
        bots.iter().filter(|bot| self.intersects(bot)).count()
    }
}

impl Ord for BoundingBox {
    fn cmp(&self, other: &BoundingBox) -> Ordering {
        let this_dist = self.find_closest(Coord::origin()).dist(&Coord::origin());
        let other_dist = other.find_closest(Coord::origin()).dist(&Coord::origin());

        if this_dist < other_dist {
            Ordering::Greater
        } else if this_dist > other_dist {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &BoundingBox) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BoundingBox: x = {}..{} y = {}..{} z = {}..{}",
            self.min.x, self.max.x, self.min.y, self.max.y, self.min.z, self.max.z,
        )
    }
}

fn part2(nanobots: &[Nanobot]) -> i32 {
    let bb = BoundingBox::from_nanobots(&nanobots);

    let mut candidates = vec![(bb.count_intersections(&nanobots), bb)];
    loop {
        if let Some((_, cand)) = candidates.pop() {
            if cand.min == cand.max {
                return cand.min.x.abs() + cand.min.y.abs() + cand.min.z.abs();
            } else {
                let children = cand.split();
                candidates.extend(
                    children
                        .into_iter()
                        .map(|c| (c.count_intersections(&nanobots), c)),
                );
            }

            candidates.sort_by_key(|(cnt, bb)| (*cnt, *bb));
        } else {
            println!("No candidates left");
            assert!(false);
        }
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut nanobots = Vec::new();
    for l in input.lines() {
        let nums: Vec<i32> = l
            .chars()
            .skip_while(|c| *c != '<')
            .skip(1)
            .take_while(|c| *c != '>')
            .collect::<String>()
            .split(',')
            .map(|s| s.parse())
            .collect::<Result<_, _>>()?;
        assert_eq!(nums.len(), 3);
        let r: i32 = l
            .chars()
            .skip_while(|c| *c != 'r')
            .skip(2)
            .collect::<String>()
            .parse()?;

        nanobots.push(Nanobot {
            p: Coord::new(nums[0], nums[1], nums[2]),
            r,
        });
    }

    let max_range = nanobots.iter().max_by_key(|n| n.r).unwrap();
    println!(
        "Part 1: {}",
        nanobots
            .iter()
            .filter(|n| max_range.p.dist(&n.p) <= max_range.r)
            .count()
    );

    println!("Part 2: {}", part2(&nanobots));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let nanobots = vec![
            Nanobot { p: Coord::new(10, 12, 12), r: 2 },
            Nanobot { p: Coord::new(12, 14, 12), r: 2 },
            Nanobot { p: Coord::new(16, 12, 12), r: 4 },
            Nanobot { p: Coord::new(14, 14, 14), r: 6 },
            Nanobot { p: Coord::new(50, 50, 50), r: 200 },
            Nanobot { p: Coord::new(10, 10, 10), r: 5 },
        ];

        assert_eq!(part2(&nanobots), 36);
    }
}
