extern crate regex;

use regex::Regex;
use std::fs;
use std::num::ParseIntError;
use std::str;
use std::str::FromStr;

struct Claim {
    id: usize,
    pos: (usize, usize),
    size: (usize, usize),
}

impl FromStr for Claim {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
        let caps = match re.captures(s) {
            Some(c) => c,
            None => panic!("Error matching: {}", s),
        };

        Ok(Claim {
            id: caps.get(1).unwrap().as_str().parse()?,
            pos: (
                caps.get(2).unwrap().as_str().parse()?,
                caps.get(3).unwrap().as_str().parse()?,
            ),
            size: (
                caps.get(4).unwrap().as_str().parse()?,
                caps.get(5).unwrap().as_str().parse()?,
            ),
        })
    }
}

const SHEET_SIZE: usize = 1000;

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;
    let claims: Vec<Claim> = input.lines().map(|s| s.parse()).collect::<Result<_, _>>()?;

    let mut frequencies = vec![0usize; SHEET_SIZE * SHEET_SIZE];
    for c in &claims {
        for x in c.pos.0..c.pos.0 + c.size.0 {
            for y in c.pos.1..c.pos.1 + c.size.1 {
                frequencies[x + y * SHEET_SIZE] += 1;
            }
        }
    }

    println!(
        "Square inches: {}",
        frequencies.iter().filter(|&&n| n > 1).count()
    );

    let intact = match claims.iter().find(|&c| {
        for x in c.pos.0..c.pos.0 + c.size.0 {
            for y in c.pos.1..c.pos.1 + c.size.1 {
                if frequencies[x + y * SHEET_SIZE] > 1 {
                    return false;
                }
            }
        }
        true
    }) {
        Some(c) => c,
        None => panic!("No non-overlapping claim found"),
    };

    println!("ID of non-overlapping claim: {}", intact.id);

    Ok(())
}
