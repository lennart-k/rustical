use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlError {
    #[error(transparent)]
    QuickXmlError(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXmlAttrError(#[from] quick_xml::events::attributes::AttrError),
    #[error("Invalid tag [{0}]{1}. Expected [{2}]{3}")]
    InvalidTag(String, String, String, String),
    #[error("Missing field {0}")]
    MissingField(&'static str),
    #[error("End of file, expected closing tags")]
    Eof,
    #[error("Unsupported xml event: {0}")]
    UnsupportedEvent(&'static str),
    #[error("{0}")]
    Other(String),
    #[error("Invalid variant: {0}")]
    InvalidVariant(String),
    #[error("Invalid field name in {0}: {1}")]
    InvalidFieldName(&'static str, String),
    #[error(transparent)]
    InvalidValue(#[from] crate::value::ParseValueError),
}
