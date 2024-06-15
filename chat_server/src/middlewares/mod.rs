mod request_id;
mod server_time;

use axum::Router;
use request_id::RequestIDLayer;
use server_time::ServerTimeLayer;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

const REQUEST_ID_HEADER: &str = "x-request-id";
const REQUEST_TIME_HEADER: &str = "x-server-time";

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
    .layer(RequestIDLayer {}) // .layer(from_fn(set_request_id))
    .layer(ServerTimeLayer {})
}
