use std::convert::Infallible;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

use crate::XmlDeError;

#[derive(Debug, Error)]
pub enum ParseValueError {
    #[error(transparent)]
    Infallible(#[from] Infallible),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

pub trait Value: Sized {
    fn serialize(&self) -> String;
    fn deserialize(val: &str) -> Result<Self, XmlDeError>;
}

impl<E, T: FromStr<Err = E> + ToString> Value for T
where
    ParseValueError: From<E>,
{
    fn serialize(&self) -> String {
        self.to_string()
    }
    fn deserialize(val: &str) -> Result<Self, XmlDeError> {
        val.parse()
            .map_err(ParseValueError::from)
            .map_err(XmlDeError::from)
    }
}
