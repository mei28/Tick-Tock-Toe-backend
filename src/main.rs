use crate::handlers::game::{get_board, make_move, new_game, reset_game, AppState};
use crate::handlers::health::health_check;
use actix_web::{web, App};
use std::collections::HashMap;
use tokio::sync::Mutex; // tokio::sync::Mutex を使用する

mod ai;
mod game;
mod handlers;
mod three_solver;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_state = web::Data::new(AppState {
        games: Mutex::new(HashMap::new()), // tokio::sync::Mutex を使用
    });

    actix_web::HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .app_data(game_state.clone())
            .service(new_game)
            .service(make_move)
            .service(get_board)
            .service(reset_game)
            .service(health_check)
    })
    .bind((
        "0.0.0.0",
        std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap(),
    ))?
    .run()
    .await
}
