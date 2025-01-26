use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde_json::Value;
use tracing::error;

use crate::{
    models::{AuthRequest, Claims},
    states::AppState,
    utils::{
        DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET, DISCORD_GUILD_ID, DISCORD_REDIRECT_URL, FRONTEND_URL, JWT_SECRET
    },
};

pub async fn discord_login(State(state): State<Arc<AppState>>) -> Redirect {
    let client = BasicClient::new(ClientId::new(DISCORD_CLIENT_ID.to_string()))
        .set_client_secret(ClientSecret::new(DISCORD_CLIENT_SECRET.to_string()))
        .set_auth_uri(AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(DISCORD_REDIRECT_URL.to_string()).unwrap());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    {
        let mut code_verifier = state.code_verifier.write().await;
        *code_verifier = Some(pkce_verifier.secret().to_string());
    }

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes([
            Scope::new("identify".to_string()),
            Scope::new("guilds".to_string()),
            Scope::new("guilds.members.read".to_string()),
        ])
        .set_pkce_challenge(pkce_challenge)
        .url();

    Redirect::to(auth_url.as_ref())
}

pub async fn discord_callback(
    Query(AuthRequest { code }): Query<AuthRequest>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, String> {
    let client = BasicClient::new(ClientId::new(DISCORD_CLIENT_ID.to_string()))
        .set_client_secret(ClientSecret::new(DISCORD_CLIENT_SECRET.to_string()))
        .set_auth_uri(AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(DISCORD_REDIRECT_URL.to_string()).unwrap());

    let http_client = reqwest::Client::new();

    let code_verifier = {
        let code_verifier = state.code_verifier.read().await;
        code_verifier
            .clone()
            .ok_or_else(|| "Missing `code_verifier`".to_string())?
    };

    let pkce_verifier = PkceCodeVerifier::new(code_verifier);

    let result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    let token = result.access_token().secret();

    let user = http_client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user profile: {}", e))?
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse user profile: {}", e))?;

    let guilds = http_client
        .get(&format!(
            "https://discord.com/api/v10/users/@me/guilds/{}/member",
            *DISCORD_GUILD_ID
        ))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch guilds member data: {}", e))?
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse guilds member data: {}", e))?;

    let admin_id = "1206224912360939520";
    let moderator_id = "1206224912360939520";

    let roles = guilds["roles"]
        .as_array()
        .ok_or_else(|| "Invalid guild roles format".to_string())?;

    let role = if roles.contains(&Value::String(admin_id.to_string())) {
        "Admin"
    } else if roles.contains(&Value::String(moderator_id.to_string())) {
        "Moderator"
    } else {
        "Member"
    };

    let user_id = user["id"].as_str().unwrap_or_default().to_string();
    let username = user["username"].as_str().unwrap_or_default().to_string();

    if let Err(e) = state.save_user(&user_id, &username, role).await {
        error!("Failed to save user data: {}", e);
        return Err("Failed to save user data".to_string());
    }

    let expiration_time = Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .ok_or_else(|| "Failed to calculate expiration time")?
        .timestamp() as usize;

    let claims = Claims {
        sub: user["id"].as_str().unwrap_or_default().to_string(),
        username: user["username"].as_str().unwrap_or_default().to_string(),
        role: role.to_string(),
        exp: expiration_time,
    };

    let jwt_secret = EncodingKey::from_secret(JWT_SECRET.as_bytes());
    let jwt = encode(&Header::new(Algorithm::HS256), &claims, &jwt_secret)
        .map_err(|e| format!("Failed to generate JWT: {}", e))?;

    Ok(Redirect::to(&format!("{}/?token={}", *FRONTEND_URL, jwt)))
}
