use log::info;
use std::error::Error;

use dendrite_example::application::application;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Dendrite Example API service started");

    application().await
}
