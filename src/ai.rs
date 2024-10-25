// src/ai.rs
use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct AiPlayer {
    pub difficulty: Difficulty,
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty) -> Self {
        Self { difficulty }
    }

    pub fn make_move(&self, game_state: &mut GameState) -> Option<(usize, usize)> {
        match self.difficulty {
            Difficulty::Easy => self.random_move(game_state),
            Difficulty::Medium => self.random_move(game_state),
            Difficulty::Hard => self.minimax_move(game_state),
        }
    }

    fn random_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut rng = thread_rng();
        game_state.available_moves().choose(&mut rng).cloned()
    }

    fn minimax_move(&self, game_state: &mut GameState) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_move = None;
        let mut memo = HashMap::new(); // Memoization map to store evaluated states

        for (x, y) in game_state.available_moves() {
            game_state.place_piece(x, y); // Try the move

            // Call minimax with depth limit of 5
            let score = self.minimax(game_state, false, 5, &mut memo);

            game_state.undo_move(x, y); // Undo the move

            if score > best_score {
                best_score = score;
                best_move = Some((x, y));
            }
        }

        best_move
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
                "O" => 1,
                "X" => -1,
                _ => 0,
            };
            memo.insert(board_key, score); // Cache the result
            return score;
        } else if game_state.available_moves().is_empty() {
            return 0; // Draw
        }

        if depth == 0 {
            return 0; // Reached max depth
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
}
