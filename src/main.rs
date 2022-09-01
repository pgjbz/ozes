use fast_log::Config;
use ozes::{connection::OzesResult, server};

#[tokio::main]
async fn main() -> OzesResult {
    fast_log::init(Config::new().console()).unwrap();
    server::start_server().await?;
    Ok(())
}
