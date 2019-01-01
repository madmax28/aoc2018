use regex::Regex;

use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::str;

#[derive(Debug)]
enum Error {
    Parse,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Faction {
    ImmuneSystem,
    Infection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtkType {
    Slashing,
    Bludgeoning,
    Fire,
    Cold,
    Radiation,
}

impl str::FromStr for AtkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "slashing" => AtkType::Slashing,
            "bludgeoning" => AtkType::Bludgeoning,
            "fire" => AtkType::Fire,
            "cold" => AtkType::Cold,
            "radiation" => AtkType::Radiation,
            _ => return Err(Error::Parse),
        })
    }
}

#[derive(Debug, Clone)]
struct Group {
    id: usize,
    faction: Faction,
    size: u32,
    hp: u32,
    dmg: u32,
    init: u32,
    atk_type: AtkType,
    weak: Vec<AtkType>,
    immune: Vec<AtkType>,
}

impl Group {
    fn calc_damage(&self, other: &Group) -> u32 {
        if other.immune.contains(&self.atk_type) {
            0
        } else if other.weak.contains(&self.atk_type) {
            2 * self.size * self.dmg
        } else {
            self.size * self.dmg
        }
    }

    fn eff_power(&self) -> u32 {
        self.size * self.dmg
    }
}

#[derive(Debug, Clone)]
struct Battle {
    groups: HashMap<usize, Group>,
}

impl Battle {
    fn play_turn(&mut self) {
        let mut targets: HashMap<usize, usize> = HashMap::new();
        let mut attacker_ids: Vec<usize> = self.groups.keys().cloned().collect();
        {
            // Targeting phase
            attacker_ids.sort_by_key(|id| {
                let g = &self.groups[id];
                (g.eff_power(), g.init)
            });

            for aid in attacker_ids.iter().rev() {
                let attacker = &self.groups[aid];

                if let Some(defender) = self
                    .groups
                    .values()
                    .filter(|g| g.faction != attacker.faction)
                    .filter(|g| !targets.values().any(|id| *id == g.id))
                    .max_by_key(|g| (attacker.calc_damage(g), g.eff_power(), g.init))
                {
                    if attacker.calc_damage(defender) == 0 {
                        continue;
                    }

                    targets.insert(attacker.id, defender.id);
                }
            }
        }

        {
            // Attacking phase
            attacker_ids.sort_by_key(|id| self.groups[id].init);
            for attacker_id in attacker_ids.iter().rev() {
                if let Some(target_id) = targets.get(attacker_id) {
                    let killed = if let (Some(attacker), Some(target)) =
                        (self.groups.get(attacker_id), self.groups.get(target_id))
                    {
                        let dmg = attacker.calc_damage(target);
                        dmg / target.hp
                    } else {
                        continue;
                    };

                    let dead = {
                        let target = self.groups.get_mut(target_id).unwrap();
                        if killed >= target.size {
                            target.size = 0;
                            true
                        } else {
                            target.size -= killed;
                            false
                        }
                    };

                    if dead {
                        self.groups.remove(target_id);
                    }
                }
            }
        }
    }

    fn winner(&self) -> Option<Faction> {
        let (immune_sys, infection): (Vec<_>, Vec<_>) = self
            .groups
            .values()
            .partition(|g| g.faction == Faction::ImmuneSystem);

        if immune_sys.is_empty() {
            Some(Faction::Infection)
        } else if infection.is_empty() {
            Some(Faction::ImmuneSystem)
        } else {
            None
        }
    }

    fn count_units(&self) -> u32 {
        self.groups.values().map(|g| g.size).sum()
    }

    fn boost(&mut self, size: u32) {
        for g in self
            .groups
            .values_mut()
            .filter(|g| g.faction == Faction::ImmuneSystem)
        {
            g.dmg += size;
        }
    }
}

fn main() -> Result<(), Box<error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut id = 1;
    let mut groups: HashMap<usize, Group> = HashMap::new();
    {
        let re = Regex::new(r"^(?P<size>\d+) units each with (?P<hp>\d+) hit points (\((?P<attr>.*)\) )?with an attack that does (?P<dmg>\d+) (?P<type>[a-z]+) damage at initiative (?P<init>\d+)$")?;
        let mut faction = Faction::ImmuneSystem;
        for line in input.lines() {
            if line.starts_with("Immune System") || line == "" {
                faction = Faction::ImmuneSystem;
                continue;
            } else if line.starts_with("Infection") {
                faction = Faction::Infection;
                continue;
            }

            let caps = re.captures(line).ok_or(Error::Parse)?;

            let (mut weak, mut immune) = (Vec::new(), Vec::new());
            if let Some(m) = caps.name("attr") {
                for attr in m.as_str().split(';') {
                    let words: Vec<_> = attr
                        .split_whitespace()
                        .map(|w| w.trim_matches(','))
                        .collect();
                    if words.len() < 3 {
                        return Err(Box::new(Error::Parse));
                    }

                    for t in &words[2..] {
                        match words[0] {
                            "weak" => weak.push(t.parse()?),
                            "immune" => immune.push(t.parse()?),
                            _ => return Err(Box::new(Error::Parse)),
                        }
                    }
                }
            }

            let group = Group {
                id,
                faction,
                size: caps.name("size").ok_or(Error::Parse)?.as_str().parse()?,
                hp: caps.name("hp").ok_or(Error::Parse)?.as_str().parse()?,
                dmg: caps.name("dmg").ok_or(Error::Parse)?.as_str().parse()?,
                init: caps.name("init").ok_or(Error::Parse)?.as_str().parse()?,
                atk_type: caps.name("type").ok_or(Error::Parse)?.as_str().parse()?,
                weak,
                immune,
            };
            groups.insert(id, group);
            id += 1;
        }
    }

    let battle = Battle { groups };

    let mut b = battle.clone();
    while b.winner().is_none() {
        b.play_turn();
    }
    println!("Part 1: {}", b.count_units());

    let mut units_left = 0;
    let (mut boost, mut step) = (5000, 5000);
    while step > 0 {
        step /= 2;

        let mut b = battle.clone();
        b.boost(boost);
        let winner = {
            let mut w = None;
            let mut turns = 0;
            while w.is_none() && turns < 10_000 {
                b.play_turn();
                w = b.winner();
                turns += 1;
            }
            w
        };

        match winner {
            Some(Faction::ImmuneSystem) => {
                units_left = b.count_units();
                boost -= step;
            }
            Some(Faction::Infection) => boost += step,
            None => boost += step,
        }
    }
    println!("Part 2: {}", units_left);

    Ok(())
}
