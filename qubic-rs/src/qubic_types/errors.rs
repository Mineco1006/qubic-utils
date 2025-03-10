#[cfg(not(feature = "std"))]
use thiserror_no_std::Error;

#[cfg(feature = "std")]
use thiserror::Error;

use crate::qubic_types::QubicId;

#[derive(Debug, Error)]
pub enum QubicError {
    #[error("Invalid {ident} length (expected {expected}, found {found})")]
    InvalidIdLengthError {
        ident: &'static str,
        expected: usize,
        found: usize,
    },

    #[error("Invalid format of {ident}. Make sure all charcters are upper/lower case")]
    InvalidIdFormatError { ident: &'static str },

    #[error("Elliptic curve error. Decoded point was not found found on the elliptic curve")]
    EllipticCurveError,

    #[error("Public key is not formatted correctly for 128bit access")]
    FormattingError,

    #[error("Found non matching signer (expected: {expected}, found: {found})")]
    WrongSignature { expected: QubicId, found: QubicId },
}

#[derive(Debug, Error)]
pub enum ByteEncodingError {
    #[error("Invalid data length (expected {expected}, found {found})")]
    InvalidDataLength { expected: usize, found: usize },

    #[error("Invalid minimum data length (expected {expected_min}, found {found})")]
    InvalidMinimumDataLength { expected_min: usize, found: usize },
}