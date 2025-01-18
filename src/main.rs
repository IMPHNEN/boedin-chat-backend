use actix_web::{middleware, web, App, HttpServer};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};

mod handlers;
mod routes;

const CHANNEL_CAPACITY: usize = 1000;

#[derive(Clone)]
pub struct AppState {
    tx: broadcast::Sender<String>,
    history: Arc<RwLock<Vec<handlers::chat::Chat>>>,
    clients: Arc<RwLock<HashMap<String, actix_ws::Session>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, _) = broadcast::channel::<String>(CHANNEL_CAPACITY);

    let state = web::Data::new(AppState {
        tx,
        history: Arc::new(RwLock::new(Vec::new())),
        clients: Arc::new(RwLock::new(HashMap::new())),
    });

    println!("Server started at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            .app_data(state.clone())
            .configure(routes::init)
    })
    .workers(num_cpus::get())
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
