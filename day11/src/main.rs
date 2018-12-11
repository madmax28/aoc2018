use std::cmp::max;
use std::iter::repeat;

fn calc_power(x: i32, y: i32, serial: i32) -> i32 {
    ((x + 10) * y + serial) * (x + 10) / 100 % 10 - 5
}

#[derive(Debug)]
struct Grid {
    size: i32,
    serial: i32,
    coords: Vec<Vec<Option<i32>>>,
}

impl Grid {
    fn new(size: i32, serial: i32) -> Self {
        let mut g = Grid {
            size,
            serial,
            coords: Vec::with_capacity((size * size) as usize),
        };

        for x in 1..=g.size {
            for y in 1..=g.size {
                let cap = g.size - max(x, y) + 1;
                g.coords.push(vec![None; cap as usize]);
            }
        }

        g
    }

    fn power(&mut self, x: i32, y: i32, n: usize) -> i32 {
        if let Some(pow) = self.get(x, y, n) {
            pow
        } else if n == 1 {
            let pow = calc_power(x, y, self.serial);

            self.set(x, y, n, pow);
            pow
        } else if n % 2 == 0 {
            let step = n / 2;
            let mut pow = self.get_or_calc(x, y, step);
            pow += self.get_or_calc(x + (step as i32), y, step);
            pow += self.get_or_calc(x, y + (step as i32), step);
            pow += self.get_or_calc(x + (step as i32), y + (step as i32), step);

            self.set(x, y, n, pow);
            pow
        } else {
            let mut pow = self.get_or_calc(x + 1, y + 1, n - 1);
            pow += self.get_or_calc(x, y, 1);
            for step in 1..(n as i32) {
                pow += self.get_or_calc(x, y + step, 1);
                pow += self.get_or_calc(x + step, y, 1);
            }

            self.set(x, y, n, pow);
            pow
        }
    }

    fn get(&self, x: i32, y: i32, n: usize) -> Option<i32> {
        self.coords[(x - 1 + (y - 1) * self.size) as usize][n - 1]
    }

    fn set(&mut self, x: i32, y: i32, n: usize, pow: i32) {
        self.coords[(x - 1 + (y - 1) * self.size) as usize][n - 1] = Some(pow);
    }

    fn get_or_calc(&mut self, x: i32, y: i32, n: usize) -> i32 {
        if let Some(pow) = self.get(x, y, n) {
            pow
        } else {
            self.power(x, y, n)
        }
    }
}

fn main() {
    let grid_size: i32 = 300;
    let serial: i32 = 8141;
    let mut grid = Grid::new(grid_size, serial);

    let max = (1..=grid_size - 2)
        .flat_map(|x| repeat(x).zip(1..=grid_size - 2))
        .max_by_key(|(x, y)| grid.power(*x, *y, 3))
        .expect("no coords");
    println!("Part 1: {},{}", max.0, max.1);

    let max = (1..=grid_size)
        .flat_map(|sz| repeat(sz).zip(1..=grid_size - sz + 1))
        .flat_map(|(sz, x)| repeat((sz, x)).zip(1..=grid_size - sz + 1))
        .map(|((sz, x), y)| (x, y, sz))
        .max_by_key(|(x, y, sz)| grid.power(*x, *y, *sz as usize))
        .expect("no coords");
    println!("Part 2: {},{},{}", max.0, max.1, max.2);
}
