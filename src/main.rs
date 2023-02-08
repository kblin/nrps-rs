// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.
use std::env;
use std::path::PathBuf;

use clap::Parser;

use nrps_rs::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    signatures: PathBuf,

    #[arg(short, long, value_name = "DIR")]
    models: Option<PathBuf>,

    #[arg(short, long, default_value_t = 1)]
    count: usize,
}

fn main() {
    let cli = Cli::parse();
    let mut model_dir: PathBuf;

    if let Some(dir) = cli.models {
        model_dir = dir;
    } else {
        model_dir = env::current_dir().unwrap();
        model_dir.push("data");
        model_dir.push("models");
    }

    eprintln!("Running on {}", cli.signatures.display());
    eprintln!("Printing the best {} hit(s)", cli.count);
    eprintln!("Using models from {}", model_dir.display());

    run(cli.signatures, model_dir, cli.count);
}

#[cfg(test)]
extern crate assert_approx_eq;
#[cfg(test)]
extern crate rstest;
