use actix_web::{App, HttpServer};

use routes::routes;

mod handlers;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(routes))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
