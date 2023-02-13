// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::env;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use nrps_rs::config::{parse_config, Config};
use nrps_rs::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    signatures: PathBuf,

    #[arg(short, long, default_value_t = 1)]
    count: usize,

    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let mut config_file: PathBuf;

    if let Some(file) = cli.config {
        config_file = file;
    } else {
        config_file = env::current_dir().unwrap();
        config_file.push("nrps.toml");
    }

    eprintln!("Running on {}", cli.signatures.display());
    eprintln!("Printing the best {} hit(s)", cli.count);
    let config: Config;

    if config_file.exists() {
        eprintln!("Using config from {}", config_file.display());
        config = parse_config(File::open(config_file).unwrap()).unwrap();
    } else {
        eprintln!("Using default config");
        config = parse_config("".as_bytes()).unwrap();
    }
    eprintln!("Model dir is {}", &config.model_dir.display());

    run(config, cli.signatures, cli.count).unwrap();
}

#[cfg(test)]
extern crate assert_approx_eq;
#[cfg(test)]
extern crate rstest;
