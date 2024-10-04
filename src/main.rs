use crate::game::state::GameState;
use crate::handlers::game::{get_board, make_move, reset_game, AppState}; // reset_gameをインポート
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
            .service(reset_game) // reset_gameのエンドポイントを登録
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

