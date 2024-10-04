use crate::handlers;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::game::make_move)
        .service(handlers::game::get_board);
}
