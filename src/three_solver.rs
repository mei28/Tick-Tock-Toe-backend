use crate::game::state::GameState;
use rand::{seq::SliceRandom, thread_rng, Rng}; // Import Rng here
use std::collections::HashMap;

pub struct ThreeSolver {
    q_table: HashMap<String, HashMap<(usize, usize), f32>>,
    alpha: f32,
    gamma: f32,
    epsilon: f32,
}

impl ThreeSolver {
    pub fn new() -> Self {
        Self {
            q_table: HashMap::new(),
            alpha: 0.1,
            gamma: 0.9,
            epsilon: 0.2,
        }
    }

    pub fn update(&mut self, state: &str, action: (usize, usize), reward: f32, next_state: &str) {
        let current_q = self.get_q_value(state, action);
        let max_q_next = self
            .q_table
            .get(next_state)
            .map(|actions| actions.values().cloned().fold(f32::MIN, f32::max))
            .unwrap_or(0.0);

        let new_q = current_q + self.alpha * (reward + self.gamma * max_q_next - current_q);
        self.q_table
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action, new_q);
    }

    pub fn select_best_action(
        &mut self,
        state: &str,
        game_state: &GameState,
    ) -> Option<(usize, usize)> {
        if thread_rng().gen::<f32>() < self.epsilon {
            game_state
                .available_moves()
                .choose(&mut thread_rng())
                .cloned()
        } else {
            self.q_table.get(state).and_then(|actions| {
                actions
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(&action, _)| action)
            })
        }
    }

    fn get_q_value(&self, state: &str, action: (usize, usize)) -> f32 {
        *self
            .q_table
            .get(state)
            .and_then(|actions| actions.get(&action))
            .unwrap_or(&0.0)
    }

    pub fn decay_epsilon(&mut self) {
        if self.epsilon > 0.01 {
            self.epsilon *= 0.995;
        }
    }
}
