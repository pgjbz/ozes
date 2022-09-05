use fast_log::Config;
use ozes::server;

#[tokio::main]
async fn main() {
    fast_log::init(Config::new().console()).unwrap();
    if let Err(e) = server::start_server(7656).await {
        log::error!("error on startup server {}", e)
    }
}
