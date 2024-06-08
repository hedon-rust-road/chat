mod auth;
mod chat;
mod messages;

pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use messages::*;

use axum::response::IntoResponse;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}
