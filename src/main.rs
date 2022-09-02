use fast_log::Config;
use ozes::server::{self, OzesResult};

#[tokio::main]
async fn main() -> OzesResult {
    fast_log::init(Config::new().console()).unwrap();
    server::start_server(7656).await?;
    Ok(())
}
