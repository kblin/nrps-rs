// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::env;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use nrps_rs::config::{parse_config, Config};
use nrps_rs::{print_results, run};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Signature file to run predictions on
    signatures: PathBuf,

    /// Number of results to return per category
    #[arg(short, long, default_value_t = 1)]
    count: usize,

    /// Runs the NRPSPredictor2 fungal models
    #[arg(long, default_value_t = false)]
    fungal: bool,

    /// Sets a custom config file
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Overrides the config file settings for the Stachelhaus signature file
    #[arg(short, long, value_name = "FILE")]
    stachelhaus_signatures: Option<PathBuf>,

    /// Overrides the config file settings for the SVM model dir
    #[arg(short, long, value_name = "DIR")]
    model_dir: Option<PathBuf>,
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

    let count: usize;
    if cli.count >= 1 {
        count = cli.count;
    } else {
        eprintln!("Can't use count of {}, using 1 instead.", cli.count);
        count = 1;
    }

    eprintln!("Running on {}", cli.signatures.display());
    eprintln!("Printing the best {} hit(s)", count);
    let config: Config;

    if config_file.exists() {
        eprintln!("Using config from {}", config_file.display());
        config = parse_config(
            File::open(config_file).unwrap(),
            cli.model_dir,
            cli.stachelhaus_signatures,
        )
        .unwrap();
    } else {
        eprintln!("Using default config");
        config = parse_config("".as_bytes(), cli.model_dir, cli.stachelhaus_signatures).unwrap();
    }
    eprintln!("Model dir is {}", &config.model_dir.display());
    eprintln!(
        "Stachelhaus signatures from {}",
        &config.stachelhaus_signatures.display()
    );

    let domains = run(&config, cli.signatures).unwrap();
    print_results(&domains, count, cli.fungal).unwrap();
}

#[cfg(test)]
extern crate assert_approx_eq;
#[cfg(test)]
extern crate rstest;
