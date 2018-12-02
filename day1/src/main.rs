use std::collections::HashSet;
use std::fs;
use std::str;

fn main() {
    let input = fs::read("input").unwrap();
    let nums: Vec<i32> = input
        .split(|&c| c as char == '\n')
        .filter_map(|v| str::from_utf8(v).unwrap().parse().ok())
        .collect();

    let mut sum = 0;
    for num in &nums {
        sum += num;
    }

    let first_dup = {
        let mut seen = HashSet::new();
        let mut sum = 0;
        for num in nums.iter().cycle() {
            if seen.contains(&sum) {
                break;
            }
            seen.insert(sum);
            sum += num;
        }
        sum
    };

    println!("Sum: {}", sum);
    println!("First duplicate: {}", first_dup);
}
