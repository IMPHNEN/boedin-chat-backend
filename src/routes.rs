use actix_web::web;

use crate::handlers;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/hello").route(web::get().to(handlers::hello)));
}
