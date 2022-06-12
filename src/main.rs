#![no_std]
#![no_main]

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

    let mut d2 = pins.d2.into_output();
    let mut d3 = pins.d3.into_output();
    let mut d4 = pins.d4.into_output();
    let mut d5 = pins.d5.into_output();
    let mut d6 = pins.d6.into_output();
    let mut d7 = pins.d7.into_output();
    let mut led = pins.d13.into_output();

    loop {
        let mut pin: Option<u8> = None;
        let mut action: Option<Action> = None;

        loop {
            let byte = serial.read_byte();
            match byte.into() {
                '\n' => break,
                '1'..='7' => pin = Some(byte - 48),
                '+' => action = Some(Action::Output(PinState::High)),
                '-' => action = Some(Action::Output(PinState::Low)),
                _ => ufmt::uwrite!(&mut serial, "Unexpected byte {} ({}). ", byte, byte as char)
                    .unwrap(),
            }
        }
        if let (Some(pin), Some(Action::Output(state))) = (pin, action) {
            match (pin, state) {
                (1, state) => led.set_state(state).unwrap(),
                (2, state) => d2.set_state(state).unwrap(),
                (3, state) => d3.set_state(state).unwrap(),
                (4, state) => d4.set_state(state).unwrap(),
                (5, state) => d5.set_state(state).unwrap(),
                (6, state) => d6.set_state(state).unwrap(),
                (7, state) => d7.set_state(state).unwrap(),
                _ => ufmt::uwriteln!(&mut serial, "Invalid pin number {}.", pin).unwrap(),
            }
        } else {
            ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap();
        }

        arduino_hal::delay_ms(1000);
    }
}
