#![no_std]
#![no_main]

use arduino_hal::port::{
    mode::{Input, Output, PullUp},
    Pin,
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

impl<T> IOPin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn output_state(self, state: PinState) -> Self {
        let mut output_pin = match self {
            IOPin::Input(input_pin) => input_pin.into_output(),
            IOPin::Output(output_pin) => output_pin,
        };
        output_pin.set_state(state).unwrap();
        IOPin::Output(output_pin)
    }

    fn input(self) -> (Self, bool) {
        let input_pin = match self {
            IOPin::Input(input_pin) => input_pin,
            IOPin::Output(output_pin) => output_pin.into_pull_up_input(),
        };
        let is_high = input_pin.is_high();
        (IOPin::Input(input_pin), is_high)
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

    let mut d2 = IOPin::Input(pins.d2.into_pull_up_input());
    let mut d3 = IOPin::Input(pins.d3.into_pull_up_input());
    let mut d4 = IOPin::Input(pins.d4.into_pull_up_input());
    let mut d5 = IOPin::Input(pins.d5.into_pull_up_input());
    let mut d6 = IOPin::Input(pins.d6.into_pull_up_input());
    let mut d7 = IOPin::Input(pins.d7.into_pull_up_input());
    let mut led = IOPin::Input(pins.d13.into_pull_up_input());

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
                    '1' => led = led.output_state(state),
                    '2' => d2 = d2.output_state(state),
                    '3' => d3 = d3.output_state(state),
                    '4' => d4 = d4.output_state(state),
                    '5' => d5 = d5.output_state(state),
                    '6' => d6 = d6.output_state(state),
                    '7' => d7 = d7.output_state(state),
                    _ => unreachable!(),
                },
                Action::Input => {
                    let is_high;
                    match pin {
                        '1' => (led, is_high) = led.input(),
                        '2' => (d2, is_high) = d2.input(),
                        '3' => (d3, is_high) = d3.input(),
                        '4' => (d4, is_high) = d4.input(),
                        '5' => (d5, is_high) = d5.input(),
                        '6' => (d6, is_high) = d6.input(),
                        '7' => (d7, is_high) = d7.input(),
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
