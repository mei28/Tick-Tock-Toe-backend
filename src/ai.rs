use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct AiPlayer {
    pub difficulty: Difficulty,
    pub q_learning: Option<QLearning>,
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty) -> Self {
        let q_learning = if difficulty == Difficulty::Hard {
            Some(QLearning::new(0.1, 0.9, 0.5, "q_table.json")) // Q学習のためのパラメータ設定
        } else {
            None
        };
        Self {
            difficulty,
            q_learning,
        }
    }

    pub fn make_move(&mut self, game_state: &mut GameState) -> Option<(usize, usize)> {
        match self.difficulty {
            Difficulty::Easy => self.random_move(game_state),
            Difficulty::Medium => self.minimax_move(game_state, false),
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

    fn q_learning_move(&mut self, game_state: &GameState) -> Option<(usize, usize)> {
        let state = format!("{:?}", game_state.board);
        let mut rng = thread_rng();

        if rng.gen::<f32>() < self.q_learning.as_ref().unwrap().epsilon {
            game_state.available_moves().choose(&mut rng).cloned()
        } else {
            self.q_learning
                .as_mut()?
                .select_best_action(&state, game_state)
        }
    }
}

// QLearning structは同様に定義
pub struct QLearning {
    q_table: HashMap<String, HashMap<(usize, usize), f32>>,
    alpha: f32,
    gamma: f32,
    pub epsilon: f32,
}

impl QLearning {
    pub fn new(alpha: f32, gamma: f32, epsilon: f32, q_table_file: &str) -> Self {
        let q_table = Self::load_q_table(q_table_file).unwrap_or_default();
        Self {
            q_table,
            alpha,
            gamma,
            epsilon,
        }
    }

    fn load_q_table(file_path: &str) -> io::Result<HashMap<String, HashMap<(usize, usize), f32>>> {
        if let Ok(contents) = fs::read_to_string(file_path) {
            Ok(serde_json::from_str(&contents)?)
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_q_table(&self, file_path: &str) -> io::Result<()> {
        let contents = serde_json::to_string(&self.q_table)?;
        let mut file = File::create(file_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    fn get_q_value(&self, state: &str, action: (usize, usize)) -> f32 {
        *self
            .q_table
            .get(state)
            .and_then(|actions| actions.get(&action))
            .unwrap_or(&0.0)
    }

    pub fn update_q_value(
        &mut self,
        state: &str,
        action: (usize, usize),
        reward: f32,
        next_state: &str,
    ) {
        let next_max_q = *self
            .q_table
            .get(next_state)
            .unwrap_or(&HashMap::new())
            .values()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&0.0);

        let q_value = self.get_q_value(state, action);
        let new_q_value = q_value + self.alpha * (reward + self.gamma * next_max_q - q_value);

        self.q_table
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action, new_q_value);
    }

    pub fn select_best_action(
        &mut self,
        state: &str,
        game_state: &GameState,
    ) -> Option<(usize, usize)> {
        let available_moves = game_state.available_moves();
        let actions = self.q_table.get(state)?;

        available_moves.into_iter().max_by(|&a, &b| {
            actions
                .get(&a)
                .unwrap_or(&0.0)
                .partial_cmp(&actions.get(&b).unwrap_or(&0.0))
                .unwrap()
        })
    }

    pub fn decay_epsilon(&mut self) {
        if self.epsilon > 0.01 {
            self.epsilon *= 0.995;
        }
    }
}
