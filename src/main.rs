// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::env;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use nrps_rs::config::{parse_config, Cli};
use nrps_rs::{print_results, run_on_file};

fn main() {
    let cli = Cli::parse();
    let mut config_file: PathBuf;

    if let Some(file) = &cli.config {
        config_file = file.clone();
    } else {
        config_file = env::current_dir().unwrap();
        config_file.push("nrps.toml");
    }

    eprintln!("Running on {}", cli.signatures.display());

    let config = if config_file.exists() {
        eprintln!("Using config from {}", config_file.display());
        parse_config(File::open(config_file).unwrap(), &cli).unwrap()
    } else {
        eprintln!("Using default config");
        parse_config("".as_bytes(), &cli).unwrap()
    };

    eprintln!("Printing the best {} hit(s)", &config.count);
    eprintln!("Model dir is {}", &config.model_dir().display());

    if !config.skip_stachelhaus {
        eprintln!(
            "Stachelhaus signatures from {}",
            &config.stachelhaus_signatures().display()
        );
    }

    let domains = run_on_file(&config, cli.signatures).unwrap();
    print_results(&config, &domains).unwrap();
}

#[cfg(test)]
extern crate assert_approx_eq;
#[cfg(test)]
extern crate rstest;
