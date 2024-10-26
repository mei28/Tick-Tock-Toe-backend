use crate::ai::{AiPlayer, Difficulty};
use crate::game::state::GameState;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::Mutex;
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
            // AIの動作はAIモードが有効な場合のみ行う
            if game.is_ai_game && game.winner.is_none() {
                let difficulty = game.difficulty.clone().unwrap_or(Difficulty::Medium);
                let mut ai_player = AiPlayer::new(difficulty);

                if let Some((ai_x, ai_y)) = ai_player.make_move(game) {
                    game.place_piece(ai_x, ai_y);
                }
            }
            HttpResponse::Ok().json(game.clone())
        } else {
            HttpResponse::BadRequest().body("Invalid move")
        }
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

#[post("/reset/{game_id}")]
pub async fn reset_game(data: web::Data<AppState>, game_id: web::Path<String>) -> impl Responder {
    let game_id = game_id.into_inner();
    let mut games = data.games.lock().unwrap();

    if let Some(game) = games.get_mut(&game_id) {
        // ゲームのリセット時にis_ai_gameとdifficultyを保持
        let is_ai_game = game.is_ai_game;
        let difficulty = game.difficulty.clone();
        game.reset();
        game.is_ai_game = is_ai_game;
        game.difficulty = difficulty;
        HttpResponse::Ok().json(game.clone())
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
