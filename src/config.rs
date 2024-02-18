// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::convert::From;
use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde::Deserialize;
use toml;

use crate::errors::NrpsError;
use crate::predictors::predictions::PredictionCategory;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Signature file to run predictions on
    pub signatures: PathBuf,

    /// Number of results to return per category
    #[arg(short, long)]
    pub count: Option<usize>,

    /// Runs the NRPSPredictor2 fungal models
    #[arg(short = 'F', long, default_value_t = false)]
    pub fungal: bool,

    /// Sets a custom config file
    #[arg(short = 'C', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Overrides the config file settings for the Stachelhaus signature file
    #[arg(short, long, value_name = "FILE")]
    pub stachelhaus_signatures: Option<PathBuf>,

    /// Overrides the config file settings for the SVM model dir
    #[arg(short, long, value_name = "DIR")]
    pub model_dir: Option<PathBuf>,

    /// Disable v3 models
    #[arg(short = '3', long)]
    pub skip_v3: bool,

    /// Disable v2 models
    #[arg(short = '2', long)]
    pub skip_v2: bool,

    /// Disable v1 models
    #[arg(short = '1', long)]
    pub skip_v1: bool,

    /// Disable Stachelhaus lookups
    #[arg(short = 'S', long)]
    pub skip_stachelhaus: bool,

    /// Disable printing new-style AA34 Stachelhaus results
    #[arg(long)]
    pub skip_new_stachelhaus_output: bool,
}

#[derive(Debug, Deserialize)]
struct ParsedConfig {
    pub model_dir: Option<String>,
    pub stachelhaus_signatures: Option<String>,
    pub count: Option<usize>,
    pub skip_v3: Option<bool>,
    pub skip_v2: Option<bool>,
    pub skip_v1: Option<bool>,
    pub skip_stachelhaus: Option<bool>,
    pub skip_new_stachelhaus_output: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    model_dir: PathBuf,
    stachelhaus_signatures: PathBuf,
    stach_sig_derived: bool,
    pub count: usize,
    pub fungal: bool,
    pub skip_v3: bool,
    pub skip_v2: bool,
    pub skip_v1: bool,
    pub skip_stachelhaus: bool,
    pub skip_new_stachelhaus_output: bool,
}

fn set_stach_from_model_dir(model_dir: &Path) -> PathBuf {
    let mut stachelhaus_signatures = model_dir.to_owned();
    stachelhaus_signatures.push("signatures.tsv");
    stachelhaus_signatures
}

impl Config {
    pub fn new() -> Self {
        let mut model_dir: PathBuf;
        model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
        let stachelhaus_signatures = set_stach_from_model_dir(&model_dir);

        Config {
            model_dir,
            stachelhaus_signatures,
            stach_sig_derived: true,
            count: 1,
            fungal: false,
            skip_v3: false,
            skip_v2: false,
            skip_v1: false,
            skip_stachelhaus: false,
            skip_new_stachelhaus_output: false,
        }
    }

    pub fn model_dir(&self) -> &PathBuf {
        &self.model_dir
    }

    pub fn set_model_dir(&mut self, model_dir: PathBuf) {
        self.model_dir = model_dir;
        if self.stach_sig_derived {
            self.stachelhaus_signatures = set_stach_from_model_dir(&self.model_dir);
        }
    }

    pub fn stachelhaus_signatures(&self) -> &PathBuf {
        &self.stachelhaus_signatures
    }

    pub fn set_stachelhaus_signatures(&mut self, stachelhaus_signatures: PathBuf) {
        self.stach_sig_derived = false;
        self.stachelhaus_signatures = stachelhaus_signatures;
    }

    pub fn categories(&self) -> Vec<PredictionCategory> {
        let mut categories: Vec<PredictionCategory> = Vec::with_capacity(12);
        if !self.skip_v3 {
            categories.extend_from_slice(&[
                PredictionCategory::ThreeClusterV3,
                PredictionCategory::LargeClusterV3,
                PredictionCategory::SmallClusterV3,
                PredictionCategory::SingleV3,
            ]);
        }

        if !self.skip_stachelhaus {
            categories.push(PredictionCategory::Stachelhaus);
        }

        if !self.skip_v2 {
            categories.extend_from_slice(&[
                PredictionCategory::ThreeClusterV2,
                PredictionCategory::LargeClusterV2,
                PredictionCategory::SmallClusterV2,
                PredictionCategory::SingleV2,
            ]);
        }

        if self.fungal && !self.skip_v2 {
            categories.push(PredictionCategory::ThreeClusterFungalV2);
        }

        if !self.skip_v1 {
            categories.extend_from_slice(&[
                PredictionCategory::LargeClusterV1,
                PredictionCategory::SmallClusterV1,
            ]);
        }

        categories
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl From<ParsedConfig> for Config {
    fn from(item: ParsedConfig) -> Self {
        let mut config = Config::new();

        if let Some(dir_str) = item.model_dir {
            config.set_model_dir(PathBuf::from(dir_str));
        }

        if let Some(file_name) = item.stachelhaus_signatures {
            config.set_stachelhaus_signatures(PathBuf::from(file_name));
        }

        if let Some(count) = item.count {
            config.count = count;
        }

        if let Some(skip_v3) = item.skip_v3 {
            config.skip_v3 = skip_v3;
        }

        if let Some(skip_v2) = item.skip_v2 {
            config.skip_v2 = skip_v2;
        }

        if let Some(skip_v1) = item.skip_v1 {
            config.skip_v1 = skip_v1;
        }

        if let Some(skip_stachelhaus) = item.skip_stachelhaus {
            config.skip_stachelhaus = skip_stachelhaus;
        }

        if let Some(skip_new_stach) = item.skip_new_stachelhaus_output {
            config.skip_new_stachelhaus_output = skip_new_stach;
        }

        config
    }
}

pub fn parse_config<R>(mut reader: R, args: &Cli) -> Result<Config, NrpsError>
where
    R: Read,
{
    let mut raw_config = String::new();
    reader.read_to_string(&mut raw_config)?;
    let parsed_config: ParsedConfig = toml::from_str(&raw_config)?;
    let mut config = Config::from(parsed_config);
    if let Some(md) = &args.model_dir {
        config.model_dir = md.clone();
        config.stachelhaus_signatures = set_stach_from_model_dir(&config.model_dir);
    }
    if let Some(stach) = &args.stachelhaus_signatures {
        config.stachelhaus_signatures = stach.clone();
    }
    if let Some(mut count_val) = args.count {
        if count_val < 1 {
            count_val = 1;
        }
        config.count = count_val;
    }

    config.skip_v3 = args.skip_v3;
    config.skip_v2 = args.skip_v2;
    config.skip_v1 = args.skip_v1;
    config.skip_stachelhaus = args.skip_stachelhaus;
    config.skip_new_stachelhaus_output = args.skip_new_stachelhaus_output;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::{fixture, rstest};

    #[fixture]
    fn args() -> Cli {
        Cli {
            signatures: PathBuf::from("foo.sig"),
            count: None,
            fungal: false,
            config: None,
            stachelhaus_signatures: None,
            model_dir: None,
            skip_v3: false,
            skip_v2: false,
            skip_v1: false,
            skip_stachelhaus: false,
            skip_new_stachelhaus_output: false,
        }
    }

    #[rstest]
    fn test_model_dir_set(args: Cli) {
        let mut expected = Config::new();
        expected.set_model_dir(PathBuf::from("/foo"));
        expected.set_stachelhaus_signatures(PathBuf::from("/foo/signatures.tsv"));
        expected.stach_sig_derived = true;
        let got = parse_config("model_dir = '/foo'".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_model_dir_default(args: Cli) {
        let mut model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
        let mut stach = model_dir.clone();
        stach.push("signatures.tsv");

        let mut expected = Config::new();
        expected.set_model_dir(model_dir);
        expected.set_stachelhaus_signatures(stach);
        expected.stach_sig_derived = true;
        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_stach_extra(args: Cli) {
        let mut model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
        let stach = PathBuf::from("/foo/signatures.tsv");

        let mut expected = Config::new();
        expected.set_model_dir(model_dir);
        expected.set_stachelhaus_signatures(stach);
        expected.stach_sig_derived = false;

        let got = parse_config(
            "stachelhaus_signatures = '/foo/signatures.tsv'".as_bytes(),
            &args,
        )
        .unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_override_model_dir(mut args: Cli) {
        let model_dir = PathBuf::from("/foo");
        args.model_dir = Some(model_dir.clone());
        let mut stach = model_dir.clone();
        stach.push("signatures.tsv");

        let mut expected = Config::new();
        expected.set_model_dir(model_dir.clone());
        expected.set_stachelhaus_signatures(stach);
        expected.stach_sig_derived = true;

        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_override_stach(mut args: Cli) {
        let model_dir = PathBuf::from("/foo");
        let stach = PathBuf::from("/bar/signatures.tsv");
        args.stachelhaus_signatures = Some(stach.clone());

        let mut expected = Config::new();
        expected.set_model_dir(model_dir.clone());
        expected.set_stachelhaus_signatures(stach.clone());
        expected.stach_sig_derived = true;

        let got = parse_config("model_dir = '/foo'".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_override_both(mut args: Cli) {
        let model_dir = PathBuf::from("/foo");
        let stach = PathBuf::from("/bar/signatures.tsv");
        args.model_dir = Some(model_dir.clone());
        args.stachelhaus_signatures = Some(stach.clone());

        let mut expected = Config::new();
        expected.set_model_dir(model_dir.clone());
        expected.set_stachelhaus_signatures(stach.clone());
        expected.stach_sig_derived = false;

        let got = parse_config(
            "stachelhaus_signatures = '/baz/signatures.tsv'".as_bytes(),
            &args,
        )
        .unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_skip_v3(mut args: Cli) {
        args.skip_v3 = true;

        let mut expected = Config::new();
        expected.skip_v3 = true;
        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_skip_v2(mut args: Cli) {
        args.skip_v2 = true;

        let mut expected = Config::new();
        expected.skip_v2 = true;
        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_skip_v1(mut args: Cli) {
        args.skip_v1 = true;

        let mut expected = Config::new();
        expected.skip_v1 = true;
        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    fn test_skip_stachelhaus(mut args: Cli) {
        args.skip_stachelhaus = true;

        let mut expected = Config::new();
        expected.skip_stachelhaus = true;
        let got = parse_config("".as_bytes(), &args).unwrap();
        assert_eq!(expected, got);
    }
}
