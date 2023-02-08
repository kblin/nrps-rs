// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use phf::phf_map;

use super::get_value;

pub fn encode(sequence: &String) -> Vec<f64> {
    let capacity = sequence.len() * 3;
    let encodeded: Vec<f64> = Vec::with_capacity(capacity);
    sequence
        .chars()
        .map(|c| encode_one(c))
        .fold(encodeded, |mut acc, mut part| {
            acc.append(&mut part);
            acc
        })
}

pub fn encode_one(c: char) -> Vec<f64> {
    let mut encoded: Vec<f64> = Vec::with_capacity(3);
    encoded.push(get_value(
        &HYDROPHOBICITY_MAP,
        c,
        HYDROPHOBICITY_MEAN,
        HYDROPHOBICITY_STDEV,
        false,
    ));
    encoded.push(get_value(&SIZE_MAP, c, SIZE_MEAN, SIZE_STDEV, false));
    encoded.push(get_value(
        &POLARITY_CHARGE_MAP,
        c,
        POLARITY_CHARGE_MEAN,
        POLARITY_CHARGE_STDEV,
        false,
    ));
    encoded
}

static HYDROPHOBICITY_MAP: phf::Map<char, f64> = phf_map! {
    'A' => 0.07,
    'R' => 2.88,
    'N' => 3.22,
    'D' => 3.64,
    'C' => 0.71,
    'Q' => 2.18,
    'E' => 3.08,
    'G' => 2.23,
    'H' => 2.41,
    'I' => -4.44,
    'L' => -4.19,
    'K' => 2.84,
    'M' => -2.49,
    'F' => -4.92,
    'P' => -1.22,
    'S' => 1.96,
    'T' => 0.92,
    'W' => -4.75,
    'Y' => -1.39,
    'V' => -2.69,
};
const HYDROPHOBICITY_MEAN: f64 = 0.001923076923076976;
const HYDROPHOBICITY_STDEV: f64 = 2.6160275521955336;

static SIZE_MAP: phf::Map<char, f64> = phf_map! {
    'A' => -1.73,
    'R' => 2.52,
    'N' => 1.45,
    'D' => 1.13,
    'C' => -0.97,
    'Q' => 0.53,
    'E' => 0.39,
    'G' => -5.36,
    'H' => 1.74,
    'I' => -1.68,
    'L' => -1.03,
    'K' => 1.41,
    'M' => -0.27,
    'F' => 1.3,
    'P' => 0.88,
    'S' => -1.63,
    'T' => -2.09,
    'W' => 3.65,
    'Y' => 2.32,
    'V' => -2.53,
};
const SIZE_MEAN: f64 = 0.0011538461538461635;
const SIZE_STDEV: f64 = 1.8589595518420015;

static POLARITY_CHARGE_MAP: phf::Map<char, f64> = phf_map! {
    'A' => 0.09,
    'R' => -3.44,
    'N' => 0.84,
    'D' => 2.36,
    'C' => 4.13,
    'Q' => -1.14,
    'E' => -0.07,
    'G' => 0.3,
    'H' => 1.11,
    'I' => -1.03,
    'L' => -0.98,
    'K' => -3.14,
    'M' => -0.41,
    'F' => 0.45,
    'P' => 2.23,
    'S' => 0.57,
    'T' => -1.4,
    'W' => 0.85,
    'Y' => 0.01,
    'V' => -1.29,
};
const POLARITY_CHARGE_MEAN: f64 = 0.0015384615384615096;
const POLARITY_CHARGE_STDEV: f64 = 1.545268112160973;

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    static DATA: phf::Map<char, [f64; 3]> = phf_map! {
        'A' => [0.026023, -0.931249, 0.057247, ],
        'C' => [0.270669, -0.522418, 2.671680, ],
        'D' => [1.390688, 0.607246, 1.526247, ],
        'E' => [1.176623, 0.209174, -0.046295, ],
        'F' => [-1.881449, 0.698695, 0.290216, ],
        'G' => [0.851702, -2.883954, 0.193145, ],
        'H' => [0.920509, 0.935387, 0.717326, ],
        'I' => [-1.697965, -0.904352, -0.667547, ],
        'K' => [1.084880, 0.757868, -2.033005, ],
        'L' => [-1.602400, -0.554694, -0.635190, ],
        'M' => [-0.952560, -0.145863, -0.266322, ],
        'N' => [1.230139, 0.779386, 0.542599, ],
        'P' => [-0.467091, 0.472762, 1.442120, ],
        'Q' => [0.832589, 0.284485, -0.738732, ],
        'R' => [1.100171, 1.354976, -2.227146, ],
        'S' => [0.748492, -0.877455, 0.367872, ],
        'T' => [0.350943, -1.124906, -0.906987, ],
        'V' => [-1.029012, -1.361597, -0.835802, ],
        'W' => [-1.816465, 1.962843, 0.549071, ],
        'X' => [-0.000735, -0.000621, -0.000996, ],
        'Y' => [-0.532075, 1.247389, 0.005476, ],
        '-' => [-0.000735, -0.000621, -0.000996, ],
    };

    #[test]
    fn test_wold_encoder() {
        for (c, expected) in DATA.entries() {
            let query = c.to_string();
            let got = encode(&query);
            for (i, value) in got.iter().enumerate() {
                assert_approx_eq!(value.clone(), expected[i]);
            }
        }
    }
}
