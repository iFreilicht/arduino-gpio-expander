#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod pin_name;
pub use pin_name::PinName;

mod buffered_iterator;
pub use buffered_iterator::BufferedIterator;

use core::fmt::Debug;
use serde::{Deserialize, Serialize};

pub type PinLabel = char;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy)]
pub enum PinState {
    #[default]
    Low,
    High,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Action {
    Output(PinLabel, PinState),
    Input(PinLabel),
    List,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Response {
    Output(PinLabel, PinState),
    Input(PinLabel, PinState),
    List(PinLabel, PinName), // This response is sent once for every pin
    Err,
}

/// Maximum size a serialized [`Action`] can have on the wire, in bytes
const MAX_ACTION_WIRE_SIZE: usize = 32;

pub fn try_action_from_iter<T>(iter: &mut T) -> postcard::Result<Action>
where
    T: Iterator<Item = u8>,
{
    let mut buffer = [0_u8; MAX_ACTION_WIRE_SIZE];
    let buffered_iter = BufferedIterator::from_iter_and_buffer(iter, &mut buffer);
    let mut deserializer = postcard::Deserializer::from_flavor(buffered_iter);
    let t = Action::deserialize(&mut deserializer)?;
    postcard::Result::Ok(t)
}

#[cfg(test)]
mod test {
    use heapless::Vec;

    use super::*;

    #[test]
    fn serialize_rountrip() {
        //! Just a sanity check to ensure all traits are properly implemented
        let action = Action::Output('7', PinState::High);
        let mut buffer = [0_u8; MAX_ACTION_WIRE_SIZE];
        let serialized = postcard::to_slice(&action, &mut buffer).unwrap();
        let deserialized = postcard::from_bytes(&serialized).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn deserialize_from_iter() {
        //! Test our own [`BufferedIterator`] flavor
        let action = Action::Output('a', PinState::Low);
        let serialized: Vec<u8, MAX_ACTION_WIRE_SIZE> = postcard::to_vec(&action).unwrap();
        let deserialized = try_action_from_iter(&mut serialized.into_iter()).unwrap();
        assert_eq!(action, deserialized);
    }
}
