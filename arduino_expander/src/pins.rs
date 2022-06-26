use arduino_hal::hal::port::{
    mode::{Floating, Input, Output, PullUp},
    Pin,
};
use core::{cell::Cell, fmt};
use embedded_hal::digital::v2::{self as hal_digital, OutputPin};
use gpio_actions::{PinLabel, PinName, PinState};
use heapless::FnvIndexMap;

fn convert_state(state: PinState) -> hal_digital::PinState {
    match state {
        PinState::High => hal_digital::PinState::High,
        PinState::Low => hal_digital::PinState::Low,
    }
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
        output_pin.set_state(convert_state(state)).unwrap();
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

pub struct MutablePin<T> {
    pin: Cell<Option<StatefulPin<T>>>,
    name: &'static str,
}

impl<T> MutablePin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    pub fn new(pin: Pin<Input<Floating>, T>, name: &'static str) -> Self {
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

pub trait IOPin: fmt::Debug {
    fn output_state(&mut self, state: PinState);
    fn input(&mut self) -> PinState;
    fn name(&self) -> PinName;
}

impl<T> IOPin for MutablePin<T>
where
    T: avr_hal_generic::port::PinOps,
{
    fn output_state(&mut self, state: PinState) {
        self.pin.set(Some(self.pin.take().unwrap().output_state(state)))
    }

    fn input(&mut self) -> PinState {
        let mut is_high = false;
        self.pin.set(Some(self.pin.take().unwrap().input(&mut is_high)));
        if is_high {
            PinState::High
        } else {
            PinState::Low
        }
    }

    fn name(&self) -> PinName {
        let mut name = PinName::default();
        for (i, c) in self.name.chars().enumerate() {
            name[i] = c;
        }
        name
    }
}

type PinMap<'a> = FnvIndexMap<PinLabel, &'a mut dyn IOPin, 64>;

#[derive(Default)]
pub struct PinDispatcher<'a> {
    pin_map: PinMap<'a>,
}

impl<'a> PinDispatcher<'a> {
    pub fn new() -> Self {
        PinDispatcher { pin_map: PinMap::new() }
    }

    pub fn add_pin(&mut self, pin_label: PinLabel, pin: &'a mut dyn IOPin) {
        let maybe_previous_pin = self.pin_map.insert(pin_label, pin).unwrap();
        if maybe_previous_pin.is_some() {
            panic!("Inserting pin failed because the pin_label was already in use.")
        }
    }

    pub fn output(&mut self, pin_label: PinLabel, state: PinState) {
        self.get_pin(pin_label).output_state(state);
    }

    pub fn input(&mut self, pin_label: PinLabel) -> PinState {
        self.get_pin(pin_label).input()
    }

    pub fn has_pin(&self, pin_label: PinLabel) -> bool {
        self.pin_map.contains_key(&pin_label)
    }

    fn get_pin(&mut self, pin_label: PinLabel) -> &mut dyn IOPin {
        *self.pin_map.get_mut(&pin_label).unwrap()
    }
}

impl<'a, 'b> IntoIterator for &'a PinDispatcher<'b> {
    type Item = <&'a PinMap<'b> as IntoIterator>::Item;
    type IntoIter = <&'a PinMap<'b> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.pin_map.iter()
    }
}

#[macro_export]
macro_rules! add_pin {
    ($dispatcher:ident, $pins:ident.$name:ident, $tag:literal) => {
        let mut $name = $crate::pins::MutablePin::new($pins.$name, stringify!($name));
        $dispatcher.add_pin($tag, &mut $name);
    };
}
