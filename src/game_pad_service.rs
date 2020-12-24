use std::{net::SocketAddr, pin::Pin, sync::Arc};

use futures_core::Stream;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status, Streaming};
use vgp_device::{VgpDevice, VgpDeviceEvent, VgpDeviceForceFeedbackType, VgpDeviceImpl};

use crate::proto::service_prelude::*;

struct VgpDeviceWrapper {
    device: VgpDeviceImpl,
    should_stop_reading: bool,
    should_stop_writing: bool,
    remote_addr: Option<SocketAddr>,
}

impl Drop for VgpDeviceWrapper {
    fn drop(&mut self) {
        log::info!(
            "Destroying game pad that was made for client ({:?}).",
            self.remote_addr
        );
    }
}

#[derive(Debug, Default)]
pub struct GamePadImpl;

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
            "Client ({:?}) requested to instantiate game pad.",
            remote_addr
        );

        let mut stream = request.into_inner();

        let device = VgpDeviceImpl::new().map_err(|e| {
            log::error!(
                "An error occurred while trying to create game pad device. {}",
                e
            );
            Status::internal("An error occurred while trying to create game pad device.")
        })?;

        log::info!("Instantiated game pad for client ({:?}).", remote_addr);

        let device_wrapper = VgpDeviceWrapper {
            device,
            remote_addr,
            should_stop_reading: false,
            should_stop_writing: false,
        };
        let device_wrapper = Arc::new(Mutex::new(device_wrapper));
        let device_wrapper_clone = Arc::clone(&device_wrapper);

        tokio::spawn(async move {
            while let Some(input) = stream.next().await {
                let mut device_wrapper = device_wrapper.lock().await;

                if (device_wrapper.should_stop_writing) {
                    log::info!(
                        "Stopping receiving input data from client ({:?}). Read already stopped.",
                        remote_addr,
                    );
                    break;
                }

                let vgp_device_input = match input {
                    Ok(input) => input.to_vgp_device_input(),
                    Err(status) => {
                        log::info!(
                            "Stopping receiving input data from client ({:?}). Received status ({:?}).",
                            remote_addr,
                            status,
                        );

                        device_wrapper.should_stop_reading = true;

                        break;
                    }
                };
                let make_input_result = match vgp_device_input {
                    Ok(input) => {
                        log::info!(
                            "Received game pad input data ({:?}) from client ({:?}).",
                            input,
                            remote_addr,
                        );

                        device_wrapper.device.make_input(input)
                    }
                    Err(e) => {
                        log::error!(
                            "Received an invalid game pad input data from client ({:?}). Error: {}",
                            remote_addr,
                            e
                        );

                        device_wrapper.should_stop_reading = true;

                        break;
                    }
                };
                match make_input_result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "An error occurred while acknowledging input for the game pad device from client ({:?}). Error: {}",
                            remote_addr, e
                        );

                        device_wrapper.should_stop_reading = true;

                        break;
                    }
                }
            }
        });

        let output_stream = async_stream::try_stream! {
            loop {
                let mut device_wrapper = device_wrapper_clone.lock().await;

                if (device_wrapper.should_stop_reading) {
                    log::info!(
                        "Stopping sending output data to client ({:?}). Write already stopped.",
                        remote_addr,
                    );
                    break;
                }

                let event = match device_wrapper.device.get_next_event() {
                    Ok(e) => e,
                    Err(e) => {
                        log::error!(
                            "An error occurred while trying to read the game pad device for client ({:?}). Error: {}",
                            remote_addr, e
                        );

                        device_wrapper.should_stop_writing = true;

                        break;
                    }
                };
                match event {
                    VgpDeviceEvent::None => {}
                    VgpDeviceEvent::Unsupported => {
                        log::warn!(
                            "Received an unsupported event from game pad device for client ({:?}",
                            remote_addr
                        );
                    }
                    VgpDeviceEvent::ForceFeedbackUploaded(ff) => match ff.r#type {
                        VgpDeviceForceFeedbackType::Unsupported => {}
                        VgpDeviceForceFeedbackType::Rumble {
                            strong_magnitude,
                            weak_magnitude,
                        } => {
                            log::info!(
                                "Uploading rumble ff to game pad device for client ({:?}",
                                remote_addr
                            );

                            yield OutputData {
                                feedback: Some(Feedback::FfUploaded(ForceFeedbackUploaded {
                                    id: ff.id as i32,
                                    direction: ff.direction as u32,
                                    replay_length: ff.replay.length as u32,
                                    replay_delay: ff.replay.delay as u32,
                                    r#type: Some(ForceFeedbackType::Rumble(RumbleForceFeedback {
                                        strong_magnitude: strong_magnitude as u32,
                                        weak_magnitude: weak_magnitude as u32,
                                    })),
                                })),
                            };
                        }
                    },
                    VgpDeviceEvent::ForceFeedbackErased(ff_id) => {
                        log::info!(
                            "Erasing rumble ff from game pad device for client ({:?}",
                            remote_addr
                        );

                        yield OutputData {
                            feedback: Some(Feedback::FfErased(ForceFeedbackErased {
                                id: ff_id as i32,
                            })),
                        };
                    }
                }
            }
        };

        Ok(Response::new(
            Box::pin(output_stream) as Self::InstantiateStream
        ))
    }
}

pub fn create() -> GamePadServer<GamePadImpl> {
    GamePadServer::new(GamePadImpl::default())
}
