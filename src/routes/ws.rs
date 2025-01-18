use actix_web::{web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::{handlers, AppState};

pub async fn chat(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, body)?;

    let client_id = Uuid::new_v4().to_string();
    state
        .clients
        .write()
        .await
        .insert(client_id.clone(), session.clone());

    for chat in state.history.read().await.iter() {
        let json = serde_json::to_string(chat).unwrap();
        session.text(json).await.unwrap();
    }

    actix_web::rt::spawn(handlers::chat::handle_incoming_messages(
        stream,
        client_id,
        state.tx.clone(),
        state.clients.clone(),
    ));

    actix_web::rt::spawn(handlers::chat::handle_outgoing_messages(
        state.tx.subscribe(),
        session,
        state.history.clone(),
    ));

    Ok(res)
}
