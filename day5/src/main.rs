use std::fs;

fn react(s: &[u8]) -> Vec<u8> {
    let mut result = s.to_owned();
    loop {
        let bounds = &result
            .windows(2)
            .enumerate()
            .filter_map(|(idx, vs)| {
                if (vs[0] as i32 - vs[1] as i32).abs() == 32 {
                    Some(idx)
                } else {
                    None
                }
            }).collect::<Vec<_>>();

        if bounds.is_empty() {
            break;
        }

        let mut prev = result.len();
        for b in bounds.iter().rev() {
            if b + 1 < prev {
                result.drain(b..&(b + 2));
                prev = *b;
            }
        }
    }
    result
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read("input")?;

    let units = react(&input).len();
    println!("Part 1: units remaining: {}", units);

    let mut min = input.len();
    for c in 'A' as u8..'Z' as u8 {
        min = std::cmp::min(
            min,
            react(
                &input
                    .iter()
                    .cloned()
                    .filter(|&d| d != c && d != c + 32)
                    .collect::<Vec<_>>(),
            ).len(),
        );
    }
    println!("Part 2: units remaining: {}", min);

    Ok(())
}
