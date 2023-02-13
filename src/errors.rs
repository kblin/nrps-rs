// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::io;
use std::num;

use thiserror::Error;
use toml;
use walkdir;

#[derive(Error, Debug)]
pub enum NrpsError {
    #[error("Error parsing config")]
    ConfigError(#[from] toml::de::Error),
    #[error("Invalid result count: `{0}`")]
    CountError(usize),
    #[error("Dimension mismatch: `{first}` vs. `{second}`")]
    DimensionMismatch { first: usize, second: usize },
    #[error("Dir error")]
    DirError(#[from] walkdir::Error),
    #[error("Error parsing float")]
    FloatParserError(#[from] num::ParseFloatError),
    #[error("Error parsing int")]
    IntParserError(#[from] num::ParseIntError),
    #[error("Invalid feature line `{0}`")]
    InvalidFeatureLine(String),
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Signature error `{0}`")]
    SignatureError(String),
}
