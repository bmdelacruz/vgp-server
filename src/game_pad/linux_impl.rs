use std::convert::TryFrom;

use crate::proto::io_prelude::*;
use evdev_rs::{
    enums::{BusType, EventCode, EventType, EV_ABS, EV_FF, EV_KEY, EV_SYN},
    AbsInfo, InputEvent, TimeVal,
};
use evdev_rs::{Device, UInputDevice};

pub struct GamePad {
    _device: Device,
    input_device: UInputDevice,
    abs_val: f32,
}

unsafe impl Send for GamePad {}

impl GamePad {
    pub fn create(abs_val: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let device = Device::new().ok_or("Failed to create evdev device!")?;
        device.set_name("virtual gamepad (vpg)");
        device.set_vendor_id(0x0bdc);
        device.set_product_id(0x4853);
        device.set_version(1);
        device.set_bustype(BusType::BUS_VIRTUAL as u16);

        device.enable(&EventType::EV_KEY)?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_UP))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_DOWN))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_LEFT))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_RIGHT))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_NORTH))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_SOUTH))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_EAST))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_WEST))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TL))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TR))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TL2))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_TR2))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_THUMBL))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_THUMBR))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_SELECT))?;
        device.enable(&EventCode::EV_KEY(EV_KEY::BTN_START))?;

        device.enable(&EventType::EV_ABS)?;
        device.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_X),
            Some(&AbsInfo {
                value: 0,
                minimum: -512,
                maximum: 512,
                fuzz: 0,
                flat: 15,
                resolution: 0,
            }),
        )?;
        device.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_Y),
            Some(&AbsInfo {
                value: 0,
                minimum: -512,
                maximum: 512,
                fuzz: 0,
                flat: 15,
                resolution: 0,
            }),
        )?;
        device.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_RX),
            Some(&AbsInfo {
                value: 0,
                minimum: -512,
                maximum: 512,
                fuzz: 0,
                flat: 15,
                resolution: 0,
            }),
        )?;
        device.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_RY),
            Some(&AbsInfo {
                value: 0,
                minimum: -512,
                maximum: 512,
                fuzz: 0,
                flat: 15,
                resolution: 0,
            }),
        )?;

        device.enable(&EventType::EV_FF)?;
        device.enable(&EventCode::EV_FF(EV_FF::FF_PERIODIC))?;

        let input_device = UInputDevice::create_from_device(&device)?;

        Ok(GamePad {
            _device: device,
            input_device,
            abs_val,
        })
    }

    pub fn control(&mut self, data: Control) -> Result<(), Box<dyn std::error::Error>> {
        let time_val = TimeVal::try_from(std::time::SystemTime::now())?;
        match data {
            Control::Button(b) => {
                let button_type =
                    ButtonType::from_i32(b.r#type).ok_or("The `button.type` is invalid!")?;
                let button_state =
                    ButtonState::from_i32(b.state).ok_or("The `button.state` is invalid!")?;

                let ev_key = match button_type {
                    ButtonType::A => EV_KEY::BTN_SOUTH,
                    ButtonType::B => EV_KEY::BTN_EAST,
                    ButtonType::X => EV_KEY::BTN_WEST,
                    ButtonType::Y => EV_KEY::BTN_NORTH,
                    ButtonType::Up => EV_KEY::BTN_DPAD_UP,
                    ButtonType::Down => EV_KEY::BTN_DPAD_DOWN,
                    ButtonType::Left => EV_KEY::BTN_DPAD_LEFT,
                    ButtonType::Right => EV_KEY::BTN_DPAD_RIGHT,
                    ButtonType::TriggerLeft => EV_KEY::BTN_TL,
                    ButtonType::TriggerRight => EV_KEY::BTN_TR,
                    ButtonType::Trigger2Left => EV_KEY::BTN_TL2,
                    ButtonType::Trigger2Right => EV_KEY::BTN_TR2,
                    ButtonType::ThumbLeft => EV_KEY::BTN_THUMBL,
                    ButtonType::ThumbRight => EV_KEY::BTN_THUMBR,
                    ButtonType::Start => EV_KEY::BTN_START,
                    ButtonType::Select => EV_KEY::BTN_SELECT,
                };
                let value = match button_state {
                    ButtonState::Pressed => 1,
                    ButtonState::Released => 0,
                };

                let input_event = InputEvent::new(&time_val, &EventCode::EV_KEY(ev_key), value);
                self.input_device.write_event(&input_event)?;
            }
            Control::ThumbStick(t) => {
                let thumb_stick_type = ThumbStickType::from_i32(t.r#type)
                    .ok_or("The `thumb_stick.type` is invalid!")?;

                let (ev_abs_x, ev_abs_y) = match thumb_stick_type {
                    ThumbStickType::LeftThumbStick => (EV_ABS::ABS_X, EV_ABS::ABS_Y),
                    ThumbStickType::RightThumbStick => (EV_ABS::ABS_RX, EV_ABS::ABS_RY),
                };

                let input_event = InputEvent::new(
                    &time_val,
                    &EventCode::EV_ABS(ev_abs_x),
                    (t.x * self.abs_val) as i32,
                );
                self.input_device.write_event(&input_event)?;

                let input_event = InputEvent::new(
                    &time_val,
                    &EventCode::EV_ABS(ev_abs_y),
                    (t.y * self.abs_val) as i32,
                );
                self.input_device.write_event(&input_event)?;
            }
        };

        let syn_event = InputEvent::new(&time_val, &EventCode::EV_SYN(EV_SYN::SYN_REPORT), 0);
        self.input_device.write_event(&syn_event)?;

        Ok(())
    }
}
