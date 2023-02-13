// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::convert::From;
use std::env;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;
use toml;

use crate::errors::NrpsError;

#[derive(Debug, Deserialize)]
struct ParsedConfig {
    pub model_dir: Option<String>,
    pub stachelhaus_signatures: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub model_dir: PathBuf,
    pub stachelhaus_signatures: PathBuf,
}

impl From<ParsedConfig> for Config {
    fn from(item: ParsedConfig) -> Self {
        let mut model_dir: PathBuf;
        if let Some(dir_str) = item.model_dir {
            model_dir = PathBuf::from(dir_str);
        } else {
            model_dir = env::current_dir().unwrap();
            model_dir.push("data");
            model_dir.push("models");
        }

        let mut stachelhaus_signatures: PathBuf;
        if let Some(file_name) = item.stachelhaus_signatures {
            stachelhaus_signatures = PathBuf::from(file_name);
        } else {
            stachelhaus_signatures = model_dir.clone();
            stachelhaus_signatures.push("signatures.tsv");
        }

        Config {
            model_dir,
            stachelhaus_signatures,
        }
    }
}

pub fn parse_config<R>(mut reader: R) -> Result<Config, NrpsError>
where
    R: Read,
{
    let mut raw_config = String::new();
    reader.read_to_string(&mut raw_config)?;
    let parsed_config: ParsedConfig = toml::from_str(&raw_config)?;
    Ok(parsed_config.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dir_set() {
        let expected = Config {
            model_dir: PathBuf::from("/foo"),
            stachelhaus_signatures: PathBuf::from("/foo/signatures.tsv"),
        };
        let got = parse_config("model_dir = '/foo'".as_bytes()).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn test_model_dir_default() {
        let mut model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
        let mut stach = model_dir.clone();
        stach.push("signatures.tsv");

        let expected = Config {
            model_dir,
            stachelhaus_signatures: stach,
        };
        let got = parse_config("".as_bytes()).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn test_stach_extra() {
        let mut model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
        let stach = PathBuf::from("/foo/signatures.tsv");

        let expected = Config {
            model_dir,
            stachelhaus_signatures: stach,
        };
        let got =
            parse_config("stachelhaus_signatures = '/foo/signatures.tsv'".as_bytes()).unwrap();
        assert_eq!(expected, got);
    }
}
