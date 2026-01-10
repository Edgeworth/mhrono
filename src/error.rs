use std::num::ParseIntError;

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid time components")]
    InvalidTimeComponents,

    #[error("ambiguous or nonexistent local datetime: {0}")]
    InvalidLocalDateTime(String),

    #[error("duration parse error: {0}")]
    DurationParse(String),

    #[error("frequency parse error: {0}")]
    FrequencyParse(String),

    #[error("out of range: {0}")]
    OutOfRange(String),

    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error(transparent)]
    TimeZoneParse(#[from] chrono_tz::ParseError),

    #[error(transparent)]
    DecimalParse(#[from] rust_decimal::Error),

    #[error(transparent)]
    IntParse(#[from] ParseIntError),

    #[error(transparent)]
    StrumParse(#[from] strum::ParseError),

    #[error(transparent)]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    /// Create a custom error from any error type.
    pub fn custom<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::Custom(Box::new(err))
    }
}
