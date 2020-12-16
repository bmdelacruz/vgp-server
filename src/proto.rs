tonic::include_proto!("gamepad");

pub mod service_prelude {
    pub use super::game_pad_server::{GamePad, GamePadServer};
    pub use super::{CheckRequest, CheckResponse, InputData, OutputData};
}

pub mod io_prelude {
    pub use super::input_data::Control;
    pub use super::{ButtonState, ButtonType, ThumbStickType};
}
