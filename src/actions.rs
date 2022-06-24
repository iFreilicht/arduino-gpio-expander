use postcard::de_flavors::Flavor;
use serde::{Deserialize, Serialize};

pub type PinLabel = char;

#[derive(Serialize, Deserialize)]
pub enum PinState {
    High,
    Low,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Output(PinLabel, PinState),
    Input(PinLabel),
    List,
}

/// Maximum size a serialized [`Action`] can have on the wire, in bytes
const MAX_ACTION_WIRE_SIZE: usize = 32;

pub trait ReadByte {
    fn read_byte(&mut self) -> u8;
}

impl Action {
    pub fn from_serial<T>(serial: &mut T) -> postcard::Result<Action>
    where
        T: ReadByte,
    {
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

struct BufferedSerial<'a, T> {
    serial: &'a mut T,
    buffer: &'a mut [u8],
}

impl<'de, T> Flavor<'de> for BufferedSerial<'de, T>
where
    T: ReadByte,
{
    type Remainder = ();
    type Source = BufferedSerial<'de, T>;
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
