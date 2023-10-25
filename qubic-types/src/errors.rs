use thiserror::Error;

#[derive(Debug, Error)]
pub enum QubicError {
    #[error("Invalid {ident} lenght (expected {expected}, found {found})")]
    InvalidIdLengthError { ident: &'static str, expected: usize, found: usize },

    #[error("Invalid format of {ident}. Make sure all charcters are upper/lower case")]
    InvalidIdFormatError { ident: &'static str },

    #[error("Elliptic curve error. Decoded point was not found found on the elliptic curve")]
    EllipticCurveError,

    #[error("Public key is not formatted correctly for 128bit access")]
    FormattingError
}