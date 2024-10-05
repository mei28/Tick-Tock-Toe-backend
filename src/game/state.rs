use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub board: [[Option<String>; 3]; 3],
    pub current_player: String,
    pub moves_x: Vec<(usize, usize)>,
    pub moves_o: Vec<(usize, usize)>,
    pub winner: Option<String>,
    pub winning_line: Option<[(usize, usize); 3]>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: [[None, None, None], [None, None, None], [None, None, None]],
            current_player: "X".to_string(),
            moves_x: Vec::new(),
            moves_o: Vec::new(),
            winner: None,
            winning_line: None,
        }
    }

    pub fn place_piece(&mut self, x: usize, y: usize) -> bool {
        if self.winner.is_some() {
            return false; // すでに勝者がいる場合は駒を置けない
        }

        if self.board[x][y].is_none() {
            self.board[x][y] = Some(self.current_player.clone());

            let moves = if self.current_player == "X" {
                &mut self.moves_x
            } else {
                &mut self.moves_o
            };

            moves.push((x, y));

            // 4個以上の駒がある場合に最も古い駒を削除
            if moves.len() > 3 {
                let (old_x, old_y) = moves.remove(0);
                self.board[old_x][old_y] = None;
            }

            self.check_winner(); // 勝利条件を確認

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

    fn check_winner(&mut self) {
        // 勝利パターンをチェックするための配列
        let win_patterns = [
            [(0, 0), (0, 1), (0, 2)], // 横
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            [(0, 0), (1, 0), (2, 0)], // 縦
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            [(0, 0), (1, 1), (2, 2)], // 斜め
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
                println!("Winner found: {:?}", self.winner);
                return;
            }
        }
    }

    pub fn reset(&mut self) {
        self.board = [[None, None, None], [None, None, None], [None, None, None]];
        self.current_player = "X".to_string();
        self.moves_x.clear();
        self.moves_o.clear();
        self.winner = None;
        self.winning_line = None;
    }
}

