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

macro_rules! add_pin {
    ($dispatcher:ident, $pins:ident.$name:ident, $tag:literal) => {
        let mut $name = MutablePin::new($pins.$name, stringify!($name));
        $dispatcher.add_pin($tag, &mut $name);
    };
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

        let maybe_pin = serial.read_byte() as char;
        if maybe_pin == '\n' {
            continue;
        } else if pin_dispatcher.pin_map.contains_key(&maybe_pin) {
            pin = Some(maybe_pin)
        } else {
            ufmt::uwrite!(&mut serial, "Invalid pin '{}'. ", maybe_pin).unwrap()
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
