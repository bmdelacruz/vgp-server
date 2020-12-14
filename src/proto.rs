tonic::include_proto!("com.bmdelacruz.vgp");

pub mod service_prelude {
    pub use super::game_pad_service_server::{GamePadService, GamePadServiceServer};
    pub use super::{Input, Output};
}

pub mod io_prelude {
    pub use super::input::{
        button::{State as ButtonState, Type as ButtonType},
        position::Type as PositionType,
        Control,
    };
}
