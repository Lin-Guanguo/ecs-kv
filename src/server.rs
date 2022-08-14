use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    db::{Db, KeyValue},
    zset::ScoreValue,
};
use bytes::Buf;
use hyper::{service::Service, Body, Method};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Server {
    db: Db,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinMaxScore {
    min_score: f64,
    max_score: f64,
}

type Request = hyper::Request<Body>;
type Response = hyper::Response<Body>;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("hyper error {0}")]
    HyperError(#[from] hyper::Error),

    #[error("serde error {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("rejection")]
    Rejection(),
}

impl Server {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn route(db: Db, req: Request) -> Result<Response, ServerError> {
        match req.method() {
            &Method::GET => {
                let mut uri = req.uri().path().split('/');
                let _ = uri.next().ok_or(ServerError::Rejection())?;
                let m = uri.next().ok_or(ServerError::Rejection())?;
                match m {
                    "init" => Ok(hyper::Response::builder().body(Body::from("ok")).unwrap()),
                    "del" => {
                        let key = uri.next().ok_or(ServerError::Rejection())?;
                        db.del(key);
                        Ok(hyper::Response::builder().body(Body::empty()).unwrap())
                    }
                    "query" => {
                        let key = uri.next().ok_or(ServerError::Rejection())?;
                        let value = db.query(key);
                        match value {
                            Some(value) => {
                                Ok(hyper::Response::builder().body(Body::from(value)).unwrap())
                            }
                            None => Ok(hyper::Response::builder()
                                .status(404)
                                .body(Body::empty())
                                .unwrap()),
                        }
                    }
                    "zrmv" => {
                        let key = uri.next().ok_or(ServerError::Rejection())?;
                        let value = uri.next().ok_or(ServerError::Rejection())?;
                        db.zremove(key, value);
                        Ok(hyper::Response::builder().body(Body::empty()).unwrap())
                    }
                    _ => Err(ServerError::Rejection()),
                }
            }
            &Method::POST => {
                let mut uri = req.uri().path().split('/');
                let _ = uri.next().ok_or(ServerError::Rejection())?;
                let m = uri.next().ok_or(ServerError::Rejection())?;
                match m {
                    "add" => {
                        let body = hyper::body::aggregate(req.into_body()).await?;
                        let kv = serde_json::from_reader::<_, KeyValue>(body.reader())?;
                        db.add(kv);
                        Ok(hyper::Response::new(Body::empty()))
                    }
                    "batch" => {
                        let body = hyper::body::aggregate(req.into_body()).await?;
                        let input = serde_json::from_reader::<_, Vec<KeyValue>>(body.reader())?;
                        db.batch(input);
                        Ok(hyper::Response::new(Body::empty()))
                    }
                    "list" => {
                        let body = hyper::body::aggregate(req.into_body()).await?;
                        let input = serde_json::from_reader::<_, Vec<String>>(body.reader())?;
                        let resp = db.list(input);
                        if resp.len() == 0 {
                            Ok(hyper::Response::builder()
                                .status(404)
                                .body(Body::empty())
                                .unwrap())
                        } else {
                            Ok(hyper::Response::builder()
                                .body(Body::from(serde_json::to_vec(&resp)?))
                                .unwrap())
                        }
                    }
                    "zadd" => {
                        let key = uri.next().ok_or(ServerError::Rejection())?.to_string();
                        let body = hyper::body::aggregate(req.into_body()).await?;
                        let input = serde_json::from_reader::<_, ScoreValue>(body.reader())?;
                        db.zadd(key, input.value, input.score);
                        Ok(hyper::Response::new(Body::empty()))
                    }
                    "zrange" => {
                        let key = uri.next().ok_or(ServerError::Rejection())?.to_string();
                        let body = hyper::body::aggregate(req.into_body()).await?;
                        let input = serde_json::from_reader::<_, MinMaxScore>(body.reader())?;
                        let resp = db.zrange(&key, input.min_score, input.max_score);
                        if resp.len() == 0 {
                            Ok(hyper::Response::builder()
                                .status(404)
                                .body(Body::empty())
                                .unwrap())
                        } else {
                            Ok(hyper::Response::builder()
                                .body(Body::from(serde_json::to_vec(&resp)?))
                                .unwrap())
                        }
                    }
                    _ => Err(ServerError::Rejection()),
                }
            }
            _ => Err(ServerError::Rejection()),
        }
    }
}

impl Service<Request> for Server {
    type Response = Response;
    type Error = ServerError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        Box::pin(Self::route(self.db.clone(), req))
    }
}
