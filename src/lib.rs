// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

pub mod encodings;
pub mod errors;
pub mod predictors;
pub mod svm;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use errors::NrpsError;
use predictors::predictions::{ADomain, PredictionCategory};
use predictors::{load_models, Predictor};

pub fn run(signature_file: PathBuf, model_dir: PathBuf, count: usize) {
    let mut domains = parse_domains(signature_file).unwrap();
    let models = load_models(model_dir).unwrap();
    let predictor = Predictor { models };
    predictor.predict(&mut domains).unwrap();

    let categories = &[
        PredictionCategory::ThreeCluster,
        PredictionCategory::LargeCluster,
        PredictionCategory::SmallCluster,
        PredictionCategory::Single,
        PredictionCategory::LegacyThreeCluster,
        PredictionCategory::LegacyLargeCluster,
        PredictionCategory::LegacySmallCluster,
        PredictionCategory::LegacySingle,
        PredictionCategory::LegacyThreeClusterFungal,
    ];

    let cat_strings: Vec<String> = categories.iter().map(|c| format!("{c:?}")).collect();

    println!("Name\t{}", cat_strings.join("\t"));

    for domain in domains.iter() {
        let mut best_predictions: Vec<String> = Vec::new();
        for cat in categories.iter() {
            let mut best = domain
                .get_best_n(&cat, count)
                .iter()
                .fold("".to_string(), |acc, new| format!("{acc}|{}", new.name))
                .trim_matches('|')
                .to_string();
            if best == "" {
                best = "N/A".to_string();
            }
            best_predictions.push(best)
        }
        println!("{}\t{}", domain.name, best_predictions.join("\t"));
    }
}

pub fn parse_domains(signature_file: PathBuf) -> Result<Vec<ADomain>, NrpsError> {
    let mut domains = Vec::new();

    let handle = File::open(signature_file)?;
    let reader = BufReader::new(handle);

    for line_res in reader.lines() {
        let line = line_res?.trim().to_string();
        if line == "" {
            continue;
        }
        let parts: Vec<&str> = line.split("\t").collect();
        if parts.len() != 2 {
            return Err(NrpsError::SignatureError(line));
        }
        if parts[0].len() != 34 {
            return Err(NrpsError::SignatureError(line));
        }

        domains.push(ADomain::new(parts[1].to_string(), parts[0].to_string()));
    }

    Ok(domains)
}
