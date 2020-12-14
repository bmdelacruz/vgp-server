use crate::proto::io_prelude::*;
use uinput::event::{
    absolute::{Absolute::Position as PositionEvent, Position as Positions},
    controller::{
        Controller::{DPad as DPadEvent, GamePad as GamePadEvent},
        DPad as DPadKeys, GamePad as GamePadKeys,
    },
    Event::{Absolute, Controller},
    Position, Press, Release,
};
use uinput::Device;

pub struct GamePad {
    device: Device,
}

impl GamePad {
    pub fn create() -> Result<Self, Box<dyn std::error::Error>> {
        let device = uinput::default()?
            .name("virtual gamepad (vpg)")?
            .vendor(0x0bdc)
            .product(0x4853)
            .version(1)
            .bus(0x06) // virtual bus
            .event(Controller(DPadEvent(DPadKeys::Up)))?
            .event(Controller(DPadEvent(DPadKeys::Down)))?
            .event(Controller(DPadEvent(DPadKeys::Left)))?
            .event(Controller(DPadEvent(DPadKeys::Right)))?
            .event(Controller(GamePadEvent(GamePadKeys::A)))?
            .event(Controller(GamePadEvent(GamePadKeys::B)))?
            .event(Controller(GamePadEvent(GamePadKeys::X)))?
            .event(Controller(GamePadEvent(GamePadKeys::Y)))?
            .event(Controller(GamePadEvent(GamePadKeys::TL)))?
            .event(Controller(GamePadEvent(GamePadKeys::TR)))?
            .event(Controller(GamePadEvent(GamePadKeys::TL2)))?
            .event(Controller(GamePadEvent(GamePadKeys::TR2)))?
            .event(Controller(GamePadEvent(GamePadKeys::ThumbL)))?
            .event(Controller(GamePadEvent(GamePadKeys::ThumbR)))?
            .event(Controller(GamePadEvent(GamePadKeys::Select)))?
            .event(Controller(GamePadEvent(GamePadKeys::Start)))?
            .event(Absolute(PositionEvent(Positions::X)))?
            .min(-512)
            .max(512)
            .fuzz(0)
            .flat(15)
            .event(Absolute(PositionEvent(Positions::Y)))?
            .min(-512)
            .max(512)
            .fuzz(0)
            .flat(15)
            .event(Absolute(PositionEvent(Positions::RX)))?
            .min(-512)
            .max(512)
            .fuzz(0)
            .flat(15)
            .event(Absolute(PositionEvent(Positions::RY)))?
            .min(-512)
            .max(512)
            .fuzz(0)
            .flat(15)
            .create()?;

        Ok(GamePad { device })
    }

    pub fn control(&mut self, data: Control) -> Result<(), Box<dyn std::error::Error>> {
        match data {
            Control::Button(b) => {
                let button_type =
                    ButtonType::from_i32(b.r#type).ok_or("The `button.type` is invalid!")?;
                match button_type {
                    ButtonType::A => self.apply_gamepad_key_state(&GamePadKeys::A, b.state)?,
                    ButtonType::B => self.apply_gamepad_key_state(&GamePadKeys::B, b.state)?,
                    ButtonType::X => self.apply_gamepad_key_state(&GamePadKeys::X, b.state)?,
                    ButtonType::Y => self.apply_gamepad_key_state(&GamePadKeys::Y, b.state)?,
                    ButtonType::Up => self.apply_dpad_key_state(&DPadKeys::Up, b.state)?,
                    ButtonType::Down => self.apply_dpad_key_state(&DPadKeys::Down, b.state)?,
                    ButtonType::Left => self.apply_dpad_key_state(&DPadKeys::Left, b.state)?,
                    ButtonType::Right => self.apply_dpad_key_state(&DPadKeys::Right, b.state)?,
                    ButtonType::TriggerLeft => {
                        self.apply_gamepad_key_state(&GamePadKeys::TL, b.state)?
                    }
                    ButtonType::TriggerRight => {
                        self.apply_gamepad_key_state(&GamePadKeys::TR, b.state)?
                    }
                    ButtonType::Trigger2Left => {
                        self.apply_gamepad_key_state(&GamePadKeys::TL2, b.state)?
                    }
                    ButtonType::Trigger2Right => {
                        self.apply_gamepad_key_state(&GamePadKeys::TR2, b.state)?
                    }
                    ButtonType::ThumbLeft => {
                        self.apply_gamepad_key_state(&GamePadKeys::ThumbL, b.state)?
                    }
                    ButtonType::ThumbRight => {
                        self.apply_gamepad_key_state(&GamePadKeys::ThumbR, b.state)?
                    }
                    ButtonType::Start => {
                        self.apply_gamepad_key_state(&GamePadKeys::Start, b.state)?
                    }
                    ButtonType::Select => {
                        self.apply_gamepad_key_state(&GamePadKeys::Select, b.state)?
                    }
                }
            }
            Control::Position(p) => {
                let position_type =
                    PositionType::from_i32(p.r#type).ok_or("The `position.type` is invalid!")?;
                match position_type {
                    PositionType::LeftX => self.apply_position(&Positions::X, p.position)?,
                    PositionType::LeftY => self.apply_position(&Positions::Y, p.position)?,
                    PositionType::RightX => self.apply_position(&Positions::RX, p.position)?,
                    PositionType::RightY => self.apply_position(&Positions::RY, p.position)?,
                }
            }
        }

        Ok(())
    }

    fn apply_gamepad_key_state(
        &mut self,
        event: &GamePadKeys,
        state: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let button_state = ButtonState::from_i32(state).ok_or("The `button.state` is invalid!")?;
        match button_state {
            ButtonState::Pressed => self.press_key(event)?,
            ButtonState::Released => self.release_key(event)?,
        }

        Ok(())
    }

    fn apply_dpad_key_state(
        &mut self,
        event: &DPadKeys,
        state: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let button_state = ButtonState::from_i32(state).ok_or("The `button.state` is invalid!")?;
        match button_state {
            ButtonState::Pressed => self.press_key(event)?,
            ButtonState::Released => self.release_key(event)?,
        }

        Ok(())
    }

    fn apply_position<T: Position>(
        &mut self,
        event: &T,
        value: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.device.position(event, value)?;
        self.device.synchronize()?;

        Ok(())
    }

    fn press_key<T: Press>(&mut self, event: &T) -> Result<(), Box<dyn std::error::Error>> {
        self.device.press(event)?;
        self.device.synchronize()?;

        Ok(())
    }

    fn release_key<T: Release>(&mut self, event: &T) -> Result<(), Box<dyn std::error::Error>> {
        self.device.release(event)?;
        self.device.synchronize()?;

        Ok(())
    }
}
