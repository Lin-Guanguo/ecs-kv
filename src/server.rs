use std::{
    future::{Future, self},
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    db::{Db, KeyValue},
    zset::ScoreValue,
};
use hyper::{service::Service, Body, Method, Request, Response};
use serde_derive::{Deserialize, Serialize};

pub struct Server {
    db: Db,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinMaxScore {
    min_score: f64,
    max_score: f64,
}

impl Server {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl Service<Request<Body>> for Server {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let r = match req.method() {
            &Method::GET => todo!(),
            &Method::POST => todo!(),
            _ => Err("not support method"),
        }
        Box::pin(futures::future::ok(r))
    }
}
