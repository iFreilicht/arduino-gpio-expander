#![no_std]
#![no_main]

mod pins;
use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};
use gpio_actions::{Action, Response, TryFromIter, MAX_ACTION_WIRE_SIZE};
use heapless::Vec;
use pins::PinDispatcher;

use panic_halt as _;

type BoardSerial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

struct UnoSerial<'a>(&'a mut BoardSerial);

impl<'a> Iterator for UnoSerial<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        Some(self.0.read_byte())
    }
}

fn send_response(serial: &mut BoardSerial, response: Response) {
    // We have to use unwrap_or_default() instead of unwrap() here, otherwise the size of the .elf baloons by ~10K.
    // I think this is because the panic!() inside unwrap() has to format a lot of stuff.
    let serialized: Vec<u8, 32> = postcard::to_vec(&response).unwrap_or_default();
    for byte in serialized {
        serial.write_byte(byte);
    }
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
        match Action::try_from_iter::<MAX_ACTION_WIRE_SIZE>(&mut UnoSerial(&mut serial)) {
            postcard::Result::Ok(action) => match action {
                Action::Output(pin_label, write_state) => {
                    pin_dispatcher.output(pin_label, write_state);
                    send_response(&mut serial, Response::Output(pin_label, write_state));
                }
                Action::Input(pin_label) => {
                    let read_state = pin_dispatcher.input(pin_label);
                    send_response(&mut serial, Response::Input(pin_label, read_state));
                }
                Action::List => {
                    for (pin_label, pin) in &pin_dispatcher {
                        send_response(&mut serial, Response::List(*pin_label, pin.name()));
                    }
                }
            },
            postcard::Result::Err(_error) => (),
        }
    }
}
