#![no_std]
#![no_main]

use arduino_hal::{
    hal::port::{PB5, PD2, PD3, PD4, PD5, PD6, PD7},
    port::{
        mode::{Input, Output, PullUp},
        Pin,
    },
};
use embedded_hal::digital::v2::{OutputPin, PinState};
use panic_halt as _;

enum Action {
    Output(PinState),
    Input,
}

enum IOPin<T> {
    Input(Pin<Input<PullUp>, T>),
    Output(Pin<Output, T>),
}

enum UnoPin {
    D13(IOPin<PB5>),
    D2(IOPin<PD2>),
    D3(IOPin<PD3>),
    D4(IOPin<PD4>),
    D5(IOPin<PD5>),
    D6(IOPin<PD6>),
    D7(IOPin<PD7>),
}

fn output_state_io<T>(io_pin: IOPin<T>, state: PinState) -> IOPin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    let mut output_pin = match io_pin {
        IOPin::Input(input_pin) => input_pin.into_output(),
        IOPin::Output(output_pin) => output_pin,
    };
    output_pin.set_state(state).unwrap();
    IOPin::Output(output_pin)
}

fn output_state(uno_pin: UnoPin, state: PinState) -> UnoPin {
    match uno_pin {
        UnoPin::D13(io_pin) => UnoPin::D13(output_state_io(io_pin, state)),
        UnoPin::D2(io_pin) => UnoPin::D2(output_state_io(io_pin, state)),
        UnoPin::D3(io_pin) => UnoPin::D3(output_state_io(io_pin, state)),
        UnoPin::D4(io_pin) => UnoPin::D4(output_state_io(io_pin, state)),
        UnoPin::D5(io_pin) => UnoPin::D5(output_state_io(io_pin, state)),
        UnoPin::D6(io_pin) => UnoPin::D6(output_state_io(io_pin, state)),
        UnoPin::D7(io_pin) => UnoPin::D7(output_state_io(io_pin, state)),
    }
}

fn input_io<T>(io_pin: IOPin<T>) -> (IOPin<T>, bool)
where
    T: avr_hal_generic::port::PinOps,
{
    let input_pin = match io_pin {
        IOPin::Input(input_pin) => input_pin,
        IOPin::Output(output_pin) => output_pin.into_pull_up_input(),
    };
    let is_high = input_pin.is_high();
    (IOPin::Input(input_pin), is_high)
}

fn input(uno_pin: UnoPin) -> (UnoPin, bool) {
    match uno_pin {
        UnoPin::D13(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D13(pin), is_high)
        }
        UnoPin::D2(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D2(pin), is_high)
        }
        UnoPin::D3(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D3(pin), is_high)
        }
        UnoPin::D4(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D4(pin), is_high)
        }
        UnoPin::D5(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D5(pin), is_high)
        }
        UnoPin::D6(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D6(pin), is_high)
        }
        UnoPin::D7(io_pin) => {
            let (pin, is_high) = input_io(io_pin);
            (UnoPin::D7(pin), is_high)
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut d2 = UnoPin::D2(IOPin::Input(pins.d2.into_pull_up_input()));
    let mut d3 = UnoPin::D3(IOPin::Input(pins.d3.into_pull_up_input()));
    let mut d4 = UnoPin::D4(IOPin::Input(pins.d4.into_pull_up_input()));
    let mut d5 = UnoPin::D5(IOPin::Input(pins.d5.into_pull_up_input()));
    let mut d6 = UnoPin::D6(IOPin::Input(pins.d6.into_pull_up_input()));
    let mut d7 = UnoPin::D7(IOPin::Input(pins.d7.into_pull_up_input()));
    let mut led = UnoPin::D13(IOPin::Input(pins.d13.into_pull_up_input()));

    loop {
        let mut pin: Option<char> = None;
        let mut action: Option<Action> = None;

        match serial.read_byte() as char {
            '\n' => continue,
            '+' => action = Some(Action::Output(PinState::High)),
            '-' => action = Some(Action::Output(PinState::Low)),
            '?' => action = Some(Action::Input),
            byte => ufmt::uwrite!(&mut serial, "Invalid action '{}'. ", byte).unwrap(),
        }

        match serial.read_byte() as char {
            '\n' => continue,
            pin_num @ '1'..='7' => pin = Some(pin_num),
            byte => ufmt::uwrite!(&mut serial, "Invalid pin '{}'. ", byte).unwrap(),
        }

        while serial.read_byte() as char != '\n' {
            ufmt::uwrite!(&mut serial, "Expecting newline. ").unwrap();
        }
        if let (Some(pin), Some(action)) = (pin, action) {
            match action {
                Action::Output(state) => match pin {
                    '1' => led = output_state(led, state),
                    '2' => d2 = output_state(d2, state),
                    '3' => d3 = output_state(d3, state),
                    '4' => d4 = output_state(d4, state),
                    '5' => d5 = output_state(d5, state),
                    '6' => d6 = output_state(d6, state),
                    '7' => d7 = output_state(d7, state),
                    _ => unreachable!(),
                },
                Action::Input => {
                    let is_high;
                    match pin {
                        '1' => (led, is_high) = input(led),
                        '2' => (d2, is_high) = input(d2),
                        '3' => (d3, is_high) = input(d3),
                        '4' => (d4, is_high) = input(d4),
                        '5' => (d5, is_high) = input(d5),
                        '6' => (d6, is_high) = input(d6),
                        '7' => (d7, is_high) = input(d7),
                        _ => unreachable!(),
                    };
                    if is_high {
                        ufmt::uwrite!(&mut serial, "Pin is high. ").unwrap();
                    } else {
                        ufmt::uwrite!(&mut serial, "Pin is low. ").unwrap();
                    }
                }
            }

            ufmt::uwriteln!(&mut serial, "Done.").unwrap();
        } else {
            ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap();
        }
    }
}
