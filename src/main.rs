use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    if let Err(err) = vincent::run().await {
        log::error!("{}", err)
    }
}
