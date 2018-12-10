use std::fs;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Marble {
    next: usize,
    prev: usize,
}

#[derive(Debug)]
struct MarbleGame {
    scores: Vec<usize>,
    marbles: Vec<Marble>,

    num_turns: usize,
    next_player_idx: usize,
    cur_marble: usize,
}

impl MarbleGame {
    fn new(num_players: usize, num_marbles: usize) -> MarbleGame {
        let mut game = MarbleGame {
            scores: vec![0; num_players],
            marbles: vec![unsafe { std::mem::uninitialized() }; num_marbles + 1],

            num_turns: num_marbles,
            next_player_idx: 0,
            cur_marble: 0,
        };
        game.marbles[0].next = 0;
        game.marbles[0].prev = 0;
        game
    }

    fn play(&mut self) {
        for marble in 1..=self.num_turns {
            if marble % 23 != 0 {
                let left_neighbor = self.marbles[self.cur_marble].next;
                let right_neighbor = self.marbles[left_neighbor].next;

                self.marbles[left_neighbor].next = marble;
                self.marbles[right_neighbor].prev = marble;
                self.marbles[marble].next = right_neighbor;
                self.marbles[marble].prev = left_neighbor;

                self.cur_marble = marble;
            } else {
                for _ in 0..7 {
                    self.cur_marble = self.marbles[self.cur_marble].prev;
                }
                self.scores[self.next_player_idx] += marble + self.cur_marble;

                let left_neighbor = self.marbles[self.cur_marble].prev;
                let right_neighbor = self.marbles[self.cur_marble].next;
                self.marbles[left_neighbor].next = right_neighbor;
                self.marbles[right_neighbor].prev = left_neighbor;

                self.cur_marble = right_neighbor;
            }
            self.next_player_idx += 1;
            if self.next_player_idx == self.scores.len() {
                self.next_player_idx = 0;
            }
        }
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let now = Instant::now();

    let input = fs::read_to_string("input")?;
    let tokens: Vec<usize> = input
        .trim()
        .replace(" players; last marble is worth ", ",")
        .replace(" points", "")
        .split(',')
        .map(|token| token.parse())
        .collect::<Result<_, _>>()?;
    assert_eq!(tokens.len(), 2);
    let (num_players, mut num_marbles) = (tokens[0], tokens[1]);

    let mut game = MarbleGame::new(num_players, num_marbles);
    game.play();
    println!(
        "Part 1 max. score: {}",
        *game.scores.iter().max().expect("no scores")
    );

    num_marbles *= 100;
    let mut game = MarbleGame::new(num_players, num_marbles);
    game.play();
    println!(
        "Part 2 max. score: {}",
        *game.scores.iter().max().expect("no scores")
    );

    let d: Duration = now.elapsed();
    println!("> {}.{:03} seconds", d.as_secs(), d.subsec_millis());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_game(num_players: usize, num_marbles: usize, expected_score: usize) {
        let mut game = MarbleGame::new(num_players, num_marbles);
        game.play();
        assert_eq!(
            *game.scores.iter().max().expect("no scores"),
            expected_score
        );
    }

    #[test]
    fn test_game() {
        check_game(5, 25, 32);
        check_game(10, 1618, 8317);
        check_game(13, 7999, 146373);
        check_game(17, 1104, 2764);
        check_game(21, 6111, 54718);
        check_game(30, 5807, 37305);
    }
}
