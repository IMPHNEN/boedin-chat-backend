use axum::{routing, Router};
use sqlx::{migrate::Migrator, SqlitePool};
use states::AppState;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tracing::info;
use utils::MIGRATIONS_DIR;

mod models;
mod routes;
mod states;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let env_path = utils::get_executable_dir().join(".env");
    utils::prepare_env_file(env_path.clone())?;
    dotenv::from_path(env_path).ok();

    let db_path = utils::get_executable_dir().join("imphnen.db");
    let database_url = format!("sqlite:{}", db_path.display());

    if !db_path.exists() {
        std::fs::File::create(&db_path)?;
    }

    let pool = SqlitePool::connect(&database_url).await?;

    let temp_dir = tempdir()?;
    let migrations_path = temp_dir.path();

    for file in MIGRATIONS_DIR.files() {
        let file_path = migrations_path.join(file.path());

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&file_path, file.contents())?
    }

    let migrator = Migrator::new(migrations_path).await?;
    migrator.run(&pool).await?;

    temp_dir.close()?;

    let app_state = AppState::new(pool).await;
    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/auth/login", routing::get(routes::discord_login))
                .route("/auth/authorized", routing::get(routes::discord_callback)),
        )
        .route("/ws", routing::get(routes::chat_ws))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Server is running on '{}'", listener.local_addr()?);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
