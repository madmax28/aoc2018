use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fs;

type Point = (i32, i32, i32, i32);

fn dist(lhs: &Point, rhs: &Point) -> i32 {
    (lhs.0 - rhs.0).abs() + (lhs.1 - rhs.1).abs() + (lhs.2 - rhs.2).abs() + (lhs.3 - rhs.3).abs()
}

fn main() -> Result<(), Box<error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut clusters = HashMap::new();
    for (idx, line) in input.lines().enumerate() {
        let nums: Vec<i32> = line
            .split(',')
            .map(|s| s.parse())
            .collect::<Result<_, _>>()?;
        assert_eq!(nums.len(), 4);
        clusters.insert(idx, vec![(nums[0], nums[1], nums[2], nums[3])]);
    }

    let mut distances = HashMap::new();
    for from in 0..clusters.len() {
        for to in from + 1..clusters.len() {
            distances.insert((from, to), dist(&clusters[&from][0], &clusters[&to][0]));
        }
    }

    loop {
        let mut new_distances = distances.clone();

        if let Some(((from, to), _)) = distances.iter().find(|(_, &d)| d <= 3) {
            for c in clusters.keys() {
                if c == from || c == to {
                    continue;
                }

                let cand = if c < to {
                    *new_distances.get(&(*c, *to)).expect("edge not found")
                } else {
                    *new_distances.get(&(*to, *c)).expect("edge not found")
                };

                let to_update = if c < from {
                    new_distances.get_mut(&(*c, *from)).expect("edge not found")
                } else {
                    new_distances.get_mut(&(*from, *c)).expect("edge not found")
                };

                *to_update = cmp::min(*to_update, cand);
            }

            let ps = clusters.remove(to).expect("cluster not found");
            clusters
                .get_mut(from)
                .expect("cluster not found")
                .extend(ps);
            new_distances.retain(|(f, t), _| f != to && t != to);
        } else {
            break;
        }

        distances = new_distances
    }

    println!("Part 1: {} clusters", clusters.len());

    Ok(())
}
