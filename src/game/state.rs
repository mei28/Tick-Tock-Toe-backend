use crate::ai::Difficulty;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub board: [[Option<String>; 3]; 3], // 3x3のボードを定義
    pub current_player: String,
    pub moves_x: Vec<(usize, usize)>,
    pub moves_o: Vec<(usize, usize)>,
    pub winner: Option<String>,
    pub winning_line: Option<[(usize, usize); 3]>,
    pub is_ai_game: bool,
    pub difficulty: Option<Difficulty>,
}

impl GameState {
    pub fn new(is_ai_game: bool, difficulty: Option<Difficulty>) -> Self {
        Self {
            board: [[None, None, None], [None, None, None], [None, None, None]], // 3x3の初期化
            current_player: "X".to_string(),
            moves_x: Vec::new(),
            moves_o: Vec::new(),
            winner: None,
            winning_line: None,
            is_ai_game,
            difficulty,
        }
    }

    pub fn place_piece(&mut self, x: usize, y: usize) -> bool {
        if self.winner.is_some() {
            return false;
        }

        if self.board[x][y].is_none() {
            self.board[x][y] = Some(self.current_player.clone());

            let moves = if self.current_player == "X" {
                &mut self.moves_x
            } else {
                &mut self.moves_o
            };

            moves.push((x, y));

            if moves.len() > 3 {
                let (old_x, old_y) = moves.remove(0);
                self.board[old_x][old_y] = None;
            }

            self.check_winner();

            if self.winner.is_none() {
                self.current_player = if self.current_player == "X" {
                    "O".to_string()
                } else {
                    "X".to_string()
                };
            }

            true
        } else {
            false
        }
    }

    pub fn undo_move(&mut self, x: usize, y: usize) {
        self.board[x][y] = None;
    }

    fn check_winner(&mut self) {
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
            let [a, b, c] = pattern;
            if self.board[a.0][a.1].is_some()
                && self.board[a.0][a.1] == self.board[b.0][b.1]
                && self.board[a.0][a.1] == self.board[c.0][c.1]
            {
                self.winner = self.board[a.0][a.1].clone();
                self.winning_line = Some(*pattern);
                return;
            }
        }
    }

    pub fn available_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = vec![];
        for x in 0..3 {
            for y in 0..3 {
                if self.board[x][y].is_none() {
                    moves.push((x, y));
                }
            }
        }
        moves
    }

    pub fn reset(&mut self) {
        self.board = [[None, None, None], [None, None, None], [None, None, None]]; // 3x3のボードのリセット
        self.moves_x.clear();
        self.moves_o.clear();
        self.winner = None;
        self.winning_line = None;
        self.current_player = "X".to_string();
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.board)
    }
}
