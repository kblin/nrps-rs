// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use super::rausch;
use super::wold;

pub fn encode(sequence: &str) -> Vec<f64> {
    let capacity = sequence.len() * 3;
    let encodeded: Vec<f64> = Vec::with_capacity(capacity);
    sequence
        .chars()
        .map(encode_one)
        .fold(encodeded, |mut acc, mut part| {
            acc.append(&mut part);
            acc
        })
}

pub fn encode_one(c: char) -> Vec<f64> {
    let mut encoded: Vec<f64> = Vec::with_capacity(15);
    encoded.append(&mut rausch::encode_one(c));
    encoded.append(&mut wold::encode_one(c));
    encoded
}
