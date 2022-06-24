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

    let mut pin_dispatcher = PinDispatcher::new();
    add_pin!(pin_dispatcher, pins.d13, '1');
    add_pin!(pin_dispatcher, pins.d2, '2');
    add_pin!(pin_dispatcher, pins.d3, '3');
    add_pin!(pin_dispatcher, pins.d4, '4');
    add_pin!(pin_dispatcher, pins.d5, '5');
    add_pin!(pin_dispatcher, pins.d6, '6');
    add_pin!(pin_dispatcher, pins.d7, '7');
    add_pin!(pin_dispatcher, pins.d8, '8');
    add_pin!(pin_dispatcher, pins.d9, '9');
    add_pin!(pin_dispatcher, pins.d10, 'a');
    add_pin!(pin_dispatcher, pins.d11, 'b');
    add_pin!(pin_dispatcher, pins.d12, 'c');

    add_pin!(pin_dispatcher, pins.a0, 'A');
    add_pin!(pin_dispatcher, pins.a1, 'B');
    add_pin!(pin_dispatcher, pins.a2, 'C');
    add_pin!(pin_dispatcher, pins.a3, 'D');
    add_pin!(pin_dispatcher, pins.a4, 'E');
    add_pin!(pin_dispatcher, pins.a5, 'F');

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
