#![no_std]
#![no_main]

use arduino_hal::{
    hal::port::Dynamic,
    port::{mode::Output, Pin},
};
use embedded_hal::digital::v2::{OutputPin, PinState};
use panic_halt as _;

enum Action {
    Output(PinState),
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

    let mut d2 = pins.d2.into_output().downgrade();
    let mut d3 = pins.d3.into_output().downgrade();
    let mut d4 = pins.d4.into_output().downgrade();
    let mut d5 = pins.d5.into_output().downgrade();
    let mut d6 = pins.d6.into_output().downgrade();
    let mut d7 = pins.d7.into_output().downgrade();
    let mut led = pins.d13.into_output().downgrade();

    loop {
        let mut pin: Option<&mut Pin<Output, Dynamic>> = None;
        let mut action: Option<Action> = None;

        match serial.read_byte() as char {
            '\n' => continue,
            '+' => action = Some(Action::Output(PinState::High)),
            '-' => action = Some(Action::Output(PinState::Low)),
            byte => ufmt::uwrite!(&mut serial, "Invalid action '{}'. ", byte).unwrap(),
        }

        match serial.read_byte() as char {
            '\n' => continue,
            pin_num @ '1'..='7' => {
                pin = Some(match pin_num {
                    '1' => &mut led,
                    '2' => &mut d2,
                    '3' => &mut d3,
                    '4' => &mut d4,
                    '5' => &mut d5,
                    '6' => &mut d6,
                    '7' => &mut d7,
                    _ => unreachable!(),
                })
            }
            byte => ufmt::uwrite!(&mut serial, "Invalid pin '{}'. ", byte).unwrap(),
        }

        while serial.read_byte() as char != '\n' {
            ufmt::uwrite!(&mut serial, "Expecting newline. ").unwrap();
        }
        if let (Some(pin), Some(Action::Output(state))) = (pin, action) {
            pin.set_state(state).unwrap();
            ufmt::uwriteln!(&mut serial, "Done.").unwrap();
        } else {
            ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap();
        }
    }
}
