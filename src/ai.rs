use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct AiPlayer {
    pub difficulty: Difficulty,
    pub max_depth: usize, // Maximum depth for minimax search in Hard mode
    pub evaluate_fn: Box<dyn Fn(&GameState, bool) -> i32>, // Customizable evaluation function
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty, max_depth: usize) -> Self {
        // Define default evaluation function based on difficulty level
        let evaluate_fn: Box<dyn Fn(&GameState, bool) -> i32> = match difficulty {
            Difficulty::Easy => Box::new(Self::basic_evaluation),
            Difficulty::Medium => Box::new(Self::medium_evaluation),
            Difficulty::Hard => Box::new(Self::hard_evaluation),
        };

        println!(
            "Game started with difficulty: {:?}, max_depth:{}",
            difficulty, max_depth
        );

        Self {
            difficulty,
            max_depth,
            evaluate_fn,
        }
    }

    pub fn make_move(&self, game_state: &mut GameState) -> Option<(usize, usize)> {
        match self.difficulty {
            Difficulty::Easy => self.random_move(game_state),
            Difficulty::Medium | Difficulty::Hard => self.minimax_move(game_state),
        }
    }

    fn random_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut rng = thread_rng();
        game_state.available_moves().choose(&mut rng).cloned()
    }

    fn minimax_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_moves = vec![];
        let mut memo = HashMap::new();

        for (x, y) in game_state.available_moves() {
            let mut simulated_state = game_state.clone();
            simulated_state.place_piece(x, y);

            let depth = if self.difficulty == Difficulty::Hard {
                self.max_depth
            } else {
                3
            };

            let score = self.minimax(&mut simulated_state, false, depth, &mut memo);

            if score > best_score {
                best_score = score;
                best_moves = vec![(x, y)];
            } else if score == best_score {
                best_moves.push((x, y));
            }
        }

        let mut rng = thread_rng();
        best_moves.choose(&mut rng).cloned()
    }

    fn minimax(
        &self,
        game_state: &mut GameState,
        is_maximizing: bool,
        depth: usize,
        memo: &mut HashMap<String, i32>,
    ) -> i32 {
        let board_key = format!("{:?}-{}", game_state.board, is_maximizing);

        if let Some(&score) = memo.get(&board_key) {
            return score;
        }

        if let Some(winner) = &game_state.winner {
            let score = match winner.as_str() {
                "O" => 10,
                "X" => -10,
                _ => 0,
            };
            memo.insert(board_key, score);
            return score;
        } else if game_state.available_moves().is_empty() {
            return 0;
        }

        if depth == 0 {
            return (self.evaluate_fn)(game_state, is_maximizing);
        }

        let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
        for (x, y) in game_state.available_moves() {
            game_state.place_piece(x, y);
            let score = self.minimax(game_state, !is_maximizing, depth - 1, memo);
            game_state.undo_move(x, y);

            best_score = if is_maximizing {
                best_score.max(score)
            } else {
                best_score.min(score)
            };
        }

        memo.insert(board_key, best_score);
        best_score
    }

    fn basic_evaluation(game_state: &GameState, _: bool) -> i32 {
        let win_patterns = [
            [(0, 0), (0, 1), (0, 2)],
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            [(0, 0), (1, 0), (2, 0)],
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            [(0, 0), (1, 1), (2, 2)],
            [(0, 2), (1, 1), (2, 0)],
        ];

        win_patterns
            .iter()
            .map(|pattern| {
                let mut x_count = 0;
                let mut o_count = 0;

                for &(row, col) in pattern {
                    match game_state.board[row][col].as_deref() {
                        Some("X") => x_count += 1,
                        Some("O") => o_count += 1,
                        _ => {}
                    }
                }

                match (x_count, o_count) {
                    (0, 2) => 5,   // Medium priority to block opponent
                    (2, 0) => -3,  // Less priority for self-reach
                    (0, 3) => 10,  // Winning
                    (3, 0) => -10, // Losing
                    _ => 0,
                }
            })
            .sum()
    }

    fn medium_evaluation(game_state: &GameState, _: bool) -> i32 {
        Self::basic_evaluation(game_state, true)
    }

    fn hard_evaluation(game_state: &GameState, _: bool) -> i32 {
        Self::basic_evaluation(game_state, true) * 2 // Custom scoring scale for Hard mode
    }
}
