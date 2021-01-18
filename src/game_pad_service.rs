use std::{cell::RefCell, pin::Pin, sync::Arc};

use futures_core::Stream;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status, Streaming};
use vgp_device::Bus;

use crate::proto::service_prelude::*;

pub struct GamePadImpl {
    bus: Arc<Mutex<RefCell<Bus>>>,
}

#[tonic::async_trait]
impl GamePad for GamePadImpl {
    type InstantiateStream =
        Pin<Box<dyn Stream<Item = Result<OutputData, Status>> + Send + Sync + 'static>>;

    async fn check(
        &self,
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        Ok(Response::new(CheckResponse {}))
    }

    async fn instantiate(
        &self,
        request: Request<Streaming<InputData>>,
    ) -> Result<Response<Self::InstantiateStream>, Status> {
        let remote_addr = request.remote_addr();

        log::info!(
            "Client ({:?}) requested to instantiate game pad device.",
            remote_addr
        );

        let mut device = {
            let bus = self.bus.lock().await;
            let mut bbus = bus.borrow_mut();
            bbus.plug_in().unwrap()
        };

        let mut stream = request.into_inner();

        let (output_sender, output_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Result<OutputData, Status>>();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    input_data_opt = stream.next() => match input_data_opt {
                        None => break,
                        Some(input_data_res) => match input_data_res {
                            Err(e) => {
                                log::warn!(
                                    "Input error for client ({:?}). {:?}",
                                    remote_addr,
                                    e
                                );

                                break;
                            }
                            Ok(input_data) => match input_data.to_vgp_device_input() {
                                Err(e) => {
                                    log::warn!(
                                        "Received an unrecognized input from client ({:?}). {}",
                                        remote_addr,
                                        e
                                    );
                                }
                                Ok(input) => if let Err(e) = device.put_input(input) {
                                    log::error!(
                                        "An error occurred while placing input to game pad device for client ({:?}). {:?}",
                                        remote_addr,
                                        e
                                    );
                                }
                            }
                        }
                    },
                    output = device.get_output() => match output {
                        Some(output) => {
                            if let Err(e) = output_sender.send(Ok(OutputData {
                                output: output.to_proto_output_data(),
                            })) {
                                log::warn!(
                                    "Failed to send output to client ({:?}). {:?}",
                                    remote_addr,
                                    e
                                );
                            }
                        }
                        None => {
                            log::info!(
                                "No output from game pad device for client ({:?}).",
                                remote_addr,
                            );
                        }
                    },
                }
            }

            if let Err(e) = device.unplug() {
                log::error!(
                    "An error occurred while unplugging the device for client ({:?}). {:?}",
                    remote_addr,
                    e
                );
            }
        });

        Ok(Response::new(
            Box::pin(output_receiver) as Self::InstantiateStream
        ))
    }
}

pub fn create(bus: Bus) -> GamePadServer<GamePadImpl> {
    GamePadServer::new(GamePadImpl {
        bus: Arc::new(Mutex::new(RefCell::new(bus))),
    })
}
