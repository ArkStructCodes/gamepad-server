use std::io::{Cursor, Read, Result};

use bitvec::{array::BitArray, order::Lsb0};
use evdev::{AbsInfo, AbsoluteAxisCode, InputEvent, KeyCode, uinput::VirtualDevice};
use log::trace;

use crate::{axis, event, keys};

pub(crate) struct Gamepad {
    device: VirtualDevice,
}

impl Gamepad {
    pub fn new(name: &str) -> Result<Self> {
        let stick = AbsInfo::new(0, -32768, 32767, 16, 128, 1);
        let trigger = AbsInfo::new(0, 0, 255, 0, 0, 1);
        let dpad = AbsInfo::new(0, -1, 1, 0, 0, 1);

        let device = VirtualDevice::builder()?
            .name(name)
            .with_absolute_axis(axis!(ABS_X, stick))?
            .with_absolute_axis(axis!(ABS_Y, stick))?
            .with_absolute_axis(axis!(ABS_RX, stick))?
            .with_absolute_axis(axis!(ABS_RY, stick))?
            .with_absolute_axis(axis!(ABS_Z, trigger))?
            .with_absolute_axis(axis!(ABS_RZ, trigger))?
            .with_absolute_axis(axis!(ABS_HAT0X, dpad))?
            .with_absolute_axis(axis!(ABS_HAT0Y, dpad))?
            .with_keys(keys!(
                BTN_START, BTN_SELECT, // menu buttons
                BTN_THUMBL, BTN_THUMBR, // thumbsticks
                BTN_TL, BTN_TR, // bumpers
                BTN_SOUTH, BTN_EAST, BTN_WEST, BTN_NORTH, // face buttons
            ))?
            .build()?;

        Ok(Gamepad { device })
    }

    pub fn emit(&mut self, input: GamepadInput) -> Result<()> {
        trace!("attempting to emit events: {:?}", input.events);
        self.device.emit(&input.events)
    }
}

pub(crate) struct GamepadInput {
    events: [InputEvent; 18],
}

fn calculate_axis(a: bool, b: bool) -> i32 {
    if a ^ b { a as i32 - b as i32 } else { 0 }
}

impl TryFrom<&[u8; 14]> for GamepadInput {
    type Error = std::io::Error;

    fn try_from(data: &[u8; 14]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let mut buffer = [0; 2];

        reader.read_exact(&mut buffer)?;
        let buttons = BitArray::<_, Lsb0>::new(buffer);

        let dpad_y = calculate_axis(buttons[0], buttons[1]);
        let dpad_x = calculate_axis(buttons[2], buttons[3]);

        reader.read_exact(&mut buffer)?;
        let [left_trigger, right_trigger] = buffer;

        reader.read_exact(&mut buffer)?;
        let left_stick_x_axis = i16::from_le_bytes(buffer);

        reader.read_exact(&mut buffer)?;
        let left_stick_y_axis = i16::from_le_bytes(buffer);

        reader.read_exact(&mut buffer)?;
        let right_stick_x_axis = i16::from_le_bytes(buffer);

        reader.read_exact(&mut buffer)?;
        let right_stick_y_axis = i16::from_le_bytes(buffer);

        let events = [
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_X, left_stick_x_axis),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_Y, left_stick_y_axis),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_RX, right_stick_x_axis),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_RY, right_stick_y_axis),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_Z, left_trigger),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_RZ, right_trigger),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_HAT0X, dpad_x),
            event!(ABSOLUTE, AbsoluteAxisCode::ABS_HAT0Y, dpad_y),
            event!(KEY, KeyCode::BTN_SELECT, buttons[4]),
            event!(KEY, KeyCode::BTN_START, buttons[5]),
            event!(KEY, KeyCode::BTN_THUMBL, buttons[6]),
            event!(KEY, KeyCode::BTN_THUMBR, buttons[7]),
            event!(KEY, KeyCode::BTN_TL, buttons[8]),
            event!(KEY, KeyCode::BTN_TR, buttons[9]),
            event!(KEY, KeyCode::BTN_SOUTH, buttons[12]),
            event!(KEY, KeyCode::BTN_EAST, buttons[13]),
            event!(KEY, KeyCode::BTN_WEST, buttons[14]),
            event!(KEY, KeyCode::BTN_NORTH, buttons[15]),
        ];

        Ok(Self { events })
    }
}
