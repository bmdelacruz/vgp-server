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
        let mut stream = request.into_inner();

        let mut game_pad = GamePad::create().map_err(|_e| {
            Status::internal("An error occurred while trying to create game pad device.")
        })?;

        while let Some(input) = stream.next().await {
            let data = input?.control.ok_or(Status::invalid_argument(
                "The `control` argument is missing!",
            ))?;

            game_pad.control(data).map_err(|_e| {
                Status::internal("An error occurred while trying to control the game pad device.")
            })?;
        }

        Ok(Response::new(Output {}))
    }
}

pub fn create() -> GamePadServiceServer<GamePadServiceImpl> {
    GamePadServiceServer::new(GamePadServiceImpl::default())
}
