use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct AiPlayer {
    pub difficulty: Difficulty,
    pub max_depth: usize,
    evaluation_table: HashMap<String, i32>,
    q_learning: Option<QLearning>,
}

impl AiPlayer {
    pub fn new(difficulty: Difficulty, evaluation_file: &str, eval_type: &str) -> Self {
        let max_depth = match difficulty {
            Difficulty::Easy => 1,
            Difficulty::Medium => 4,
            Difficulty::Hard => 0,
        };

        let q_learning = if difficulty == Difficulty::Hard {
            Some(QLearning::new(0.1, 0.9, 0.2))
        } else {
            None
        };

        let evaluation_table = Self::load_evaluation_table(evaluation_file, &difficulty, eval_type);

        Self {
            difficulty,
            max_depth,
            evaluation_table,
            q_learning,
        }
    }

    fn load_evaluation_table(
        file_path: &str,
        difficulty: &Difficulty,
        eval_type: &str,
    ) -> HashMap<String, i32> {
        let file_content =
            fs::read_to_string(file_path).expect("Unable to read evaluation table file");
        let json_data: Value = serde_json::from_str(&file_content).expect("Unable to parse JSON");

        let difficulty_str = match difficulty {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        };

        let table_data = json_data[difficulty_str][eval_type]
            .as_object()
            .expect("Expected evaluation table object");

        table_data
            .iter()
            .map(|(key, value)| (key.clone(), value.as_i64().unwrap_or(0) as i32))
            .collect()
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

    fn q_learning_move(&mut self, game_state: &GameState) -> Option<(usize, usize)> {
        let state = format!("{:?}", game_state);
        let mut rng = thread_rng();

        if rand::random::<f32>() < self.q_learning.as_ref().unwrap().epsilon {
            game_state.available_moves().choose(&mut rng).cloned()
        } else {
            game_state.available_moves().into_iter().max_by(|&a, &b| {
                let q_a = self
                    .q_learning
                    .as_mut()
                    .unwrap()
                    .get_q_value(&format!("{}:{:?}", state, a));
                let q_b = self
                    .q_learning
                    .as_mut()
                    .unwrap()
                    .get_q_value(&format!("{}:{:?}", state, b));
                q_a.partial_cmp(&q_b).unwrap()
            })
        }
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

            let depth = if self.difficulty == Difficulty::Hard {
                self.max_depth
            } else {
                3
            };
            let score = self.minimax(
                &mut simulated_state,
                false,
                depth,
                &mut memo,
                is_detailed_eval,
            );

            if score > best_score {
                best_score = score;
                best_moves = vec![(x, y)];
            } else if score == best_score {
                best_moves.push((x, y));
            }
        }

        best_moves.choose(&mut thread_rng()).cloned()
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
            return self.evaluate_position(game_state);
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

    fn evaluate_position(&self, game_state: &GameState) -> i32 {
        let mut score = 0;
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

        for pattern in &win_patterns {
            let mut x_count = 0;
            let mut o_count = 0;

            for &(row, col) in pattern {
                match game_state.board[row][col].as_deref() {
                    Some("X") => x_count += 1,
                    Some("O") => o_count += 1,
                    _ => {}
                }
            }

            if o_count == 3 {
                score += self.evaluation_table["opponent_three_in_row"];
            } else if o_count == 2 && x_count == 0 {
                score += self.evaluation_table["opponent_two_in_row"];
            } else if x_count == 2 && o_count == 0 {
                score += self.evaluation_table["self_two_in_row"];
            } else if x_count == 3 {
                score += self.evaluation_table["self_three_in_row"];
            }
        }

        score
    }
}

#[derive(Debug)]
pub struct QLearning {
    q_table: HashMap<String, f32>,
    alpha: f32,
    gamma: f32,
    epsilon: f32,
}

impl QLearning {
    pub fn new(alpha: f32, gamma: f32, epsilon: f32) -> Self {
        Self {
            q_table: HashMap::new(),
            alpha,
            gamma,
            epsilon,
        }
    }

    fn get_q_value(&mut self, state_action: &str) -> f32 {
        *self.q_table.entry(state_action.to_string()).or_insert(0.0)
    }

    pub fn update_q_value(&mut self, state: &str, action: &str, reward: f32, next_max_q: f32) {
        let state_action = format!("{}:{}", state, action);
        let current_q = self.get_q_value(&state_action);
        let new_q = current_q + self.alpha * (reward + self.gamma * next_max_q - current_q);
        self.q_table.insert(state_action, new_q);
    }
}
