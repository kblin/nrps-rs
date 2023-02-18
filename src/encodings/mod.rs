// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

pub mod blin;
pub mod rausch;
pub mod wold;

use crate::predictors::predictions::PredictionCategory;

#[derive(Debug)]
pub enum FeatureEncoding {
    Blin,
    Rausch,
    Wold,
}

pub fn encode(
    sequence: &String,
    encoding: &FeatureEncoding,
    category: &PredictionCategory,
) -> Vec<f64> {
    let legacy_categories = &[
        PredictionCategory::LargeClusterV1,
        PredictionCategory::SmallClusterV1,
    ];
    match encoding {
        FeatureEncoding::Blin => blin::encode(sequence),
        FeatureEncoding::Rausch => {
            if legacy_categories.contains(category) {
                rausch::legacy_encode(sequence)
            } else {
                rausch::encode(sequence)
            }
        }
        FeatureEncoding::Wold => wold::encode(sequence),
    }
}

pub fn get_value(map: &phf::Map<char, f64>, c: char, mean: f64, stdev: f64, use_mean: bool) -> f64 {
    if let Some(value) = map.get(&c) {
        return normalise(value.clone(), mean, stdev);
    }
    if use_mean {
        return mean;
    }
    normalise(0.0, mean, stdev)
}

fn normalise(value: f64, mean: f64, stdev: f64) -> f64 {
    (value - mean) / stdev
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use phf::phf_map;

    static TEST_MAP: phf::Map<char, f64> = phf_map! {
        'A' => 0.00,
        'R' => 4.00,
        'K' => 2.00,
    };
    const TEST_MEAN: f64 = 2.0;
    const TEST_STDEV: f64 = 2.0;

    #[test]
    fn test_get_value() {
        assert_approx_eq!(get_value(&TEST_MAP, 'A', TEST_MEAN, TEST_STDEV, true), -1.0);
        assert_approx_eq!(get_value(&TEST_MAP, 'R', TEST_MEAN, TEST_STDEV, true), 1.0);
        assert_approx_eq!(get_value(&TEST_MAP, 'K', TEST_MEAN, TEST_STDEV, true), 0.0);
        assert_approx_eq!(get_value(&TEST_MAP, '-', TEST_MEAN, TEST_STDEV, true), 2.0);
        assert_approx_eq!(
            get_value(&TEST_MAP, '-', TEST_MEAN, TEST_STDEV, false),
            -1.0
        );
    }
}
