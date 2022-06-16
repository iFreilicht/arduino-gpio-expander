#![no_std]
#![no_main]

use arduino_hal::hal::port::{
    mode::{Floating, Input, Output, PullUp},
    Pin,
};
use core::{cell::Cell, fmt};
use embedded_hal::digital::v2::{OutputPin, PinState};
use panic_halt as _;

use heapless::FnvIndexMap;

enum Action {
    Output(PinState),
    Input,
}

enum StatefulPin<T> {
    Input(Pin<Input<PullUp>, T>),
    Output(Pin<Output, T>),
}

impl<T> StatefulPin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn output_state(self, state: PinState) -> Self {
        let mut output_pin = match self {
            StatefulPin::Input(input_pin) => input_pin.into_output(),
            StatefulPin::Output(output_pin) => output_pin,
        };
        output_pin.set_state(state).unwrap();
        StatefulPin::Output(output_pin)
    }

    fn input(self, is_high: &mut bool) -> Self {
        let input_pin = match self {
            StatefulPin::Input(input_pin) => input_pin,
            StatefulPin::Output(output_pin) => output_pin.into_pull_up_input(),
        };
        *is_high = input_pin.is_high();
        StatefulPin::Input(input_pin)
    }
}

struct MutablePin<T> {
    pin: Cell<Option<StatefulPin<T>>>,
    name: &'static str,
}

impl<T> MutablePin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn new(pin: Pin<Input<Floating>, T>, name: &'static str) -> Self {
        Self {
            pin: Cell::new(Some(StatefulPin::Input(pin.into_pull_up_input()))),
            name,
        }
    }
}

impl<T> fmt::Debug for MutablePin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("MutablePin {}", self.name))
    }
}

trait IOPin: fmt::Debug {
    fn output_state(&mut self, state: PinState);
    fn input(&mut self) -> bool;
}

impl<T> IOPin for MutablePin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn output_state(&mut self, state: PinState) {
        self.pin.set(Some(self.pin.take().unwrap().output_state(state)))
    }

    fn input(&mut self) -> bool {
        let mut is_high = false;
        self.pin.set(Some(self.pin.take().unwrap().input(&mut is_high)));
        is_high
    }
}

type PinMap<'a> = FnvIndexMap<char, &'a mut dyn IOPin, 64>;

struct PinDispatcher<'a> {
    pin_map: PinMap<'a>,
}

impl<'a> PinDispatcher<'a> {
    fn new() -> Self {
        PinDispatcher { pin_map: PinMap::new() }
    }

    fn add_pin(&mut self, pin_label: char, pin: &'a mut dyn IOPin) {
        self.pin_map.insert(pin_label, pin).unwrap();
    }

    fn output(&mut self, pin_label: char, state: PinState) {
        self.get_pin(pin_label).output_state(state);
    }

    fn input(&mut self, pin_label: char) -> bool {
        self.get_pin(pin_label).input()
    }

    fn get_pin(&mut self, pin_label: char) -> &mut dyn IOPin {
        *self.pin_map.get_mut(&pin_label).unwrap()
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

    let mut pin_dispatcher = PinDispatcher::new();
    let mut d13 = MutablePin::new(pins.d13, "D13");
    pin_dispatcher.add_pin('1', &mut d13);
    let mut d2 = MutablePin::new(pins.d2, "D2");
    pin_dispatcher.add_pin('2', &mut d2);
    let mut d3 = MutablePin::new(pins.d3, "D3");
    pin_dispatcher.add_pin('3', &mut d3);
    let mut d4 = MutablePin::new(pins.d4, "D4");
    pin_dispatcher.add_pin('4', &mut d4);
    let mut d5 = MutablePin::new(pins.d5, "D5");
    pin_dispatcher.add_pin('5', &mut d5);
    let mut d6 = MutablePin::new(pins.d6, "D6");
    pin_dispatcher.add_pin('6', &mut d6);
    let mut d7 = MutablePin::new(pins.d7, "D7");
    pin_dispatcher.add_pin('7', &mut d7);

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
