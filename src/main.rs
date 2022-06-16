#![no_std]
#![no_main]

use core::cell::Cell;

use arduino_hal::{
    hal::port::{PB5, PD2, PD3, PD4, PD5, PD6, PD7},
    port::{
        mode::{Floating, Input, Output, PullUp},
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

    fn input(self, is_high: &mut bool) -> Self {
        let input_pin = match self {
            IOPin::Input(input_pin) => input_pin,
            IOPin::Output(output_pin) => output_pin.into_pull_up_input(),
        };
        *is_high = input_pin.is_high();
        IOPin::Input(input_pin)
    }
}

fn new_pin<T>(pin: Pin<Input<Floating>, T>) -> Cell<Option<IOPin<T>>>
where
    T: avr_hal_generic::port::PinOps,
{
    Cell::new(Some(IOPin::Input(pin.into_pull_up_input())))
}

struct PinDispatcher {
    d13: Cell<Option<IOPin<PB5>>>,
    d2: Cell<Option<IOPin<PD2>>>,
    d3: Cell<Option<IOPin<PD3>>>,
    d4: Cell<Option<IOPin<PD4>>>,
    d5: Cell<Option<IOPin<PD5>>>,
    d6: Cell<Option<IOPin<PD6>>>,
    d7: Cell<Option<IOPin<PD7>>>,
}

impl PinDispatcher {
    fn output(&self, pin_label: char, state: PinState) {
        match pin_label {
            '1' => self.d13.set(Some(self.d13.take().unwrap().output_state(state))),
            '2' => self.d2.set(Some(self.d2.take().unwrap().output_state(state))),
            '3' => self.d3.set(Some(self.d3.take().unwrap().output_state(state))),
            '4' => self.d4.set(Some(self.d4.take().unwrap().output_state(state))),
            '5' => self.d5.set(Some(self.d5.take().unwrap().output_state(state))),
            '6' => self.d6.set(Some(self.d6.take().unwrap().output_state(state))),
            '7' => self.d7.set(Some(self.d7.take().unwrap().output_state(state))),
            _ => unreachable!(),
        };
    }

    fn input(&mut self, pin_label: char) -> bool {
        let mut is_high = false;
        match pin_label {
            '1' => self.d13.set(Some(self.d13.take().unwrap().input(&mut is_high))),
            '2' => self.d2.set(Some(self.d2.take().unwrap().input(&mut is_high))),
            '3' => self.d3.set(Some(self.d3.take().unwrap().input(&mut is_high))),
            '4' => self.d4.set(Some(self.d4.take().unwrap().input(&mut is_high))),
            '5' => self.d5.set(Some(self.d5.take().unwrap().input(&mut is_high))),
            '6' => self.d6.set(Some(self.d6.take().unwrap().input(&mut is_high))),
            '7' => self.d7.set(Some(self.d7.take().unwrap().input(&mut is_high))),
            _ => unreachable!(),
        };
        is_high
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

    let mut pin_dispatcher = PinDispatcher {
        d13: new_pin(pins.d13),
        d2: new_pin(pins.d2),
        d3: new_pin(pins.d3),
        d4: new_pin(pins.d4),
        d5: new_pin(pins.d5),
        d6: new_pin(pins.d6),
        d7: new_pin(pins.d7),
    };

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
        if let (Some(pin_label), Some(action)) = (pin, action) {
            match action {
                Action::Output(state) => pin_dispatcher.output(pin_label, state),
                Action::Input => {
                    if pin_dispatcher.input(pin_label) {
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
