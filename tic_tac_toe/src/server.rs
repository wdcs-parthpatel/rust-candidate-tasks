mod custom_types;

use std::collections::HashMap;
use std::io::{Result};
use tokio::net::{TcpListener};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use colored::*;
use custom_types::*;


#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("{}", "Waiting for two players...".green());

    let (p1, _) = listener.accept().await?;
    let (p2, _) = listener.accept().await?;

    let mut p1_reader = BufReader::new(p1);
    let mut p2_reader = BufReader::new(p2);

    let mut name1 = String::new();
    let mut name2 = String::new();
    
    p1_reader.get_mut().write_all(b"Player 1, enter your name: \n").await?;
    println!("{}", "Player 1 connected!".green());
    p1_reader.read_line(&mut name1).await?;

    p2_reader.get_mut().write_all(b"Player 2, enter your name: \n").await?;
    println!("{}", "Player 2 connected!".green());   
    p2_reader.read_line(&mut name2).await?;

    let name1 = name1.trim().to_string();
    let name2 = name2.trim().to_string();

    let mut player_names = HashMap::new();
    player_names.insert(Player::X, name1);
    player_names.insert(Player::O, name2);

    let mut scores = HashMap::from(
      [
          (Player::X.to_char(), 0),
          (Player::O.to_char(), 0),
          ('D', 0),
      ]);

    let mut game = Game::new(p1_reader.into_inner(), p2_reader.into_inner());

    println!("{}", "Game started!".green());

    let mut round = 0;
    while round < 5 {
        game.reset_board();
        println!("Round: {}", round);
        loop {
            let board_str = game.display_board();
            let keys: Vec<Player> = game.players.keys().cloned().collect();
            for key in &keys {
                if let Some(stream) = game.players.get_mut(key) {
                    stream.write_all(b"\x1B[2J\x1B[H").await?;
                    stream.write_all(board_str.as_bytes()).await?;
                }
            }

            if let Some(stream) = game.players.get_mut(&game.current_player) {
                let mut player_reader = BufReader::new(stream);
                player_reader.get_mut().write_all(format!("{}'s move (row col): \n", player_names[&game.current_player]).as_bytes()).await?;

                let mut input = String::new();
                player_reader.read_line(&mut input).await?;

                let parts: Vec<&str> = input.trim().split_whitespace().collect();

                if parts.len() != 2 {
                    continue;
                }

                let row = parts[0].parse::<usize>().unwrap_or(3);
                let col = parts[1].parse::<usize>().unwrap_or(3);

                if row > 2 || col > 2 {
                    continue;
                }

                if game.make_move(row, col)? {
                    if let Some(winner) = game.check_winner() {
                        let final_board = game.display_board();
                        for key in &keys {
                            if let Some(stream) = game.players.get_mut(key) {
                                stream.write_all(b"\x1B[2J\x1B[H").await?;
                                stream.write_all(final_board.as_bytes()).await?;
                                let msg = format!("Player {} ({}) wins!\n", winner.to_char(), player_names[&winner]).green();
                                stream.write_all(msg.as_bytes()).await?;
                            }
                        }
                        *scores.entry(winner.to_char()).or_default() += 1;
                        break;
                    } else if game.is_draw() {
                        let final_board = game.display_board();
                        for key in &keys {
                            if let Some(stream) = game.players.get_mut(key) {
                                stream.write_all(b"\x1B[2J\x1B[H").await?;
                                stream.write_all(final_board.as_bytes()).await?;
                                let msg = "It's a draw!\n".to_string();
                                stream.write_all(msg.as_bytes()).await?;
                            }
                        }
                        *scores.entry('D').or_default() += 1;
                        break;
                    }
                    game.current_player = game.current_player.other();
                }
            }
        }
        round += 1;
        let msg = format!("Score after round {}: \n{}: {} \n{}: {}, \nDraw: {} \n\n", 
                          round, 
                          player_names[&Player::X], scores[&Player::X.to_char()],
                          player_names[&Player::O], scores[&Player::O.to_char()], 
                          scores[&'D'],
        );
        for stream in game.players.values_mut() {
            stream.write_all(msg.as_bytes()).await?;
        }
    }
    Ok(())
}
