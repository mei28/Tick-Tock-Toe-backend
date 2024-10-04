use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub board: [[Option<String>; 3]; 3],
    pub current_player: String,
    pub moves: Vec<(usize, usize)>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: [[None, None, None], [None, None, None], [None, None, None]],
            current_player: "X".to_string(),
            moves: Vec::new(),
        }
    }

    pub fn place_piece(&mut self, x: usize, y: usize) -> bool {
        if self.board[x][y].is_none() {
            self.board[x][y] = Some(self.current_player.clone());
            self.moves.push((x, y));

            if self.moves.len() > 3 {
                let (old_x, old_y) = self.moves.remove(0);
                self.board[old_x][old_y] = None;
            }

            self.current_player = if self.current_player == "X" {
                "O".to_string()
            } else {
                "X".to_string()
            };
            true
        } else {
            false
        }
    }
}

