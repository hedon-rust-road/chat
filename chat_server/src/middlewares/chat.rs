use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::Response,
};

use crate::{AppState, User};

#[allow(unused)]
pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();

    let user = parts.extensions.get::<User>().unwrap();
    todo!()
}
