use crate::game::state::GameState;
use crate::handlers::game::{get_board, make_move, reset_game, AppState};
use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

mod game;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_state = web::Data::new(AppState {
        game: Mutex::new(GameState::new()),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive()) // CORSの設定
            .app_data(game_state.clone()) // AppStateをアプリケーションデータとして登録
            .service(make_move)
            .service(get_board)
            .service(reset_game)
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
