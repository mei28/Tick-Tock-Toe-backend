use crate::game::state::GameState;
use crate::three_solver::ThreeSolver;
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
    pub solver: Option<ThreeSolver>,
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty) -> Self {
        let solver = if difficulty == Difficulty::Hard {
            Some(ThreeSolver::new())
        } else {
            None
        };
        Self { difficulty, solver }
    }

    pub fn make_move(&mut self, game_state: &mut GameState) -> Option<(usize, usize)> {
        // 最初の3ターン目までは相手のリーチをブロックする
        if game_state.moves_x.len() + game_state.moves_o.len() < 6 {
            if let Some(block_move) = self.find_block_move(game_state) {
                return Some(block_move);
            }
        }
        match self.difficulty {
            Difficulty::Easy | Difficulty::Medium => {
                // 通常のランダムまたはミニマックスの動きを行う
                if self.difficulty == Difficulty::Easy {
                    self.random_move(game_state)
                } else {
                    self.minimax_move(game_state, false)
                }
            }
            Difficulty::Hard => self.q_learning_move(game_state),
        }
    }

    fn random_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut rng = thread_rng();
        game_state.available_moves().choose(&mut rng).cloned()
    }

    fn minimax_move(
        &self,
        game_state: &GameState,
        is_detailed_eval: bool,
    ) -> Option<(usize, usize)> {
        let mut best_score = i32::MIN;
        let mut best_moves = vec![];
        let mut memo = HashMap::new();

        for (x, y) in game_state.available_moves() {
            let mut simulated_state = game_state.clone();
            simulated_state.place_piece(x, y);

            let score = self.minimax(&mut simulated_state, false, 3, &mut memo, is_detailed_eval);

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
        is_detailed_eval: bool,
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
            memo.insert(board_key, score);
            return score;
        } else if game_state.available_moves().is_empty() {
            return 0;
        }

        if depth == 0 {
            return self.evaluate_position(game_state, is_detailed_eval);
        }

        let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
        for (x, y) in game_state.available_moves() {
            game_state.place_piece(x, y);
            let score = self.minimax(
                game_state,
                !is_maximizing,
                depth - 1,
                memo,
                is_detailed_eval,
            );
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

    fn evaluate_position(&self, game_state: &GameState, _is_detailed_eval: bool) -> i32 {
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

        let mut score = 0;
        for pattern in &win_patterns {
            let (x_count, o_count) = pattern.iter().fold((0, 0), |(x_count, o_count), &(r, c)| {
                match game_state.board[r][c].as_deref() {
                    Some("X") => (x_count + 1, o_count),
                    Some("O") => (x_count, o_count + 1),
                    _ => (x_count, o_count),
                }
            });

            if o_count == 3 {
                score += 100;
            } else if o_count == 2 && x_count == 0 {
                score += 10;
            } else if x_count == 3 {
                score -= 100;
            } else if x_count == 2 && o_count == 0 {
                score -= 10;
            }
        }

        score
    }

    fn find_block_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
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

        // 相手がリーチしている場所を特定し、ブロックする動きを返す
        for pattern in &win_patterns {
            let (x_count, o_count, empty_cell) =
                pattern
                    .iter()
                    .fold(
                        (0, 0, None),
                        |(x_count, o_count, empty), &(r, c)| match game_state.board[r][c].as_deref()
                        {
                            Some("X") => (x_count + 1, o_count, empty),
                            Some("O") => (x_count, o_count + 1, empty),
                            None => (x_count, o_count, Some((r, c))),

                            _ => (x_count, o_count, empty), // ワイルドカードを追加
                        },
                    );

            if x_count == 2 && o_count == 0 {
                return empty_cell;
            }
        }

        None
    }

    pub fn q_learning_move(&mut self, game_state: &GameState) -> Option<(usize, usize)> {
        if let Some(solver) = &mut self.solver {
            let state_key = game_state.to_string();
            let action = solver.select_best_action(&state_key, game_state)?;

            let reward = 0.0; // Placeholder: Compute actual reward based on game state
            let next_state_key = game_state.to_three_state();
            solver.update(&state_key, action, reward, &next_state_key);

            Some(action)
        } else {
            None
        }
    }
}
