tonic::include_proto!("gamepad");

pub mod service_prelude {
    use super::input_data::Control;
    use super::{ButtonState, ButtonType, ThumbStickType};

    use vgp_device::{VgpDeviceButton, VgpDeviceInput, VgpDeviceThumbStick};

    pub use super::game_pad_server::{GamePad, GamePadServer};
    pub use super::{CheckRequest, CheckResponse, InputData, OutputData};

    pub trait ToVgpDeviceInput {
        fn to_vgp_device_input(&self) -> Result<VgpDeviceInput, &'static str>;
    }

    impl ToVgpDeviceInput for InputData {
        fn to_vgp_device_input(&self) -> Result<VgpDeviceInput, &'static str> {
            match &self.control {
                Some(Control::Button(b)) => {
                    let button_type =
                        ButtonType::from_i32(b.r#type).ok_or("The `button.type` is invalid!")?;
                    let button_state =
                        ButtonState::from_i32(b.state).ok_or("The `button.state` is invalid!")?;

                    let button = match button_type {
                        ButtonType::A => VgpDeviceButton::South,
                        ButtonType::B => VgpDeviceButton::East,
                        ButtonType::X => VgpDeviceButton::West,
                        ButtonType::Y => VgpDeviceButton::North,
                        ButtonType::Up => VgpDeviceButton::DpadUp,
                        ButtonType::Down => VgpDeviceButton::DpadDown,
                        ButtonType::Left => VgpDeviceButton::DpadLeft,
                        ButtonType::Right => VgpDeviceButton::DpadRight,
                        ButtonType::TriggerLeft => VgpDeviceButton::TriggerLeft,
                        ButtonType::TriggerRight => VgpDeviceButton::TriggerRight,
                        ButtonType::Trigger2Left => VgpDeviceButton::TriggerLeft2,
                        ButtonType::Trigger2Right => VgpDeviceButton::TriggerRight2,
                        ButtonType::ThumbLeft => VgpDeviceButton::ThumbStickLeft,
                        ButtonType::ThumbRight => VgpDeviceButton::ThumbStickRight,
                        ButtonType::Start => VgpDeviceButton::Start,
                        ButtonType::Select => VgpDeviceButton::Select,
                    };
                    match button_state {
                        ButtonState::Pressed => Ok(VgpDeviceInput::PressButton(button)),
                        ButtonState::Released => Ok(VgpDeviceInput::ReleaseButton(button)),
                    }
                }
                Some(Control::ThumbStick(t)) => {
                    let thumb_stick_type = ThumbStickType::from_i32(t.r#type)
                        .ok_or("The `thumb_stick.type` is invalid!")?;

                    let thumb_stick = match thumb_stick_type {
                        ThumbStickType::LeftThumbStick => VgpDeviceThumbStick::Left,
                        ThumbStickType::RightThumbStick => VgpDeviceThumbStick::Right,
                    };

                    Ok(VgpDeviceInput::MoveThumbStick {
                        thumb_stick,
                        x: t.x,
                        y: t.y,
                    })
                }
                _ => Err("Invalid input data!"),
            }
        }
    }
}

pub mod io_prelude {
    pub use super::input_data::Control;
    pub use super::{ButtonState, ButtonType, ThumbStickType};
}
