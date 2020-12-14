use futures_util::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::game_pad::GamePad;
use crate::proto::service_prelude::*;

#[derive(Debug, Default)]
pub struct GamePadServiceImpl;

#[tonic::async_trait]
impl GamePadService for GamePadServiceImpl {
    async fn instantiate_game_pad(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Output>, Status> {
        let remote_addr = request.remote_addr();

        log::info!(
            "Client ({:?}) requested to instantiate game pad.",
            remote_addr
        );

        let mut stream = request.into_inner();

        let mut game_pad = GamePad::create().map_err(|_e| {
            Status::internal("An error occurred while trying to create game pad device.")
        })?;

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

            game_pad.control(data).map_err(|_e| {
                Status::internal("An error occurred while trying to control the game pad device.")
            })?;
        }

        log::info!(
            "Destroying game pad that was made for client ({:?}).",
            remote_addr
        );

        Ok(Response::new(Output {}))
    }
}

pub fn create() -> GamePadServiceServer<GamePadServiceImpl> {
    GamePadServiceServer::new(GamePadServiceImpl::default())
}
