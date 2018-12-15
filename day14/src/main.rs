const INPUT: usize = 030121;
const TARGET: &[usize] = &[0, 3, 0, 1, 2, 1];

fn p1() {
    let mut scores = vec![3, 7];
    let mut current = (0, 1);

    while scores.len() < INPUT + 10 {
        let sum = scores[current.0] + scores[current.1];
        let (new1, new2) = (sum / 10, sum % 10);

        if new1 > 0 {
            scores.push(new1);
        }
        scores.push(new2);

        current.0 = (1 + current.0 + scores[current.0]) % scores.len();
        current.1 = (1 + current.1 + scores[current.1]) % scores.len();
    }

    print!("Part 1: ");
    for s in &scores[INPUT..] {
        print!("{}", s);
    }
    println!();
}

fn push_check(cands: &mut Vec<usize>, v: usize, vs: &mut Vec<usize>) -> bool {
    vs.push(v);

    cands.push(0);
    *cands = cands
        .iter()
        .filter_map(|i| {
            if v == TARGET[*i] {
                Some(*i + 1)
            } else {
                None
            }
        })
        .collect();

    cands.iter().any(|i| *i == TARGET.len())
}

fn p2() {
    let mut scores = vec![3, 7];
    let mut current = (0, 1);

    let mut cands = Vec::new();
    loop {
        let sum = scores[current.0] + scores[current.1];
        let (new1, new2) = (sum / 10, sum % 10);

        if new1 > 0 {
            if push_check(&mut cands, new1, &mut scores) {
                break;
            }
        }
        if push_check(&mut cands, new2, &mut scores) {
            break;
        }

        current.0 = (1 + current.0 + scores[current.0]) % scores.len();
        current.1 = (1 + current.1 + scores[current.1]) % scores.len();
    }

    println!("Part 2: {}", scores.len() - TARGET.len());
}

fn main() {
    p1();
    p2();
}
