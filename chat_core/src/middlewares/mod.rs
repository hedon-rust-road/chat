mod auth;
mod request_id;
mod server_time;

use core::fmt;

use axum::Router;
use request_id::RequestIDLayer;
use server_time::ServerTimeLayer;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::User;

pub use auth::verify_token;

const REQUEST_ID_HEADER: &str = "x-request-id";
const REQUEST_TIME_HEADER: &str = "x-server-time";

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}

pub trait SetRequestID {
    fn set_request_id(r: Router) -> Router;
}

pub trait SetServerTime {
    fn set_server_time(r: Router) -> Router;
}

pub trait SetLayer {
    fn set_layer(r: Router) -> Router;
}

pub trait SetCompression {
    fn set_compression(r: Router) -> Router;
}

pub fn set_layer(r: Router) -> Router {
    r.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    )
    .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
    .layer(RequestIDLayer) // .layer(from_fn(set_request_id))
    .layer(ServerTimeLayer)
}
