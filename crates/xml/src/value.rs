use quick_xml::events::{BytesStart, Event};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use std::{convert::Infallible, io::BufRead};
use thiserror::Error;

use crate::{XmlDeError, XmlDeserialize};

#[derive(Debug, Error)]
pub enum ParseValueError {
    #[error(transparent)]
    Infallible(#[from] Infallible),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

pub trait Value: Sized {
    fn serialize(&self) -> String;
    fn deserialize(val: &str) -> Result<Self, XmlDeError>;
}

macro_rules! impl_value_parse {
    ($t:ty) => {
        impl Value for $t {
            fn serialize(&self) -> String {
                self.to_string()
            }

            fn deserialize(val: &str) -> Result<Self, XmlDeError> {
                val.parse()
                    .map_err(ParseValueError::from)
                    .map_err(XmlDeError::from)
            }
        }
    };
}

impl_value_parse!(String);
impl_value_parse!(i8);
impl_value_parse!(u8);
impl_value_parse!(i16);
impl_value_parse!(u16);
impl_value_parse!(f32);
impl_value_parse!(i32);
impl_value_parse!(u32);
impl_value_parse!(f64);
impl_value_parse!(i64);
impl_value_parse!(u64);
impl_value_parse!(isize);
impl_value_parse!(usize);

impl<T: Value> XmlDeserialize for T {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
        let mut string = String::new();

        if !empty {
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf)? {
                    Event::Text(text) => {
                        if !start.is_empty() {
                            // Content already written
                            return Err(XmlDeError::UnsupportedEvent("todo"));
                        }
                        string = String::from_utf8_lossy(text.as_ref()).to_string();
                    }
                    _ => return Err(XmlDeError::UnsupportedEvent("todo")),
                };
            }
        }

        <Self as Value>::deserialize(&string)
    }
}
