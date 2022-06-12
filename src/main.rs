#![no_std]
#![no_main]

use panic_halt as _;

enum Action {
    OutputHigh,
    OutputLow,
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
                '+' => action = Some(Action::OutputHigh),
                '-' => action = Some(Action::OutputLow),
                _ => ufmt::uwrite!(&mut serial, "Unexpected byte {} ({}). ", byte, byte as char)
                    .unwrap(),
            }
        }
        if let (Some(pin), Some(action)) = (pin, action) {
            match (pin, action) {
                (1, Action::OutputHigh) => led.set_high(),
                (1, Action::OutputLow) => led.set_low(),
                (2, Action::OutputHigh) => d2.set_high(),
                (2, Action::OutputLow) => d2.set_low(),
                (3, Action::OutputHigh) => d3.set_high(),
                (3, Action::OutputLow) => d3.set_low(),
                (4, Action::OutputHigh) => d4.set_high(),
                (4, Action::OutputLow) => d4.set_low(),
                (5, Action::OutputHigh) => d5.set_high(),
                (5, Action::OutputLow) => d5.set_low(),
                (6, Action::OutputHigh) => d6.set_high(),
                (6, Action::OutputLow) => d6.set_low(),
                (7, Action::OutputHigh) => d7.set_high(),
                (7, Action::OutputLow) => d7.set_low(),
                _ => ufmt::uwriteln!(&mut serial, "Invalid pin number {}.", pin).unwrap(),
            }
        } else {
            ufmt::uwriteln!(&mut serial, "Syntax error.").unwrap();
        }

        arduino_hal::delay_ms(1000);
    }
}
