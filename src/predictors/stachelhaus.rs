// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::config::Config;
use crate::errors::NrpsError;

use super::predictions::{ADomain, Prediction, PredictionCategory};

pub fn predict_stachelhaus(config: &Config, domains: &mut Vec<ADomain>) -> Result<(), NrpsError> {
    let signatures = parse_stachelhaus_sigs(config)?;
    for domain in domains.iter_mut() {
        let aa10 = extract_aa10(&domain.aa34)?;
        let mut max_matches: usize = 6; // Don't bother showing hits < 7 matches
        let mut pred_opt: Option<Prediction> = None;

        for sig in signatures.iter() {
            let matches = aa10.len() - hamming_dist(&aa10, &sig.aa10);
            if matches > max_matches {
                max_matches = matches;
                pred_opt = Some(Prediction {
                    name: sig.winner.clone(),
                    score: matches as f64 / aa10.len() as f64,
                })
            }
        }
        if let Some(pred) = pred_opt {
            domain.add(PredictionCategory::Stachelhaus, pred)
        }
    }
    Ok(())
}

#[derive(Debug)]
struct StachelhausSignature {
    pub aa10: String,
    // pub aa34: String,
    // pub all: String,
    pub winner: String,
    // pub ids: String,
}

fn parse_stachelhaus_sigs(config: &Config) -> Result<Vec<StachelhausSignature>, NrpsError> {
    let reader = File::open(&config.stachelhaus_signatures)?;
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
}
