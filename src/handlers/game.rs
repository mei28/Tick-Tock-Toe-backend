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
pub async fn new_game(data: web::Data<AppState>) -> impl Responder {
    let game_id = generate_short_id(); // 短いIDを生成
    let game_state = GameState::new();

    let mut games = data.games.lock().unwrap();
    games.insert(game_id.clone(), game_state);

    HttpResponse::Ok().json(game_id) // 短いゲームIDをクライアントに返す
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

