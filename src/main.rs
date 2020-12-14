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

    Server::builder()
        .add_service(game_pad_service::create())
        .serve(addr)
        .await?;

    Ok(())
}
