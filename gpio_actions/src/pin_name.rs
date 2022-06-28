use core::{
    convert::Infallible,
    fmt::{Debug, Write},
    str::FromStr,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::string::String;

const MAX_PIN_NAME_SIZE: usize = 3; // Pins are named things like 13, D66 or A21

#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinName([u8; MAX_PIN_NAME_SIZE]); // We don't use heapless::String because it creates large binaries

impl FromStr for PinName {
    type Err = Infallible;
    fn from_str(string: &str) -> Result<Self, Infallible> {
        if string.len() > MAX_PIN_NAME_SIZE {
            panic!("Pin name too long!")
        }
        let mut name = Self::default();
        for (i, c) in string.bytes().enumerate() {
            name.0[i] = c;
        }
        Ok(name)
    }
}

#[cfg(feature = "std")]
impl From<PinName> for String {
    fn from(pin_name: PinName) -> Self {
        String::from_utf8_lossy(&pin_name.0).into_owned()
    }
}

impl Debug for PinName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in self.0 {
            f.write_char(byte as char)?;
        }
        Ok(())
    }
}
