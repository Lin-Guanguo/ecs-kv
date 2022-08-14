use std::net::SocketAddr;

use ecs_project::*;
use hyper::server::conn::Http;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // println!("service start hyper");

    let db = Db::new();
    let addr: SocketAddr = ([0, 0, 0, 0], 8080).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let db = db.clone();
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, Server::new(db)).await {
                // println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
