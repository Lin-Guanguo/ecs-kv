use warp::Filter;

use crate::db::Db;
use serde_derive::{Deserialize, Serialize};
use warp::http::StatusCode;

pub struct Server {
    db: Db,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScoreValue {
    score: f64,
    value: String,
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

    pub async fn run(&self) {
        let route = self.server_route();
        warp::serve(route).run(([0, 0, 0, 0], 8080)).await;
    }

    fn server_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let init = self
            .with_ctx()
            .and(warp::path!("init"))
            .and_then(Server::get_init);
        let query = self
            .with_ctx()
            .and(warp::path!("query" / String))
            .and_then(Server::get_query);
        let add = self
            .with_ctx()
            .and(warp::path!("add"))
            .and(warp::body::json())
            .and_then(Server::post_add);
        let del = self
            .with_ctx()
            .and(warp::path!("del" / String))
            .and_then(Server::get_del);
        let list = self
            .with_ctx()
            .and(warp::path!("list"))
            .and(warp::body::json())
            .and_then(Server::post_list);
        let batch = self
            .with_ctx()
            .and(warp::path!("batch"))
            .and(warp::body::json())
            .and_then(Server::post_batch);
        let zadd = self
            .with_ctx()
            .and(warp::path!("zadd" / String))
            .and(warp::body::json())
            .and_then(Server::post_zadd);
        let zrange = self
            .with_ctx()
            .and(warp::path!("zrange" / String))
            .and(warp::body::json())
            .and_then(Server::post_zrange);
        let zrmv = self
            .with_ctx()
            .and(warp::path!("zrmv" / String / String))
            .and_then(Server::get_zrmv);

        let get = warp::get().and(init.or(query).or(del).or(zrmv));
        let post = warp::post().and(add.or(list).or(batch).or(zadd).or(zrange));

        let route = get.or(post);
        return route;
    }

    fn with_ctx(
        &self,
    ) -> impl warp::Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        let db = self.db.clone();
        warp::any().map(move || db.clone())
    }

    fn ok<E>() -> Result<warp::reply::WithStatus<&'static str>, E> {
        Ok(warp::reply::with_status("", StatusCode::OK))
    }

    fn plain_text<E>(v: String) -> Result<warp::reply::WithStatus<String>, E> {
        Ok(warp::reply::with_status(v, StatusCode::OK))
    }

    fn plain_text_not_found<E>() -> Result<warp::reply::WithStatus<String>, E> {
        Ok(warp::reply::with_status("".into(), StatusCode::NOT_FOUND))
    }

    fn json<J: serde::ser::Serialize, E>(
        v: J,
    ) -> Result<warp::reply::WithStatus<warp::reply::Json>, E> {
        Ok(warp::reply::with_status(
            warp::reply::json(&v),
            StatusCode::OK,
        ))
    }

    fn json_not_found<E>() -> Result<warp::reply::WithStatus<warp::reply::Json>, E> {
        Ok(warp::reply::with_status(
            warp::reply::json(&""),
            StatusCode::NOT_FOUND,
        ))
    }

    async fn get_init(_db: Db) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("get init");

        Ok(warp::reply::with_status("ok", StatusCode::OK))
    }

    async fn get_query(db: Db, key: String) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("get query {:?}", key);

        db.query(&key)
            .map_or(Self::plain_text_not_found(), |v| Self::plain_text(v))
    }

    async fn post_add(db: Db, body: KeyValue) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("post add {:?}", body);

        let KeyValue { key: k, value: v } = body;
        db.add(k, v);
        Self::ok()
    }

    async fn get_del(db: Db, key: String) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("get del {:?}", key);

        db.del(&key);
        Self::ok()
    }

    async fn post_list(db: Db, body: Vec<String>) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("post list {:?}", body);

        let ret = body
            .into_iter()
            .filter_map(|k| db.query(&k).map(|v| (k, v)))
            .map(|x| KeyValue {
                key: x.0,
                value: x.1,
            })
            .collect::<Vec<_>>();
        if ret.len() > 0 {
            Self::json(&ret)
        } else {
            Self::json_not_found()
        }
    }

    async fn post_batch(db: Db, body: Vec<KeyValue>) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("post batch {:?}", body);

        body.into_iter()
            .for_each(|KeyValue { key: k, value: v }| db.add(k, v));
        Self::ok()
    }

    async fn post_zadd(
        db: Db,
        key: String,
        body: ScoreValue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("post zadd {:?} {:?}", key, body);

        db.zadd(key, body.value, body.score);
        Self::ok()
    }

    async fn post_zrange(
        db: Db,
        key: String,
        body: MinMaxScore,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("post zrange {:?} {:?}", key, body);

        let resp = db.zrange(&key, body.min_score, body.max_score);
        if resp.len() > 0 {
            Self::json(&resp)
        } else {
            Self::json_not_found()
        }
    }

    async fn get_zrmv(
        db: Db,
        key: String,
        value: String,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        // println!("get zrmv {:?} {:?}", key, value);

        db.zremove(&key, &value);
        Self::ok()
    }
}
