use crate::game::state::GameState;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::Mutex;

pub struct AppState {
    pub game: Mutex<GameState>,
}

#[post("/move")]
pub async fn make_move(
    data: web::Data<AppState>,
    move_data: web::Json<(usize, usize)>,
) -> impl Responder {
    println!("Received move request: {:?}", move_data);

    let mut game = match data.game.lock() {
        Ok(game) => game,
        Err(e) => {
            println!("Failed to lock game state: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to lock game state");
        }
    };

    let (x, y) = move_data.into_inner();
    if x >= 3 || y >= 3 {
        println!("Invalid position: ({}, {})", x, y);
        return HttpResponse::BadRequest().body("Invalid move position");
    }

    if game.place_piece(x, y) {
        println!("Move successful: {:?}", game.board);
        HttpResponse::Ok().json(game.clone())
    } else {
        println!("Invalid move attempt: {:?}", (x, y));
        HttpResponse::BadRequest().body("Invalid move")
    }
}

#[get("/board")]
pub async fn get_board(data: web::Data<AppState>) -> impl Responder {
    let game = match data.game.lock() {
        Ok(game) => game,
        Err(e) => {
            println!("Failed to lock game state for board request: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to lock game state");
        }
    };
    println!("Board state requested");
    HttpResponse::Ok().json(game.clone())
}

