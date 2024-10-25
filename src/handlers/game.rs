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
    let ai_level = query
        .get("aiLevel")
        .cloned()
        .unwrap_or("medium".to_string());
    let difficulty = match ai_level.as_str() {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    };

    let game_id = generate_short_id();
    let game_state = GameState::new(true, Some(difficulty));

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
            if game.is_ai_game && game.winner.is_none() {
                let max_depth = if game.difficulty == Some(Difficulty::Hard) {
                    30
                } else {
                    30
                };
                let ai_player = AiPlayer::new(
                    game.difficulty.clone().unwrap_or(Difficulty::Medium),
                    max_depth,
                );
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
