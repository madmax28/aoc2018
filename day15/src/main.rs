use std::error;
use std::fmt;
use std::fs;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    InvalidInput,
    PointNotOnMap,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> usize {
        let mut dist = 0;
        if self.x > other.x {
            dist += self.x - other.x;
        } else {
            dist += other.x - self.x;
        }
        if self.y > other.y {
            dist += self.y - other.y;
        } else {
            dist += other.y - self.y;
        }
        dist
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone)]
struct Map {
    size: Point,
    buf: Vec<char>,
}

impl FromStr for Map {
    type Err = Box<error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut lines = s.lines();
        let width = lines.next().ok_or(Error::InvalidInput)?.len();
        let height = lines.count() + 1;

        Ok(Map {
            size: Point::new(width, height),
            buf: s.chars().filter(|c| *c != '\n').collect(),
        })
    }
}

impl Map {
    fn get(&self, p: Point) -> Option<&char> {
        self.buf.get(p.x + p.y * self.size.x)
    }

    fn set(&mut self, p: Point, c: char) -> Result<()> {
        if p.x >= self.size.x || p.y >= self.size.y {
            return Err(Box::new(Error::PointNotOnMap));
        }
        self.buf[p.x + p.y * self.size.x] = c;
        Ok(())
    }

    fn iter(&self) -> impl Iterator<Item = (Point, char)> + '_ {
        MapIter::new(&self)
    }

    fn neighbor_iter(&self, p: Point) -> impl Iterator<Item = Point> {
        let mut ns: Vec<Point> = Vec::new();
        if p.y > 0 {
            ns.push(Point::new(p.x, p.y - 1));
        }
        if p.x > 0 {
            ns.push(Point::new(p.x - 1, p.y));
        }
        if p.x < self.size.x {
            ns.push(Point::new(p.x + 1, p.y));
        }
        if p.y < self.size.y {
            ns.push(Point::new(p.x, p.y + 1));
        }
        ns.into_iter()
    }

    fn is_walkable(&self, p: Point) -> bool {
        if let Some(c) = self.get(p) {
            *c == '.'
        } else {
            false
        }
    }

    fn find_path(&self, from: Point, to: Point) -> Option<(Point, usize)> {
        if from.distance(&to) == 0 {
            return Some((to, 0));
        }

        let mut state: Vec<(Point, usize)> = Vec::new();
        let mut cands = vec![(to, 0usize)];
        while !cands.is_empty() {
            let mut tmp: Vec<_> = cands
                .iter()
                .filter(|(p, _)| p.distance(&from) == 1)
                .collect();
            tmp.sort_by_key(|(p, _)| (p.y, p.x));
            if let Some((goto, dist)) = tmp.iter().next() {
                return Some((*goto, *dist + 1));
            }

            state.extend(&cands);
            let mut cands_new = Vec::new();
            for (cand_pos, d) in cands.iter() {
                for n_pos in self
                    .neighbor_iter(*cand_pos)
                    .filter(|p| self.is_walkable(*p))
                    .filter(|p| !state.iter().any(|(p2, _)| *p2 == *p))
                {
                    if !cands_new.contains(&(n_pos, d + 1)) {
                        cands_new.push((n_pos, d + 1));
                    }
                }
            }
            cands = cands_new;
        }

        None
    }

    #[allow(dead_code)]
    fn print(&self) {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                print!("{}", self.buf[x + y * self.size.x]);
            }
            println!();
        }
    }
}

struct MapIter<'a> {
    p: Point,
    m: &'a Map,
}

impl<'a> MapIter<'a> {
    fn new(m: &'a Map) -> Self {
        MapIter {
            p: Point::new(0, 0),
            m,
        }
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = (Point, char);

    fn next(&mut self) -> Option<Self::Item> {
        let c = *self.m.get(self.p)?;
        let pos = self.p;

        self.p.x += 1;
        if self.p.x == self.m.size.x {
            self.p.x = 0;
            self.p.y += 1;
        }

        Some((pos, c))
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Type {
    Elf,
    Goblin,
}

impl Type {
    fn as_char(&self) -> char {
        match self {
            Type::Elf => 'E',
            Type::Goblin => 'G',
        }
    }
}

#[derive(Clone)]
struct Unit {
    id: u32,
    pos: Point,
    t: Type,
    health: i32,
    power: i32,
}

fn unit_id_to_idx(units: &[Unit], id: u32) -> usize {
    units
        .iter()
        .enumerate()
        .find(|(_, u)| u.id == id)
        .expect("unit not found")
        .0
}

impl Unit {
    fn new(id: u32, pos: Point, t: Type) -> Self {
        Unit {
            id,
            pos,
            t,
            health: 200,
            power: 3,
        }
    }

    fn play_turn(&self, map: &mut Map, units: &mut Vec<Unit>) -> Result<()> {
        let my_idx = unit_id_to_idx(&units, self.id);

        // Movement
        if let Some((_, (_, (goto, dist)))) = units
            .iter()
            .filter(|u| u.t != self.t)
            .filter_map(|u| {
                Some((
                    u,
                    map.neighbor_iter(u.pos)
                        .filter(|p| map.is_walkable(*p) || *p == units[my_idx].pos)
                        .filter_map(|p| Some((p, map.find_path(units[my_idx].pos, p)?)))
                        .min_by_key(|(_, (p, d))| (*d, p.y, p.x))?,
                ))
            })
            .min_by_key(|(_, (p, (_, d)))| (*d, p.y, p.x))
        {
            if dist > 0 {
                map.set(units[my_idx].pos, '.')?;
                units[my_idx].pos = goto;
                map.set(units[my_idx].pos, units[my_idx].t.as_char())?;
            }
        } else {
            return Ok(());
        }

        // Attack
        if let Some(target_idx) = units
            .iter()
            .filter(|u| u.id != units[my_idx].id && u.t != units[my_idx].t)
            .filter(|u| u.pos.distance(&units[my_idx].pos) == 1)
            .min_by_key(|u| (u.health, u.pos.y, u.pos.x))
            .and_then(|u| Some(unit_id_to_idx(&units, u.id)))
        {
            units[target_idx].health -= self.power;
            if units[target_idx].health <= 0 {
                map.set(units[target_idx].pos, '.')?;
                units.remove(target_idx);
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {} @ {:?}, HP={:?}",
            self.t, self.id, self.pos, self.health
        )
    }
}

#[derive(Debug, Clone)]
struct Battle {
    map: Map,
    units: Vec<Unit>,
    turns: usize,
}

impl Battle {
    fn new(map: Map) -> Self {
        let mut units = Vec::new();
        let mut id = 0;
        for (p, c) in map.iter() {
            match c {
                'G' => {
                    units.push(Unit::new(id, p, Type::Goblin));
                    id += 1;
                }
                'E' => {
                    units.push(Unit::new(id, p, Type::Elf));
                    id += 1;
                }
                _ => (),
            }
        }

        Battle {
            map,
            units,
            turns: 0,
        }
    }

    fn play_turn(&mut self) -> Result<bool> {
        let mut game_over = false;

        self.units.sort_by_key(|u| (u.pos.y, u.pos.x));
        let mut tmp_units = self.units.clone();
        for unit in &mut self.units {
            if game_over {
                self.units = tmp_units;
                return Ok(game_over);
            }

            if tmp_units.iter().find(|u| u.id == unit.id).is_none() {
                continue;
            }

            unit.play_turn(&mut self.map, &mut tmp_units)?;

            // Check if battle over
            let (elfs, gobos): (Vec<_>, Vec<_>) = tmp_units.iter().partition(|u| u.t == Type::Elf);
            if elfs.is_empty() || gobos.is_empty() {
                game_over = true;
            }
        }
        self.units = tmp_units;

        self.turns += 1;
        Ok(game_over)
    }

    fn set_elf_power(&mut self, power: i32) {
        for unit in &mut self.units {
            if unit.t == Type::Elf {
                unit.power = power;
            }
        }
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("input")?;
    let battle = Battle::new(input.parse()?);
    let mut b = battle.clone();

    while !b.play_turn()? {}

    println!(
        "Part 1: {}",
        b.turns
            * b.units
                .iter()
                .fold(0usize, |acc, u| acc + u.health as usize)
    );

    let mut elf_power = 4;
    let num_elves = battle.units.iter().filter(|u| u.t == Type::Elf).count();
    loop {
        b = battle.clone();
        b.set_elf_power(elf_power);
        while !b.play_turn()? {}
        let num_elves_left = b.units.iter().filter(|u| u.t == Type::Elf).count();
        if num_elves_left == num_elves {
            break;
        }
        elf_power += 1;
    }

    println!(
        "Part 2: {}",
        b.turns
            * b.units
                .iter()
                .fold(0usize, |acc, u| acc + u.health as usize)
    );

    Ok(())
}
