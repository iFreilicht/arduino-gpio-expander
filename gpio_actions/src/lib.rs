#![no_std]

use core::fmt::Debug;
use postcard::de_flavors::Flavor;
use serde::{Deserialize, Serialize};

pub type PinLabel = char;

#[derive(Serialize, Deserialize, Debug)]
pub enum PinState {
    High,
    Low,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Output(PinLabel, PinState),
    Input(PinLabel),
    List,
}

/// Maximum size a serialized [`Action`] can have on the wire, in bytes
const MAX_ACTION_WIRE_SIZE: usize = 32;

pub fn try_action_from_iter<T>(iter: &mut T) -> postcard::Result<Action>
where
    T: Iterator<Item = u8>,
{
    let mut buffer = [0u8; MAX_ACTION_WIRE_SIZE];
    let buffered_iter = BufferedIterator {
        iter,
        buffer: &mut buffer,
    };
    let mut deserializer = postcard::Deserializer::from_flavor(buffered_iter);
    let t = Action::deserialize(&mut deserializer)?;
    postcard::Result::Ok(t)
}

struct BufferedIterator<'a, T> {
    iter: &'a mut T,
    buffer: &'a mut [u8],
}

trait OptionToPostcardResult<T> {
    fn into_postcard_result(self) -> postcard::Result<T>;
}

impl OptionToPostcardResult<u8> for Option<u8> {
    fn into_postcard_result(self) -> postcard::Result<u8> {
        match self {
            Some(byte) => postcard::Result::Ok(byte),
            None => postcard::Result::Err(postcard::Error::DeserializeUnexpectedEnd),
        }
    }
}

impl<'de, T> Flavor<'de> for BufferedIterator<'de, T>
where
    T: Iterator<Item = u8>,
{
    type Remainder = ();
    type Source = BufferedIterator<'de, T>;
    fn pop(&mut self) -> postcard::Result<u8> {
        self.iter.next().into_postcard_result()
    }

    fn try_take_n(&mut self, ct: usize) -> postcard::Result<&'de [u8]> {
        let mut end_of_slice = 0;
        for i in 0..ct {
            self.buffer[i] = self.iter.next().into_postcard_result()?;
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
