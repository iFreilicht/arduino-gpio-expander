use super::pins::{PinLabel, PinState};
use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};

use postcard::de_flavors::Flavor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Action {
    Output(PinLabel, PinState),
    Input(PinLabel),
    List,
}

/// Maximum size a serialized [`Action`] can have on the wire, in bytes
const MAX_ACTION_WIRE_SIZE: usize = 32;

impl Action {
    pub fn from_serial(serial: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> postcard::Result<Action> {
        let mut buffer = [0u8; MAX_ACTION_WIRE_SIZE];
        let buffered_serial = BufferedSerial {
            serial,
            buffer: &mut buffer,
        };
        let mut deserializer = postcard::Deserializer::from_flavor(buffered_serial);
        let t = Action::deserialize(&mut deserializer)?;
        postcard::Result::Ok(t)
    }
}

struct BufferedSerial<'a> {
    serial: &'a mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    buffer: &'a mut [u8],
}

impl<'de> Flavor<'de> for BufferedSerial<'de> {
    type Remainder = ();
    type Source = BufferedSerial<'de>;
    fn pop(&mut self) -> postcard::Result<u8> {
        postcard::Result::Ok(self.serial.read_byte())
    }

    fn try_take_n(&mut self, ct: usize) -> postcard::Result<&'de [u8]> {
        let mut end_of_slice = 0;
        for i in 0..ct {
            self.buffer[i] = self.serial.read_byte();
            end_of_slice += 1;
        }
        // Split the buffer so the result can use the bytes we just put into the buffer. This is necessary because
        // the 'de lifetime requires that these bytes are never reused during the whole deserialization process
        let slice = core::mem::take(&mut self.buffer);
        let (head, tail) = slice.split_at_mut(end_of_slice + 1);
        self.buffer = tail;
        postcard::Result::Ok(head)
    }

    fn finalize(self) -> postcard::Result<Self::Remainder> {
        postcard::Result::Ok(())
    }
}
