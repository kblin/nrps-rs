// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

pub mod config;
pub mod encodings;
pub mod errors;
pub mod predictors;
pub mod svm;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use errors::NrpsError;
use predictors::predictions::ADomain;
use predictors::stachelhaus::predict_stachelhaus;
use predictors::{load_models, Predictor};

pub fn run_on_file(
    config: &config::Config,
    signature_file: PathBuf,
) -> Result<Vec<ADomain>, NrpsError> {
    let mut domains = parse_domains(signature_file)?;
    run(config, &mut domains)?;
    Ok(domains)
}

pub fn run(config: &config::Config, domains: &mut [ADomain]) -> Result<(), NrpsError> {
    if !config.skip_stachelhaus {
        predict_stachelhaus(config, domains)?;
    }

    let models = load_models(config)?;
    let predictor = Predictor { models };
    predictor.predict(domains)?;
    Ok(())
}

pub fn run_on_strings(
    config: &config::Config,
    lines: Vec<String>,
) -> Result<Vec<ADomain>, NrpsError> {
    let mut domains = Vec::with_capacity(lines.len());

    for line in lines.iter() {
        domains.push(parse_domain(line.to_string())?);
    }

    run(config, &mut domains)?;

    Ok(domains)
}

pub fn print_results(config: &config::Config, domains: &[ADomain]) -> Result<(), NrpsError> {
    if config.count < 1 {
        return Err(NrpsError::CountError(config.count));
    }

    let categories = config.categories();

    let cat_strings: Vec<String> = categories.iter().map(|c| format!("{c:?}")).collect();

    let mut headers: Vec<String> = Vec::with_capacity(3);

    headers.push("Name\t8A signature\tStachelhaus signature".to_string());
    if !config.skip_stachelhaus && !config.skip_new_stachelhaus_output {
        headers.push(
            [
                "Full Stachelhaus match",
                "AA10 score",
                "AA10 signature matched",
                "AA34 score",
            ]
            .join("\t")
            .to_string(),
        );
    }
    headers.push(cat_strings.join("\t"));
    println!("{}", headers.join("\t"));

    for domain in domains.iter() {
        let mut best_predictions: Vec<String> = Vec::new();
        for cat in categories.iter() {
            let mut best = domain
                .get_best_n(cat, config.count)
                .iter()
                .fold("".to_string(), |acc, new| {
                    format!("{acc}|{}({:.2})", new.name, new.score)
                })
                .trim_matches('|')
                .to_string();
            if best.is_empty() {
                best = "N/A".to_string();
            }
            best_predictions.push(best)
        }
        let mut line: Vec<String> = Vec::with_capacity(5);
        line.push(domain.name.to_string());
        line.push(domain.aa34.to_string());
        line.push(domain.aa10.to_string());
        if !config.skip_stachelhaus && !config.skip_new_stachelhaus_output {
            line.push(domain.stach_predictions.to_table());
        }
        line.push(best_predictions.join("\t"));
        println!("{}", line.join("\t"));
    }

    Ok(())
}

pub fn parse_domains(signature_file: PathBuf) -> Result<Vec<ADomain>, NrpsError> {
    if signature_file == PathBuf::from("-") {
        let reader = BufReader::new(io::stdin());
        return parse_domains_from_reader(reader);
    }

    if !signature_file.exists() {
        let err = format!("'{}' doesn't exist", signature_file.display());
        return Err(NrpsError::SignatureFileError(err));
    }

    let handle = File::open(signature_file)?;
    let reader = BufReader::new(handle);

    parse_domains_from_reader(reader)
}

fn parse_domains_from_reader<R>(reader: R) -> Result<Vec<ADomain>, NrpsError>
where
    R: BufRead,
{
    let mut domains = Vec::new();

    for line_res in reader.lines() {
        let line = line_res?.trim().to_string();
        if line.is_empty() {
            continue;
        }

        domains.push(parse_domain(line)?);
    }

    Ok(domains)
}

pub fn parse_domain(line: String) -> Result<ADomain, NrpsError> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 2 {
        return Err(NrpsError::SignatureError(line));
    }
    if parts[0].len() != 34 {
        return Err(NrpsError::SignatureError(line));
    }

    let name = match parts.len() {
        2 => parts[1].to_string(),
        _ => format!("{}_{}", parts[2], parts[1]),
    };
    Ok(ADomain::new(name, parts[0].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domains() {
        let two_parts = BufReader::new("LDASFDASLFEMYLLTGGDRNMYGPTEATMCATW\tbpsA_A1".as_bytes());
        let three_parts =
            BufReader::new("LEPAFDISLFEVHLLTGGDRHLYGPTEATLCATW\tHpg\tCAC48361.1.A1".as_bytes());
        let too_short = BufReader::new("LDASFDASLFEMYLLTGGDRNMYGPTEATMCATW".as_bytes());

        let expected_two = Vec::from([ADomain::new(
            "bpsA_A1".to_string(),
            "LDASFDASLFEMYLLTGGDRNMYGPTEATMCATW".to_string(),
        )]);

        let expected_three = Vec::from([ADomain::new(
            "CAC48361.1.A1_Hpg".to_string(),
            "LEPAFDISLFEVHLLTGGDRHLYGPTEATLCATW".to_string(),
        )]);

        let got_two = parse_domains_from_reader(two_parts).unwrap();
        assert_eq!(expected_two, got_two);

        let got_three = parse_domains_from_reader(three_parts).unwrap();
        assert_eq!(expected_three, got_three);

        let got_error = parse_domains_from_reader(too_short);
        assert!(got_error.is_err());
    }
}
