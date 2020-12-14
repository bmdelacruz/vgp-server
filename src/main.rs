use futures_util::FutureExt;
use simple_logger::SimpleLogger;
use tonic::transport::Server;

mod game_pad;
mod game_pad_service;
mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()?;

    let addr = "[::1]:50000".parse()?;

    let shutdown_signal = tokio::signal::ctrl_c().map(|_| ());

    Server::builder()
        .add_service(game_pad_service::create())
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    Ok(())
}
