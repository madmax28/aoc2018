use std::clone::Clone;
use std::cmp::PartialEq;
use std::fs;
use std::str;

fn contains_dup_n<T: PartialEq>(n: usize, els: &[T]) -> bool {
    for el in els {
        if els.iter().filter(|&x| x == el).count() == n {
            return true;
        }
    }
    false
}

fn distance<T: PartialEq>(lhs: &[T], rhs: &[T]) -> usize {
    assert_eq!(lhs.len(), rhs.len());
    let mut distance = 0;
    for idx in 0..lhs.len() {
        if lhs[idx] != rhs[idx] {
            distance += 1;
        }
    }
    distance
}

fn intersect<T: PartialEq + Clone>(lhs: &[T], rhs: &[T]) -> Vec<T> {
    assert_eq!(lhs.len(), rhs.len());
    let mut result = Vec::new();
    for idx in 0..lhs.len() {
        if lhs[idx] == rhs[idx] {
            result.push(lhs[idx].clone())
        }
    }
    result
}

fn main() {
    let mut doubles = 0;
    let mut triples = 0;
    let mut intersection = Vec::new();

    let input = fs::read("input").unwrap();
    let ids: Vec<_> = input
        .split(|&c| c as char == '\n')
        .filter(|v| !v.is_empty())
        .collect();

    for (idx, id) in ids.iter().enumerate() {
        if contains_dup_n(2, id) {
            doubles += 1;
        }
        if contains_dup_n(3, id) {
            triples += 1;
        }

        for other in &ids[idx..] {
            if distance(id, other) == 1 {
                intersection = intersect(id, other);
                break;
            }
        }
    }

    println!("Checksum: {}", doubles * triples);
    println!("Intersection: {}", str::from_utf8(&intersection).unwrap());
}
