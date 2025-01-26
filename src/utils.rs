use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use include_dir::{include_dir, Dir};

pub static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

lazy_static::lazy_static! {
    pub static ref HISTORY_LIMIT: usize = env::var("HISTORY_LIMIT").expect("HISTORY_LIMIT must be set").parse().expect("HISTORY_LIMIT must be a valid number");
    pub static ref CHANNEL_CAPACITY: usize = env::var("CHANNEL_CAPACITY").expect("CHANNEL_CAPACITY must be set").parse().expect("CHANNEL_CAPACITY must be a valid number");

    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    pub static ref JWT_SECRET_REFRESH: String = env::var("JWT_SECRET_REFRESH").expect("JWT_SECRET_REFRESH must be set");

    pub static ref BACKEND_URL: String = env::var("BACKEND_URL").expect("BACKEND_URL must be set");
    pub static ref FRONTEND_URL: String = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    pub static ref DISCORD_GUILD_ID: usize = env::var("DISCORD_GUILD_ID").expect("DISCORD_GUILD_ID must be set").parse().expect("DISCORD_GUILD_ID must be a valid number");
    pub static ref DISCORD_CLIENT_ID: usize = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set").parse().expect("DISCORD_CLIENT_ID must be a valid number");
    pub static ref DISCORD_CLIENT_SECRET: String = env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set");
    pub static ref DISCORD_REDIRECT_URL: String = env::var("DISCORD_REDIRECT_URL").expect("DISCORD_REDIRECT_URL must be set");
}

pub fn get_executable_dir() -> PathBuf {
    env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf()
}

pub fn prepare_env_file(path: PathBuf) -> std::io::Result<()> {
    let vars = vec![
        ("HISTORY_LIMIT", "30"),
        ("CHANNEL_CAPACITY", "50"),
        ("-", "-"),
        ("JWT_SECRET", "jwt_secret_here"),
        ("JWT_SECRET_REFRESH", "jwt_refresh_secret_here"),
        ("-", "-"),
        ("BACKEND_URL", "http://localhost:8080"),
        ("FRONTEND_URL", "http://localhost:4329"),
        ("DATABASE_URL", "sqlite:imphnen.db"),
        ("-", "-"),
        ("DISCORD_GUILD_ID", "1234567891234567890"),
        ("DISCORD_CLIENT_ID", "1234567891234567890"),
        ("DISCORD_CLIENT_SECRET", "discord_secret_here"),
        (
            "DISCORD_REDIRECT_URL",
            "http://localhost:8080/api/auth/discord/authorized",
        ),
    ];

    let mut contents = HashMap::new();
    if path.exists() {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some((key, value)) = line.split_once('=') {
                    contents.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    let mut new_content = String::new();
    for (key, default_value) in &vars {
        if *key == "-" {
            new_content.push('\n');
        } else if let Some(value) = contents.get(*key) {
            new_content.push_str(&format!("{key}={value}\n"));
        } else {
            new_content.push_str(&format!("{key}={default_value}\n"));
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(new_content.trim_end().as_bytes())?;

    Ok(())
}
