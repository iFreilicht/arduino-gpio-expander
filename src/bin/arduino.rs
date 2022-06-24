#![no_std]
#![no_main]

use arduino_gpio_expander::actions::Action;
use arduino_gpio_expander::{add_pin, pins::PinDispatcher};

use panic_halt as _;

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
        let action: postcard::Result<Action> = Action::from_serial(&mut serial);
        match action {
            postcard::Result::Ok(action) => {
                match action {
                    Action::Output(pin_label, state) => pin_dispatcher.output(pin_label, state),
                    Action::Input(pin_label) => {
                        if pin_dispatcher.input(pin_label) {
                            ufmt::uwrite!(&mut serial, "Pin is high. ").unwrap();
                        } else {
                            ufmt::uwrite!(&mut serial, "Pin is low. ").unwrap();
                        }
                    }
                    Action::List => {
                        for (pin_label, pin) in &pin_dispatcher {
                            ufmt::uwriteln!(&mut serial, "{}: {}", pin_label, pin.name()).unwrap()
                        }
                    }
                }
                ufmt::uwriteln!(&mut serial, "Done.").unwrap();
            }
            postcard::Result::Err(error) => ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap(),
        }
    }
}
