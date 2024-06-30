use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

#[derive(Clone)]
pub struct RequestIDLayer;

impl<S> Layer<S> for RequestIDLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct MyMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for MyMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // if has request id, use it, otherwise generate a new one.
        let headers = request.headers_mut();
        let entry = headers
            .entry(REQUEST_ID_HEADER)
            .or_insert_with(|| {
                let value = HeaderValue::from_str(&Uuid::now_v7().to_string());
                match value {
                    Ok(v) => v,
                    Err(_) => HeaderValue::from_static("CREATE-REQUEST-ID-FAILED"),
                }
            })
            .clone();

        // set request id to response header.
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            response.headers_mut().insert(REQUEST_ID_HEADER, entry);
            Ok(response)
        })
    }
}

// method 2 -> from_fn
#[allow(unused)]
pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let headers = req.headers_mut();
    let entry = headers
        .entry(REQUEST_ID_HEADER)
        .or_insert_with(|| {
            let value = HeaderValue::from_str(&Uuid::now_v7().to_string());
            match value {
                Ok(v) => v,
                Err(_) => HeaderValue::from_static("CREATE-REQUEST-ID-FAILED"),
            }
        })
        .clone();
    let mut res = next.run(req).await;
    res.headers_mut().insert(REQUEST_ID_HEADER, entry);
    res
}
