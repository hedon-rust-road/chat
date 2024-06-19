use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::warn;

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                match state.dk.verify(token) {
                    Ok(user) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(user);
                        req
                    }
                    Err(e) => {
                        let msg = format!("verify token failed: {:?}", e);
                        warn!(msg);
                        return (StatusCode::FORBIDDEN, msg).into_response();
                    }
                }
            }
            Err(e) => {
                let msg = format!("parse authorization header failed: {:?}", e);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, middleware::from_fn_with_state, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::User;

    use super::*;

    #[tokio::test]
    async fn verify_token_middleware_should_work() -> anyhow::Result<()> {
        let (_, state) = AppState::new_for_test().await?;

        let user = User::new(1, "hedon", "hedon@example.com");
        let token = state.ek.sign(user)?;

        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        // valid token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let rsp = app.clone().oneshot(req).await?;
        assert_eq!(rsp.status(), StatusCode::OK);
        let body = rsp.collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");

        // no token
        let req = Request::builder().uri("/").body(Body::empty())?;
        let rsp = app.clone().oneshot(req).await?;
        assert_eq!(rsp.status(), StatusCode::UNAUTHORIZED);

        // invalid token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer invalid")
            .body(Body::empty())?;
        let rsp = app.clone().oneshot(req).await?;
        assert_eq!(rsp.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }
}
