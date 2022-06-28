#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod pin_name;
pub use pin_name::PinName;

mod buffered_iterator;
pub use buffered_iterator::BufferedIterator;
pub use buffered_iterator::TryFromIter;

use core::fmt::Debug;
use serde::{Deserialize, Serialize};

pub type PinLabel = char;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy)]
pub enum PinState {
    #[default]
    Low,
    High,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Output(PinLabel, PinState),
    Input(PinLabel),
    List,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Output(PinLabel, PinState),
    Input(PinLabel, PinState),
    List(PinLabel, PinName), // This response is sent once for every pin
    Err,
}

/// Maximum size a serialized [`Action`] can have on the wire, in bytes
pub const MAX_ACTION_WIRE_SIZE: usize = 8;

/// Maximum size a serialized [`Response`] can have on the wire, in bytes
pub const MAX_RESPONSE_WIRE_SIZE: usize = 16;

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
        let deserialized = Action::try_from_iter::<MAX_ACTION_WIRE_SIZE>(&mut serialized.into_iter()).unwrap();
        assert_eq!(action, deserialized);
    }
}
