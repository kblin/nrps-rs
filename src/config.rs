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

fn set_stach_from_model_dir(model_dir: &PathBuf) -> PathBuf {
    let mut stachelhaus_signatures = model_dir.clone();
    stachelhaus_signatures.push("signatures.tsv");
    stachelhaus_signatures
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

        let stachelhaus_signatures: PathBuf;
        if let Some(file_name) = item.stachelhaus_signatures {
            stachelhaus_signatures = PathBuf::from(file_name);
        } else {
            stachelhaus_signatures = set_stach_from_model_dir(&model_dir);
        }

        Config {
            model_dir,
            stachelhaus_signatures,
        }
    }
}

pub fn parse_config<R>(
    mut reader: R,
    model_dir: Option<PathBuf>,
    stachelhaus_signatures: Option<PathBuf>,
) -> Result<Config, NrpsError>
where
    R: Read,
{
    let mut raw_config = String::new();
    reader.read_to_string(&mut raw_config)?;
    let parsed_config: ParsedConfig = toml::from_str(&raw_config)?;
    let mut config = Config::from(parsed_config);
    if let Some(md) = model_dir {
        config.model_dir = md;
        config.stachelhaus_signatures = set_stach_from_model_dir(&config.model_dir);
    }
    if let Some(stach) = stachelhaus_signatures {
        config.stachelhaus_signatures = stach;
    }
    Ok(config)
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
        let got = parse_config("model_dir = '/foo'".as_bytes(), None, None).unwrap();
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
        let got = parse_config("".as_bytes(), None, None).unwrap();
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
        let got = parse_config(
            "stachelhaus_signatures = '/foo/signatures.tsv'".as_bytes(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn test_override_model_dir() {
        let model_dir = PathBuf::from("/foo");
        let mut stach = model_dir.clone();
        stach.push("signatures.tsv");

        let expected = Config {
            model_dir: model_dir.clone(),
            stachelhaus_signatures: stach,
        };

        let got = parse_config("".as_bytes(), Some(model_dir), None).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn test_override_stach() {
        let model_dir = PathBuf::from("/foo");
        let stach = PathBuf::from("/bar/signatures.tsv");

        let expected = Config {
            model_dir: model_dir.clone(),
            stachelhaus_signatures: stach.clone(),
        };

        let got = parse_config("model_dir = '/foo'".as_bytes(), None, Some(stach)).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn test_override_both() {
        let model_dir = PathBuf::from("/foo");
        let stach = PathBuf::from("/bar/signatures.tsv");

        let expected = Config {
            model_dir: model_dir.clone(),
            stachelhaus_signatures: stach.clone(),
        };

        let got = parse_config(
            "stachelhaus_signatures = '/baz/signatures.tsv'".as_bytes(),
            Some(model_dir),
            Some(stach),
        )
        .unwrap();
        assert_eq!(expected, got);
    }
}
