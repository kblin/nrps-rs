// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::config::Config;
use crate::errors::NrpsError;

use super::predictions::{
    ADomain, Prediction, PredictionCategory, PredictionList, StachPrediction, StachPredictionList,
};

pub fn predict_stachelhaus(config: &Config, domains: &mut Vec<ADomain>) -> Result<(), NrpsError> {
    let signatures = parse_stachelhaus_sigs(config)?;
    predict(domains, signatures)
}

fn predict(
    domains: &mut Vec<ADomain>,
    signatures: Vec<StachelhausSignature>,
) -> Result<(), NrpsError> {
    for domain in domains.iter_mut() {
        let aa10 = extract_aa10(&domain.aa34)?;
        let mut max_aa10_matches: usize = 6; // Don't bother showing hits < 7 matches
        let mut max_aa34_matches: usize = max_aa10_matches;
        let mut predictions = PredictionList::new();
        let mut stach_predictions = StachPredictionList::new();

        for sig in signatures.iter() {
            let aa10_matches = aa10.len() - hamming_dist(&aa10, &sig.aa10);
            let aa34_matches = domain.aa34.len() - hamming_dist(&domain.aa34, &sig.aa34);
            if aa10_matches > max_aa10_matches {
                max_aa10_matches = aa10_matches;
                predictions.add(Prediction {
                    name: sig.winner.clone(),
                    score: calculate_score(
                        aa10_matches,
                        aa10.len(),
                        aa34_matches,
                        domain.aa34.len(),
                    ),
                });
                stach_predictions.add(StachPrediction {
                    name: sig.winner.clone(),
                    aa10_score: similarity(aa10_matches, aa10.len()),
                    aa10_sig: sig.aa10.clone(),
                    aa34_score: similarity(aa34_matches, sig.aa34.len()),
                    aa34_sig: sig.aa34.clone(),
                })
            } else if aa10_matches == max_aa10_matches && aa34_matches > max_aa34_matches {
                max_aa34_matches = aa34_matches;
                predictions.add(Prediction {
                    name: sig.winner.clone(),
                    score: calculate_score(
                        aa10_matches,
                        aa10.len(),
                        aa34_matches,
                        domain.aa34.len(),
                    ),
                });
                stach_predictions.add(StachPrediction {
                    name: sig.winner.clone(),
                    aa10_score: similarity(aa10_matches, aa10.len()),
                    aa10_sig: sig.aa10.clone(),
                    aa34_score: similarity(aa34_matches, sig.aa34.len()),
                    aa34_sig: sig.aa34.clone(),
                })
            }
        }
        for pred in predictions.get_best().iter() {
            domain.add(PredictionCategory::Stachelhaus, pred.clone());
        }
        domain.stach_predictions = stach_predictions;
    }
    Ok(())
}

fn calculate_score(
    primary_matches: usize,
    primary_len: usize,
    secondary_matches: usize,
    secondary_len: usize,
) -> f64 {
    let primary_score = similarity(primary_matches, primary_len);
    let penalty = 1.0 - similarity(secondary_matches, secondary_len);
    primary_score - (penalty / 10.0)
}

fn similarity(matches: usize, len: usize) -> f64 {
    matches as f64 / len as f64
}

#[derive(Debug)]
struct StachelhausSignature {
    pub aa10: String,
    pub aa34: String,
    // pub all: String,
    pub winner: String,
    // pub ids: String,
}

fn parse_stachelhaus_sigs(config: &Config) -> Result<Vec<StachelhausSignature>, NrpsError> {
    let reader = File::open(&config.stachelhaus_signatures())?;
    parse_sigs_internal(reader)
}

fn parse_sigs_internal<R>(handle: R) -> Result<Vec<StachelhausSignature>, NrpsError>
where
    R: Read,
{
    let mut signatures = Vec::with_capacity(2500);
    let reader = BufReader::new(handle);
    for line_res in reader.lines() {
        let parts: Vec<String> = line_res?
            .trim()
            .split("\t")
            .map(|s| s.to_string())
            .collect();
        if parts.len() != 5 {
            return Err(NrpsError::SignatureError(parts.join("")));
        }
        let sig = StachelhausSignature {
            aa10: parts[0].to_string(),
            aa34: parts[1].to_string(),
            winner: parts[3].to_string(),
        };
        signatures.push(sig);
    }
    Ok(signatures)
}

fn extract_aa10(aa34: &String) -> Result<String, NrpsError> {
    let mut aa10 = String::with_capacity(10);
    for (i, c) in aa34.chars().enumerate() {
        match i {
            5 | 6 | 9 | 12 | 14 | 16 | 21 | 29 | 30 => aa10.push(c),
            _ => continue,
        }
    }
    aa10.push('K');
    if aa10.len() != 10 {
        return Err(NrpsError::SignatureError(aa34.clone()));
    }

    Ok(aa10)
}

fn hamming_dist(a: &String, b: &String) -> usize {
    a.chars().zip(b.chars()).filter(|t| t.0 != t.1).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_extract_aa10() {
        let expected = "DMVICGCAAK".to_string();
        let got = extract_aa10(&"HAKSFDMSVVQCIACMGGETNCYGPTEITAAATF".to_string());
        assert!(got.is_ok());
        assert_eq!(expected, got.unwrap());
    }

    #[test]
    fn test_extract_aa10_error() {
        let got = extract_aa10(&"THISISWAYTOOSHORT".to_string());
        assert!(got.is_err());
    }

    #[test]
    fn test_hamming_dist() {
        let a = String::from("ABCDE");
        let b = String::from("ABCDF");
        let c = String::from("EDCBA");
        assert_eq!(hamming_dist(&a, &a), 0);
        assert_eq!(hamming_dist(&a, &b), 1);
        assert_eq!(hamming_dist(&a, &c), 4);
    }

    #[test]
    fn test_calculate_score() {
        let test_cases: &[((usize, usize, usize, usize), f64)] = &[
            ((10, 10, 10, 10), 1.0),
            ((10, 10, 9, 10), 0.99),
            ((10, 10, 5, 10), 0.95),
        ];
        for case in test_cases.iter() {
            let values = case.0;
            let expected = case.1;
            assert_approx_eq!(
                expected,
                calculate_score(values.0, values.1, values.2, values.3)
            );
        }
    }
}
