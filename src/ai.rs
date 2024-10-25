use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct AiPlayer {
    pub difficulty: Difficulty,
    pub max_depth: usize, // Hardモード用の探索深さ
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty, max_depth: usize) -> Self {
        Self {
            difficulty,
            max_depth,
        }
    }

    pub fn make_move(&self, game_state: &mut GameState) -> Option<(usize, usize)> {
        match self.difficulty {
            Difficulty::Easy => self.random_move(game_state),
            Difficulty::Medium => self.medium_move(game_state),
            Difficulty::Hard => self.hard_move(game_state),
        }
    }

    fn random_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut rng = thread_rng();
        game_state.available_moves().choose(&mut rng).cloned()
    }

    fn medium_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_moves = vec![];
        let mut memo = HashMap::new(); // メモ化で訪問済み盤面の結果を保存

        for (x, y) in game_state.available_moves() {
            let mut simulated_state = game_state.clone();
            simulated_state.place_piece(x, y);

            // メモ化を使ったミニマックスを行う
            let score = self.minimax(&mut simulated_state, false, usize::MAX, &mut memo);

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

    fn hard_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_moves = vec![];
        let mut memo = HashMap::new();

        for (x, y) in game_state.available_moves() {
            let mut simulated_state = game_state.clone();
            simulated_state.place_piece(x, y);

            // Hardモードで指定された探索深さで制限したミニマックスを行う
            let score = self.minimax(&mut simulated_state, false, self.max_depth, &mut memo);

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
                "O" => 1,
                "X" => -1,
                _ => 0,
            };
            memo.insert(board_key, score); // 結果をメモ化してキャッシュ
            return score;
        } else if game_state.available_moves().is_empty() {
            return 0; // 引き分け
        }

        if depth == 0 {
            return 0; // 探索深さの上限に達した場合
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
