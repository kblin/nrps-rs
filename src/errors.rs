// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::error;
use std::fmt;
use std::io;
use std::num;

use walkdir;

#[derive(Debug)]
pub enum NrpsError {
    DimensionMismatch((usize, usize)),
    DirError(walkdir::Error),
    FloatParserError(num::ParseFloatError),
    IntParserError(num::ParseIntError),
    InvalidFeatureLine(String),
    Io(io::Error),
    SignatureError(String),
}

macro_rules! implement_custom_error_from {
    ($f: ty, $e: expr) => {
        impl From<$f> for NrpsError {
            fn from(f: $f) -> NrpsError {
                $e(f)
            }
        }
    };
}

implement_custom_error_from!(walkdir::Error, NrpsError::DirError);
implement_custom_error_from!(num::ParseFloatError, NrpsError::FloatParserError);
implement_custom_error_from!(num::ParseIntError, NrpsError::IntParserError);
implement_custom_error_from!(io::Error, NrpsError::Io);

impl fmt::Display for NrpsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NrpsError::DimensionMismatch(ref err) => {
                write!(f, "Dimension mismatch: {} vs. {}", err.0, err.1)
            }
            NrpsError::DirError(ref err) => write!(f, "Dir error: {}", err),
            NrpsError::FloatParserError(ref err) => write!(f, "Failed to parse float: {}", err),
            NrpsError::IntParserError(ref err) => write!(f, "Failed to parse int: {}", err),
            NrpsError::InvalidFeatureLine(ref err) => write!(f, "Invalid feature line: {}", err),
            NrpsError::Io(ref err) => write!(f, "IO error: {}", err),
            NrpsError::SignatureError(ref err) => write!(f, "Invalid signature line: {}", err),
        }
    }
}

impl error::Error for NrpsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            NrpsError::DirError(ref err) => Some(err),
            NrpsError::FloatParserError(ref err) => Some(err),
            NrpsError::IntParserError(ref err) => Some(err),
            NrpsError::Io(ref err) => Some(err),
            NrpsError::DimensionMismatch(_)
            | NrpsError::InvalidFeatureLine(_)
            | NrpsError::SignatureError(_) => None,
        }
    }
}
