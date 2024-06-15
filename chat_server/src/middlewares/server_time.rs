use axum::{extract::Request, response::Response};
use futures_util::Future;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};
use tracing::warn;

use super::{REQUEST_ID_HEADER, REQUEST_TIME_HEADER};

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
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
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());
            match elapsed.parse() {
                Ok(v) => {
                    response.headers_mut().insert(REQUEST_TIME_HEADER, v);
                }
                Err(e) => {
                    warn!(
                        "Parsed elapsed time for request {:?} failed: {}",
                        response.headers().get(REQUEST_ID_HEADER),
                        e
                    );
                }
            }
            Ok(response)
        })
    }
}
