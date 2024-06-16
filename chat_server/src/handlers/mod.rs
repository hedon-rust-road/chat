mod auth;
mod chat;
mod messages;
mod workspace;

pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use messages::*;
pub(crate) use workspace::*;

use axum::response::IntoResponse;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}
