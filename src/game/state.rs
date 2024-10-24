use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub board: [[Option<String>; 3]; 3],
    pub current_player: String, // 常に "X" または "O"
    pub moves_x: Vec<(usize, usize)>,
    pub moves_o: Vec<(usize, usize)>,
    pub winner: Option<String>,
    pub winning_line: Option<[(usize, usize); 3]>,
    pub is_ai_game: bool,         // AIゲームかどうか
    pub ai_level: Option<String>, // AIのレベル
}

impl GameState {
    pub fn new(is_ai_game: bool, ai_level: Option<String>) -> Self {
        Self {
            board: [[None, None, None], [None, None, None], [None, None, None]],
            current_player: "X".to_string(), // プレイヤーが常に先攻
            moves_x: Vec::new(),
            moves_o: Vec::new(),
            winner: None,
            winning_line: None,
            is_ai_game,
            ai_level,
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
                // プレイヤーの切り替え
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

    pub fn reset(&mut self) {
        self.board = [[None, None, None], [None, None, None], [None, None, None]];
        self.moves_x.clear();
        self.moves_o.clear();
        self.winner = None;
        self.winning_line = None;
        self.current_player = "X".to_string(); // プレイヤーが常に先攻
    }

    pub fn ai_move(&mut self) -> Option<(usize, usize)> {
        if self.current_player == "O" && self.is_ai_game {
            // AIは常に後攻
            if let Some(ai_level) = &self.ai_level {
                match ai_level.as_str() {
                    "easy" => self.random_ai_move(),
                    "medium" => self.medium_ai_move(),
                    "hard" => self.hard_ai_move(),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn random_ai_move(&mut self) -> Option<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let available_moves = (0..3)
            .flat_map(|x| (0..3).map(move |y| (x, y)))
            .filter(|&(x, y)| self.board[x][y].is_none())
            .collect::<Vec<(usize, usize)>>();

        if let Some(&(x, y)) = available_moves.iter().choose(&mut rng) {
            self.place_piece(x, y);
            Some((x, y))
        } else {
            None
        }
    }

    fn medium_ai_move(&mut self) -> Option<(usize, usize)> {
        self.random_ai_move()
    }

    fn hard_ai_move(&mut self) -> Option<(usize, usize)> {
        self.random_ai_move()
    }
}
