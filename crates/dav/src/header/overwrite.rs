use thiserror::Error;

#[derive(Error, Debug)]
#[error("Invalid Overwrite header")]
pub struct InvalidOverwriteHeader;

#[derive(Debug, PartialEq, Default)]
pub enum Overwrite {
    #[default]
    T,
    F,
}

impl Overwrite {
    pub fn is_true(&self) -> bool {
        matches!(self, Self::T)
    }
}

impl TryFrom<&[u8]> for Overwrite {
    type Error = InvalidOverwriteHeader;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"T" => Ok(Overwrite::T),
            b"F" => Ok(Overwrite::F),
            _ => Err(InvalidOverwriteHeader),
        }
    }
}
