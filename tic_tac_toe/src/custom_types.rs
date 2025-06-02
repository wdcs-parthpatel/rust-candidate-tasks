use std::collections::HashMap;
use tokio::net::TcpStream;
use colored::*;
use std::io::Result;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn to_char(self) -> char {
        match self {
            Player::X => 'X',
            Player::O => 'O',
        }
    }

    pub fn other(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Debug)]
pub struct Game {
    pub board: [[char; 3]; 3],
    pub current_player: Player,
    pub players: HashMap<Player, TcpStream>,
}

impl Game {
    pub fn new(player_x: TcpStream, player_o: TcpStream) -> Self {
        Self {
            board: [[' '; 3]; 3],
            current_player: Player::X,
            players: HashMap::from([(Player::X, player_x), (Player::O, player_o)]),
        }
    }

    pub fn reset_board(&mut self) {
        self.board = [[' '; 3]; 3];
        self.current_player = Player::X;
    }
    
    pub fn display_board(&self) -> String {
        let mut output = String::new();
        for row in &self.board {
            for &cell in row {
                let s = match cell {
                    'X' => "X".red().bold().to_string(),
                    'O' => "O".blue().bold().to_string(),
                    _ => " ".normal().to_string(),
                };
                output.push_str(&format!("[{}]", s));
            }
            output.push('\n');
        }
        output
    }

    pub fn check_winner(&self) -> Option<Player> {
        let lines = [
            [(0, 0), (0, 1), (0, 2)],
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            [(0, 0), (1, 0), (2, 0)],
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            [(0, 0), (1, 1), (2, 2)],
            [(0, 2), (1, 1), (2, 0)],
        ];

        for &line in &lines {
            let [a, b, c] = line;
            let values = [self.board[a.0][a.1], self.board[b.0][b.1], self.board[c.0][c.1]];
            if values[0] != ' ' && values.iter().all(|&x| x == values[0]) {
                return Some(if values[0] == 'X' { Player::X } else { Player::O });
            }
        }
        None
    }

    pub fn make_move(&mut self, row: usize, col: usize) -> Result<bool> {
        if self.board[row][col] == ' ' {
            self.board[row][col] = self.current_player.to_char();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn is_draw(&self) -> bool {
        self.board.iter().all(|row| row.iter().all(|&x| x != ' '))
    }
}
