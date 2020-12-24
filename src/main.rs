#![recursion_limit = "256"]

use futures_util::FutureExt;
use simple_logger::SimpleLogger;
use tonic::transport::Server;

mod game_pad_service;
mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("vgp_device", log::LevelFilter::Trace)
        .with_module_level("vgp_server", log::LevelFilter::Trace)
        .init()?;

    let addr = "0.0.0.0:50000".parse()?;

    let shutdown_signal = tokio::signal::ctrl_c().map(|_| ());

    log::info!("Starting the server at {:?}...", addr);

    Server::builder()
        .add_service(game_pad_service::create())
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    log::info!("Stopped the server!");

    Ok(())
}
