use crate::game::state::GameState;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone)]
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
            Difficulty::Hard => self.minmax_move(game_state),
        }
    }

    fn random_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        let mut rng = thread_rng();
        game_state.available_moves().choose(&mut rng).cloned()
    }

    fn minmax_move(&self, game_state: &GameState) -> Option<(usize, usize)> {
        // Placeholder for the Minimax algorithm
        None
    }
}
