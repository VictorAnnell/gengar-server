use gengar::*;
use mysql::*;
use mysql::prelude::*;

#[tokio::main]
async fn main() {
    start_server().await;
}
