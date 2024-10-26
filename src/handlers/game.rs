use crate::ai::{AiPlayer, Difficulty};
use crate::game::state::GameState;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::info;
use rand::seq::SliceRandom; // Import for .choose()
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub games: Mutex<HashMap<String, GameState>>,
}

fn generate_short_id() -> String {
    Uuid::new_v4().to_string()[..5].to_string()
}

#[post("/new")]
pub async fn new_game(
    data: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let ai_level = query.get("aiLevel").cloned().unwrap_or("none".to_string());
    let difficulty = match ai_level.as_str() {
        "easy" => Some(Difficulty::Easy),
        "medium" => Some(Difficulty::Medium),
        "hard" => Some(Difficulty::Hard),
        _ => None,
    };

    let game_id = generate_short_id();
    let game_state = GameState::new(difficulty.is_some(), difficulty);

    let mut games = data.games.lock().await;
    games.insert(game_id.clone(), game_state);

    HttpResponse::Ok().json(game_id)
}

#[post("/move/{game_id}")]
pub async fn make_move(
    data: web::Data<AppState>,
    game_id: web::Path<String>,
    move_data: web::Json<(usize, usize)>,
) -> impl Responder {
    let game_id = game_id.into_inner();
    let mut games = data.games.lock().await;

    if let Some(game) = games.get_mut(&game_id) {
        let (x, y) = move_data.into_inner();
        if x >= 3 || y >= 3 {
            return HttpResponse::BadRequest().body("Invalid move position");
        }

        if game.place_piece(x, y) {
            let mut is_thinking = false;

            // Start AI decision process if it's an AI game and no winner yet
            if game.is_ai_game && game.winner.is_none() {
                is_thinking = true;
                let difficulty = game.difficulty.clone().unwrap_or(Difficulty::Medium);
                let mut ai_player = AiPlayer::new(difficulty.clone()); // Clone here

                // Log the current game state and AI difficulty for debugging
                info!(
                    "AI thinking - Difficulty: {:?}, Current State: {:?}",
                    difficulty,
                    game.to_three_state()
                );

                // Calculate AI's move with error handling
                let ai_move = ai_player.make_move(game).or_else(|| {
                    // Default to a random move if no decision is made
                    game.available_moves()
                        .choose(&mut rand::thread_rng())
                        .cloned()
                });

                if let Some((ai_x, ai_y)) = ai_move {
                    game.place_piece(ai_x, ai_y);
                    info!("AI placed move at: ({}, {})", ai_x, ai_y);
                } else {
                    info!("AI could not determine a move.");
                }

                is_thinking = false;
            }

            let response = HttpResponse::Ok().json(json!({
                "board": game.board,
                "current_player": game.current_player,
                "winner": game.winner,
                "winning_line": game.winning_line,
                "isThinking": is_thinking,
            }));

            return response;
        } else {
            return HttpResponse::BadRequest().body("Invalid move");
        }
    }

    HttpResponse::NotFound().body("Game not found")
}

#[get("/board/{game_id}")]
pub async fn get_board(data: web::Data<AppState>, game_id: web::Path<String>) -> impl Responder {
    let game_id = game_id.into_inner();
    let games = data.games.lock().await;

    if let Some(game) = games.get(&game_id) {
        HttpResponse::Ok().json(game.clone())
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

#[post("/reset/{game_id}")]
pub async fn reset_game(data: web::Data<AppState>, game_id: web::Path<String>) -> impl Responder {
    let game_id = game_id.into_inner();
    let mut games = data.games.lock().await;

    if let Some(game) = games.get_mut(&game_id) {
        let is_ai_game = game.is_ai_game;
        let difficulty = game.difficulty.clone();
        game.reset(); // Resets the game state
        game.is_ai_game = is_ai_game;
        game.difficulty = difficulty;
        HttpResponse::Ok().json(game.clone())
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}
