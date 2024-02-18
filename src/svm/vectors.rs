// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use crate::errors::NrpsError;

pub trait Vector {
    fn values(&self) -> &Vec<f64>;
    fn dim(&self) -> usize {
        self.values().len()
    }
    fn square_dist<T: Vector>(&self, other: &T) -> Result<f64, NrpsError> {
        let temp = element_subtract(self.values(), other.values())?;
        dot(&temp, &temp)
    }

    fn dist<T: Vector>(&self, other: &T) -> Result<f64, NrpsError> {
        Ok(self.square_dist(other)?.sqrt())
    }

    fn similarity<T: Vector>(&self, other: &T) -> Result<f64, NrpsError> {
        dot(self.values(), other.values())
    }
}

#[derive(Debug)]
pub struct FeatureVector {
    values: Vec<f64>,
}

impl FeatureVector {
    pub fn new(values: Vec<f64>) -> FeatureVector {
        FeatureVector { values }
    }
}

impl Vector for FeatureVector {
    fn values(&self) -> &Vec<f64> {
        &self.values
    }
}

#[derive(Debug)]
pub struct SupportVector {
    values: Vec<f64>,
    pub yalpha: f64,
}

impl SupportVector {
    pub fn new(values: Vec<f64>, yalpha: f64) -> Self {
        SupportVector { values, yalpha }
    }
    pub fn from_line(line: String, dimension: usize) -> Result<Self, NrpsError> {
        let mut values = vec![0.0; dimension];
        let parts: Vec<&str> = line.split(char::is_whitespace).collect();
        if parts.len() < 2 {
            return Err(NrpsError::InvalidFeatureLine(line));
        }
        let yalpha = parts[0].parse::<f64>()?;

        for token in parts[1..].iter() {
            if token == &"#" {
                break;
            }
            let value_parts: Vec<&str> = token.splitn(2, ':').collect();
            let idx = value_parts[0].parse::<usize>()? - 1;
            if idx > dimension - 1 {
                return Err(NrpsError::InvalidFeatureLine(line));
            }
            let value = value_parts[1].parse::<f64>()?;
            values[idx] = value;
        }

        Ok(SupportVector { values, yalpha })
    }
}

impl Vector for SupportVector {
    fn values(&self) -> &Vec<f64> {
        &self.values
    }
}

fn dot(a: &[f64], b: &[f64]) -> Result<f64, NrpsError> {
    if a.len() != b.len() {
        return Err(NrpsError::DimensionMismatch {
            first: a.len(),
            second: b.len(),
        });
    }
    Ok(a.iter()
        .zip(b.iter())
        .fold(0.0, |sum, (el_a, el_b)| sum + el_a * el_b))
}

fn element_subtract(a: &[f64], b: &[f64]) -> Result<Vec<f64>, NrpsError> {
    if a.len() != b.len() {
        return Err(NrpsError::DimensionMismatch {
            first: a.len(),
            second: b.len(),
        });
    }
    Ok(a.iter()
        .zip(b.iter())
        .map(|(el_a, el_b)| el_a - el_b)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_dist() {
        let v1 = FeatureVector::new(Vec::<f64>::from([1.0, 0.0, 1.0]));
        let v2 = FeatureVector::new(Vec::<f64>::from([1.0, 2.0, 3.0]));
        assert_eq!(v1.square_dist(&v2).unwrap(), 8.0);
    }

    #[test]
    fn test_dist() {
        let v1 = FeatureVector::new(Vec::<f64>::from([1.0, 0.0, 1.0]));
        let v2 = FeatureVector::new(Vec::<f64>::from([1.0, 2.0, 1.0]));
        assert_eq!(v1.dist(&v2).unwrap(), 2.0);
    }

    #[test]
    fn test_similarity() {
        let v1 = FeatureVector::new(Vec::<f64>::from([1.0, 0.0, 1.0]));
        let v2 = FeatureVector::new(Vec::<f64>::from([1.0, 2.0, 3.0]));
        assert_eq!(v1.similarity(&v2).unwrap(), 4.0);
    }

    #[test]
    fn test_element_subtract() {
        let v1 = FeatureVector::new(Vec::<f64>::from([3.0, 2.0]));
        let v2 = FeatureVector::new(Vec::<f64>::from([1.0, -2.0]));
        let expected = Vec::from([2.0_f64, 4.0]);
        assert_eq!(
            element_subtract(v1.values(), v2.values()).unwrap(),
            expected
        );
    }

    #[test]
    fn test_from_line() {
        let line = String::from("10 1:-1.6023999 3:-0.55470002 5:-0.63520002 # some junk");
        let v1 = SupportVector::from_line(line, 5).unwrap();
        assert_eq!(v1.yalpha, 10.0);
        assert_eq!(v1.values, [-1.6023999, 0., -0.55470002, 0., -0.63520002]);
    }
}
