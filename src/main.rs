#![no_std]
#![no_main]

mod pins;
use pins::PinDispatcher;

use embedded_hal::digital::v2::PinState;
use panic_halt as _;

enum Action {
    Output(PinState),
    Input,
    List,
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    create_pin_dispatcher!(
        let mut pin_dispatcher {
            '1' => pins.d13,
            '2' => pins.d2,
            '3' => pins.d3,
            '4' => pins.d4,
            '5' => pins.d5,
            '6' => pins.d6,
            '7' => pins.d7,
            '8' => pins.d8,
            '9' => pins.d9,
            'a' => pins.d10,
            'b' => pins.d11,
            'c' => pins.d12,
            'A' => pins.a0,
            'B' => pins.a1,
            'C' => pins.a2,
            'D' => pins.a3,
            'E' => pins.a4,
            'F' => pins.a5
    });

    loop {
        let mut pin: Option<char> = None;
        let mut action: Option<Action> = None;

        match serial.read_byte() as char {
            '\n' => continue,
            '+' => action = Some(Action::Output(PinState::High)),
            '-' => action = Some(Action::Output(PinState::Low)),
            '?' => action = Some(Action::Input),
            'l' => action = Some(Action::List),
            byte => ufmt::uwrite!(&mut serial, "Invalid action '{}'. ", byte).unwrap(),
        }

        if let Some(Action::List) = action {
        } else {
            let maybe_pin = serial.read_byte() as char;
            if maybe_pin == '\n' {
                continue;
            } else if pin_dispatcher.has_pin(maybe_pin) {
                pin = Some(maybe_pin)
            } else {
                ufmt::uwrite!(&mut serial, "Invalid pin '{}'. ", maybe_pin).unwrap()
            }
        }

        while serial.read_byte() as char != '\n' {
            ufmt::uwrite!(&mut serial, "Expecting newline. ").unwrap();
        }
        match (pin, action) {
            (Some(pin_label), Some(action)) => {
                match action {
                    Action::Output(state) => pin_dispatcher.output(pin_label, state),
                    Action::Input => {
                        if pin_dispatcher.input(pin_label) {
                            ufmt::uwrite!(&mut serial, "Pin is high. ").unwrap();
                        } else {
                            ufmt::uwrite!(&mut serial, "Pin is low. ").unwrap();
                        }
                    }
                    Action::List => unreachable!(),
                }
                ufmt::uwriteln!(&mut serial, "Done.").unwrap();
            }
            (_, Some(Action::List)) => {
                for (pin_label, pin) in &pin_dispatcher {
                    ufmt::uwriteln!(&mut serial, "{}: {}", pin_label, pin.name()).unwrap()
                }
            }
            _ => ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap(),
        }
    }
}
