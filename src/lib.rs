mod db;
mod server;
mod zset;

pub use db::Db;
pub use server::Server;
pub use zset::Zset;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
