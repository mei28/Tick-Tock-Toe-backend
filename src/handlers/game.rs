use crate::game::state::GameState;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub games: Mutex<HashMap<String, GameState>>,
}

// UUIDを5文字に短縮する関数
fn generate_short_id() -> String {
    Uuid::new_v4().to_string()[..5].to_string() // UUIDの最初の5文字を取り出す
}

#[post("/new")]
pub async fn new_game(
    data: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let is_ai_game = query.get("ai").map(|v| v == "true").unwrap_or(false);
    let first_player = query.get("firstPlayer").unwrap_or(&"X".to_string()).clone(); // Get first player
    let ai_level = query.get("aiLevel").cloned(); // Get AI level

    let game_id = generate_short_id();
    let mut game_state = GameState::new(is_ai_game, Some(first_player), ai_level); // Pass parameters

    if game_state.first_player == "AI" && is_ai_game {
        game_state.ai_move(); // AI makes the first move if it goes first
    }

    let mut games = data.games.lock().unwrap();
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
    let mut games = data.games.lock().unwrap();

    if let Some(game) = games.get_mut(&game_id) {
        let (x, y) = move_data.into_inner();
        if x >= 3 || y >= 3 {
            return HttpResponse::BadRequest().body("Invalid move position");
        }

        if game.place_piece(x, y) {
            // If it's an AI game and there's no winner, let the AI make its move
            if game.is_ai_game && game.winner.is_none() && game.current_player == "O" {
                game.ai_move(); // AI move
            }
            HttpResponse::Ok().json(game.clone())
        } else {
            HttpResponse::BadRequest().body("Invalid move")
        }
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

#[get("/board/{game_id}")]
pub async fn get_board(data: web::Data<AppState>, game_id: web::Path<String>) -> impl Responder {
    let game_id = game_id.into_inner();
    let games = data.games.lock().unwrap();

    if let Some(game) = games.get(&game_id) {
        HttpResponse::Ok().json(game.clone())
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

#[post("/reset/{game_id}")]
pub async fn reset_game(data: web::Data<AppState>, game_id: web::Path<String>) -> impl Responder {
    let game_id = game_id.into_inner();
    let mut games = data.games.lock().unwrap();

    if let Some(game) = games.get_mut(&game_id) {
        game.reset();
        HttpResponse::Ok().json(game.clone())
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

