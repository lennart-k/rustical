#[derive(Debug, thiserror::Error)]

pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Resource already exists and overwrite=false")]
    AlreadyExists,

    #[error("Invalid ics/vcf input: {0}")]
    InvalidData(String),

    #[error(transparent)]
    ParserError(#[from] ical::parser::ParserError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
