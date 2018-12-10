extern crate regex;

use regex::Regex;

use std::error;
use std::fmt;
use std::fs;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    ParseLight,
    NoLights,
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

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Light {
    pos: Point,
    velocity: Point,
}

impl Light {
    fn tick(&mut self, n: i32) {
        self.pos.x += n * self.velocity.x;
        self.pos.y += n * self.velocity.y;
    }
}

impl FromStr for Light {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let re =
            Regex::new(r"position=<\s?(-?\d+),\s\s?(-?\d+)> velocity=<\s?(-?\d+),\s\s?(-?\d+)>")?;
        let caps = re.captures(s).ok_or(Error::ParseLight)?;

        Ok(Light {
            pos: Point {
                x: caps.get(1).ok_or(Error::ParseLight)?.as_str().parse()?,
                y: caps.get(2).ok_or(Error::ParseLight)?.as_str().parse()?,
            },
            velocity: Point {
                x: caps.get(3).ok_or(Error::ParseLight)?.as_str().parse()?,
                y: caps.get(4).ok_or(Error::ParseLight)?.as_str().parse()?,
            },
        })
    }
}

#[derive(Debug)]
struct LightSet {
    lights: Vec<Light>,
}

impl LightSet {
    fn new(lights: Vec<Light>) -> Self {
        LightSet { lights }
    }

    fn find_xdist(&self) -> Result<u32> {
        let xs: Vec<_> = self.lights.iter().map(|l| l.pos.x).collect();
        let xmin = xs.iter().min().ok_or(Error::NoLights)?;
        let xmax = xs.iter().max().ok_or(Error::NoLights)?;

        Ok((xmax - xmin) as u32)
    }

    fn tick(&mut self, n: i32) {
        for l in &mut self.lights {
            l.tick(n);
        }
    }

    fn to_string(&self) -> Result<String> {
        let (xs, ys): (Vec<_>, Vec<_>) = self.lights.iter().map(|l| (l.pos.x, l.pos.y)).unzip();
        let (xmin, xmax, ymin, ymax) = (
            xs.iter().min().ok_or(Error::NoLights)?,
            xs.iter().max().ok_or(Error::NoLights)?,
            ys.iter().min().ok_or(Error::NoLights)?,
            ys.iter().max().ok_or(Error::NoLights)?,
        );

        let (width, height) = ((xmax - xmin + 1) as usize, (ymax - ymin + 1) as usize);
        let mut grid = vec![vec![' '; width]; height];
        for l in &self.lights {
            grid[(l.pos.y - ymin) as usize][(l.pos.x - xmin) as usize] = '*';
        }
        let grid = grid.join(&'\n');

        Ok(grid.into_iter().collect::<String>())
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let lights: Vec<Light> = input.lines().map(|l| l.parse()).collect::<Result<_>>()?;
    let mut lights = LightSet::new(lights);

    let mut seconds = 0;
    let mut dist = lights.find_xdist()?;
    let mut step: i32 = 10000;
    while step != 0 {
        loop {
            lights.tick(step);
            let cand = lights.find_xdist()?;
            if cand < dist {
                seconds += step;
                dist = cand;
            } else {
                lights.tick(-step);
                break;
            }
        }

        if step > 0 {
            step /= -10;
        } else {
            step *= -1;
        }
    }
    println!("Part 1:\n{}", lights.to_string()?);
    println!("Part 2: {} seconds", seconds);

    Ok(())
}
