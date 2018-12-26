mod util;

use crate::util::Point;

use std::cmp::min;
use std::fs;
use std::collections::HashMap;

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut distances: HashMap<Point, u32> = HashMap::new();
    let mut pos = Point::new(0, 0);
    let mut dist = 0;
    let mut stack: Vec<(Point, u32)> = Vec::new();

    for c in input.chars() {
        match c {
            'N' | 'E' | 'W' | 'S' => {
                pos += Point::from_char(c);
                dist += 1;

                distances.entry(pos)
                    .and_modify(|d| *d = min(*d, dist))
                    .or_insert(dist);
            },
            '(' => {
                stack.push((pos, dist));
            },
            '|' => {
                let entry = *stack.last().expect("stack empty");
                pos = entry.0;
                dist = entry.1;
            },
            ')' => {
                let entry = stack.pop().expect("stack empty");
                pos = entry.0;
                dist = entry.1;
            },
            '^' | '$' | '\n' => (),
            _ => panic!("invalid char"),
        }
    }

    println!("Part 1: {}", distances.values().max().expect("no max dist found"));
    println!("Part 2: {}", distances.values().filter(|&d| *d >= 1000).count());

    Ok(())
}
