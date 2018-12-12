use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    InvalidInput,
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

#[derive(Debug)]
struct PotSet {
    state: Vec<char>,
    offset: i64,
    generation: u64,
    rules: HashMap<Vec<char>, char>,
    states_seen: HashMap<Vec<char>, (u64, i64)>,
}

impl FromStr for PotSet {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut lines = s.lines();

        let state: Vec<char> = lines
            .next()
            .ok_or(Error::InvalidInput)?
            .replace("initial state: ", "")
            .chars()
            .collect();

        let rules: HashMap<Vec<char>, char> = lines
            .skip(1)
            .map(|l| {
                let mut token = l.split(" => ");
                Ok((
                    token.next().ok_or(Error::InvalidInput)?.chars().collect(),
                    token
                        .next()
                        .ok_or(Error::InvalidInput)?
                        .chars()
                        .next()
                        .ok_or(Error::InvalidInput)?,
                ))
            })
            .collect::<Result<_>>()?;

        Ok(PotSet::new(state, rules))
    }
}

impl PotSet {
    fn new(state: Vec<char>, rules: HashMap<Vec<char>, char>) -> Self {
        let mut ps = PotSet {
            state,
            offset: 0,
            generation: 0,
            rules,
            states_seen: HashMap::new(),
        };
        ps.states_seen.insert(ps.state.clone(), (0, 0));
        ps
    }

    fn tick(&mut self, n: u64) {
        let target_gen = self.generation + n;
        while self.generation < target_gen {
            debug_assert!(self.generation <= target_gen);

            {
                let mut tmp: Vec<char> = Vec::with_capacity(self.state.len() + 4);
                let mut pattern: Vec<char> = vec!['.'; 5];
                self.generation += 1;
                self.offset -= 2;
                for idx in 0..self.state.len() + 4 {
                    pattern.rotate_left(1);
                    pattern[4] = *self.state.get(idx).unwrap_or(&'.');

                    let cand = self.rules.get(&pattern).unwrap_or(&'.');
                    if cand == &'.' && tmp.is_empty() {
                        // Skip leading '.'
                        self.offset += 1;
                    } else {
                        tmp.push(*cand);
                    }
                }
                // Trim trailing '.'
                while let Some('.') = tmp.last() {
                    tmp.pop();
                }
                self.state = tmp;
            }

            if let Some((gen, offset)) = self.states_seen.get(&self.state) {
                let recursion_length = self.generation - gen;
                let step = (target_gen - self.generation) / recursion_length;
                self.generation += step;
                self.offset += step as i64 * (self.offset - *offset);
            } else {
                self.states_seen
                    .insert(self.state.clone(), (self.generation, self.offset));
            }
        }
    }

    #[allow(dead_code)]
    fn print(&self) {
        println!("{}", self.state.iter().collect::<String>());
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;

    let mut pots: PotSet = input.parse()?;
    pots.tick(20);
    println!(
        "Part 1: {}",
        pots.state
            .iter()
            .enumerate()
            .fold(0i64, |acc, (idx, c)| if *c != '.' {
                acc + idx as i64 + pots.offset as i64
            } else {
                acc
            })
    );

    pots.tick(50_000_000_000 - 20);
    println!(
        "Part 2: {}",
        pots.state
            .iter()
            .enumerate()
            .fold(0i64, |acc, (idx, c)| if *c != '.' {
                acc + idx as i64 + pots.offset as i64
            } else {
                acc
            })
    );

    Ok(())
}
