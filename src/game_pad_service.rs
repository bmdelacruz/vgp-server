use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status, Streaming};

use crate::game_pad::GamePad as GamePadDevice;
use crate::proto::service_prelude::*;

#[derive(Debug, Default)]
pub struct GamePadImpl;

#[tonic::async_trait]
impl GamePad for GamePadImpl {
    async fn check(
        &self,
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        Ok(Response::new(CheckResponse {}))
    }

    async fn instantiate(
        &self,
        request: Request<Streaming<InputData>>,
    ) -> Result<Response<OutputData>, Status> {
        let remote_addr = request.remote_addr();

        log::info!(
            "Client ({:?}) requested to instantiate game pad.",
            remote_addr
        );

        let mut stream = request.into_inner();

        let game_pad = GamePadDevice::create(512f32).map_err(|_e| {
            Status::internal("An error occurred while trying to create game pad device.")
        })?;
        let game_pad = Arc::new(Mutex::new(game_pad));

        log::info!("Instantiated game pad for client ({:?}).", remote_addr);

        while let Some(input) = stream.next().await {
            let data = input?.control.ok_or(Status::invalid_argument(
                "The `control` argument is missing!",
            ))?;

            log::info!(
                "Received game pad input data ({:?}) from client ({:?}).",
                data,
                remote_addr
            );

            game_pad.lock().await.control(data).map_err(|_e| {
                Status::internal("An error occurred while trying to control the game pad device.")
            })?;
        }

        log::info!(
            "Destroying game pad that was made for client ({:?}).",
            remote_addr
        );

        Ok(Response::new(OutputData {}))
    }
}

pub fn create() -> GamePadServer<GamePadImpl> {
    GamePadServer::new(GamePadImpl::default())
}
