use std::{future::Future, pin::Pin, task::Poll};

use axum::{extract::Request, response::Response};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use crate::middlewares::REQUEST_ID_HEADER;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(req);
        Box::pin(async move {
            let mut res: Response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());
            match elapsed.parse() {
                Ok(v) => {
                    res.headers_mut().insert(SERVER_TIME_HEADER, v);
                }
                Err(e) => {
                    warn!(
                        "parse elapsed time failed: {} for request: {:?}",
                        e,
                        res.headers().get(REQUEST_ID_HEADER)
                    );
                }
            }
            Ok(res)
        })
    }
}
