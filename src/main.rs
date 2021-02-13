mod api;
mod db;
mod server;

#[tokio::main]
async fn main() {
    api::run().await;
}
