use futures_util::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use vgp_device::{VgpDevice, VgpDeviceImpl};

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

        let mut game_pad = VgpDeviceImpl::new().map_err(|e| {
            log::error!(
                "An error occurred while trying to create game pad device. {}",
                e
            );
            Status::internal("An error occurred while trying to create game pad device.")
        })?;

        log::info!("Instantiated game pad for client ({:?}).", remote_addr);

        while let Some(input) = stream.next().await {
            let input = input?
                .to_vgp_device_input()
                .map_err(|e| Status::invalid_argument(e))?;

            log::info!(
                "Received game pad input data ({:?}) from client ({:?}).",
                input,
                remote_addr,
            );

            game_pad.make_input(input).map_err(|e| {
                log::error!(
                    "An error occurred while acknowledging input for the game pad device. {}",
                    e
                );
                Status::internal(
                    "An error occurred while acknowledging input for the game pad device.",
                )
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
