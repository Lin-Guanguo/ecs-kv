use ecs_project::*;

#[tokio::main]
async fn main() {
    println!("hello world");

    let db = Db::new();
    let server = Server::new(db);
    server.run().await;
}
