tonic::include_proto!("gamepad");

pub mod service_prelude {
    use super::input_data::Input;
    use super::{ButtonState, ButtonType, ThumbStickType};

    use vgp_device::{Button, ThumbStick};

    pub use super::game_pad_server::{GamePad, GamePadServer};
    pub use super::output_data::Output;
    pub use super::{CheckRequest, CheckResponse, InputData, OutputData, RumbleForceFeedback};

    pub trait ToVgpDeviceInputExt {
        fn to_vgp_device_input(self) -> Result<vgp_device::Input, &'static str>;
    }

    impl ToVgpDeviceInputExt for InputData {
        fn to_vgp_device_input(self) -> Result<vgp_device::Input, &'static str> {
            match &self.input {
                Some(Input::Button(b)) => {
                    let button_type =
                        ButtonType::from_i32(b.r#type).ok_or("The `button.type` is invalid!")?;
                    let button_state =
                        ButtonState::from_i32(b.state).ok_or("The `button.state` is invalid!")?;

                    let button = match button_type {
                        ButtonType::A => Button::South,
                        ButtonType::B => Button::East,
                        ButtonType::X => Button::West,
                        ButtonType::Y => Button::North,
                        ButtonType::Up => Button::DpadUp,
                        ButtonType::Down => Button::DpadDown,
                        ButtonType::Left => Button::DpadLeft,
                        ButtonType::Right => Button::DpadRight,
                        ButtonType::TriggerLeft => Button::TriggerLeft,
                        ButtonType::TriggerRight => Button::TriggerRight,
                        ButtonType::Trigger2Left => Button::TriggerLeft2,
                        ButtonType::Trigger2Right => Button::TriggerRight2,
                        ButtonType::ThumbLeft => Button::ThumbStickLeft,
                        ButtonType::ThumbRight => Button::ThumbStickRight,
                        ButtonType::Start => Button::Start,
                        ButtonType::Select => Button::Select,
                    };
                    match button_state {
                        ButtonState::Pressed => Ok(vgp_device::Input::Press(button)),
                        ButtonState::Released => Ok(vgp_device::Input::Release(button)),
                    }
                }
                Some(Input::ThumbStick(t)) => {
                    let thumb_stick_type = ThumbStickType::from_i32(t.r#type)
                        .ok_or("The `thumb_stick.type` is invalid!")?;

                    let thumb_stick = match thumb_stick_type {
                        ThumbStickType::LeftThumbStick => ThumbStick::Left,
                        ThumbStickType::RightThumbStick => ThumbStick::Right,
                    };

                    Ok(vgp_device::Input::Move {
                        thumb_stick,
                        x: t.x,
                        y: t.y,
                    })
                }
                _ => Err("Invalid input data!"),
            }
        }
    }

    pub trait ToProtoOutputDataExt {
        fn to_proto_output_data(self) -> Option<Output>;
    }

    impl ToProtoOutputDataExt for vgp_device::Output {
        fn to_proto_output_data(self) -> Option<Output> {
            match self {
                vgp_device::Output::None => None,
                vgp_device::Output::Unsupported => None,
                vgp_device::Output::Rumble {
                    large_motor,
                    small_motor,
                } => Some(Output::Rumble(RumbleForceFeedback {
                    strong_magnitude: large_motor.into(),
                    weak_magnitude: small_motor.into(),
                })),
            }
        }
    }
}
