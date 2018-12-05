extern crate chrono;
extern crate regex;

use chrono::NaiveDateTime;
use chrono::Timelike;

use regex::Regex;

use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::str::FromStr;

#[derive(Debug)]
enum Event {
    BeginShift { id: u32 },
    WakeUp,
    FallAsleep,
}

#[derive(Debug)]
struct Record {
    datetime: NaiveDateTime,
    event: Event,
}

#[derive(Debug)]
enum Error {
    Record,
    ParseRecord,
    ParseInt(std::num::ParseIntError),
    ParseRegex(regex::Error),
    ParseDT(chrono::format::ParseError),
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::ParseRegex(err)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(err: chrono::format::ParseError) -> Error {
        Error::ParseDT(err)
    }
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

impl FromStr for Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = Regex::new(r"\[(.*)\] (?:(?P<beg>Guard #(?P<id>\d+) begins shift)|(?P<wak>wakes up)|(?P<slp>falls asleep))")?
            .captures(s)
            .ok_or(Error::ParseRecord)?;

        Ok(Record {
            datetime: NaiveDateTime::parse_from_str(
                caps.get(1).ok_or(Error::ParseRecord)?.into(),
                "%Y-%m-%d %H:%M",
            )?,
            event: if caps.name("beg").is_some() {
                Event::BeginShift {
                    id: caps
                        .name("id")
                        .ok_or(Error::ParseRecord)?
                        .as_str()
                        .parse()?,
                }
            } else if caps.name("wak").is_some() {
                Event::WakeUp
            } else if caps.name("slp").is_some() {
                Event::FallAsleep
            } else {
                return Err(Error::ParseRecord);
            },
        })
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut records: Vec<Record> = input.lines().map(|l| l.parse()).collect::<Result<_, _>>()?;
    records.sort_by(|lhs, rhs| lhs.datetime.cmp(&rhs.datetime));

    let mut sleepy_times = HashMap::new();
    {
        let (mut on_duty, mut from) = (0, 0);
        for rec in &records {
            match rec.event {
                Event::BeginShift { id } => on_duty = id,
                Event::FallAsleep => from = rec.datetime.minute(),
                Event::WakeUp => {
                    let until = rec.datetime.minute();
                    for i in from..until {
                        sleepy_times.entry(on_duty).or_insert(vec![0u32; 60])[i as usize] += 1;
                    }
                }
            }
        }
    }

    let (guard, minute) = sleepy_times
        .iter()
        .max_by_key::<u32, _>(|(_, vs)| vs.iter().sum())
        .map(|(&k, vs)| (k, vs.iter().enumerate().max_by_key(|(_, &v)| v).unwrap().0))
        .ok_or(Error::Record)?;

    println!("=== Strategy #1");
    println!("Sleepiest guard: {}", guard);
    println!("Sleepiest minute: {}", minute);
    println!("Product: {}", guard * minute as u32);

    let (guard, minute) = sleepy_times
        .iter()
        .max_by_key(|(_, vs)| vs.iter().max())
        .map(|(&k, vs)| (k, vs.iter().enumerate().max_by_key(|(_, &v)| v).unwrap().0))
        .ok_or(Error::Record)?;

    println!("=== Strategy #2");
    println!("Sleepiest guard: {}", guard);
    println!("Sleepiest minute: {}", minute);
    println!("Product: {}", guard * minute as u32);

    Ok(())
}
