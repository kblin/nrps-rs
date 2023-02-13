// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::{cmp::min, collections::HashMap};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum PredictionCategory {
    ThreeCluster,
    LargeCluster,
    SmallCluster,
    Single,
    Stachelhaus,
    LegacyThreeCluster,
    LegacyThreeClusterFungal,
    LegacyLargeCluster,
    LegacySmallCluster,
    LegacySingle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prediction {
    pub name: String,
    pub score: f64,
}

#[derive(Debug)]
pub struct PredictionList {
    predictions: Vec<Prediction>,
}

impl PredictionList {
    pub fn new() -> Self {
        let predictions = Vec::with_capacity(80);
        PredictionList { predictions }
    }
    pub fn add(&mut self, prediction: Prediction) {
        self.predictions.push(prediction);
        self.predictions
            .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap())
    }
    pub fn get_best_n(&self, count: usize) -> Vec<Prediction> {
        let mut predictions = Vec::with_capacity(count);
        let slice_end = min(count, self.predictions.len());
        if self.predictions.len() == 0 {
            return predictions;
        }

        predictions.extend_from_slice(&self.predictions[0..slice_end]);
        for pred in self.predictions[slice_end..].iter() {
            if pred.score < predictions[count - 1].score {
                break;
            }
            predictions.push(pred.clone())
        }

        predictions
    }
    pub fn get_best(&self) -> Vec<Prediction> {
        self.get_best_n(1)
    }
    pub fn len(&self) -> usize {
        self.predictions.len()
    }
}

pub struct ADomain {
    pub name: String,
    pub aa34: String,
    predictions: HashMap<PredictionCategory, PredictionList>,
}

impl ADomain {
    pub fn new(name: String, aa34: String) -> Self {
        ADomain {
            name,
            aa34,
            predictions: HashMap::new(),
        }
    }

    pub fn add(&mut self, category: PredictionCategory, prediction: Prediction) {
        match self.predictions.get_mut(&category) {
            Some(existing) => existing.add(prediction),
            None => {
                let mut plist = PredictionList::new();
                plist.add(prediction);
                self.predictions.insert(category, plist);
            }
        }
    }

    pub fn get_best_n(&self, category: &PredictionCategory, count: usize) -> Vec<Prediction> {
        if let Some(results) = self.predictions.get(category) {
            results.get_best_n(count)
        } else {
            Vec::new()
        }
    }

    pub fn get_all(&self, category: &PredictionCategory) -> Vec<Prediction> {
        if let Some(results) = self.predictions.get(category) {
            results.predictions.clone()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::{fixture, rstest};

    #[fixture]
    pub fn data() -> [Prediction; 4] {
        [
            Prediction {
                name: "Ala".to_string(),
                score: 23.0,
            },
            Prediction {
                name: "Leu".to_string(),
                score: 42.0,
            },
            Prediction {
                name: "D-Ala".to_string(),
                score: 17.0,
            },
            Prediction {
                name: "Ile".to_string(),
                score: 42.0,
            },
        ]
    }

    #[rstest]
    fn test_add(data: [Prediction; 4]) {
        let mut pred_list = PredictionList::new();
        pred_list.add(data[0].clone());
        assert_eq!(pred_list.len(), 1);

        pred_list.add(data[1].clone());
        assert_eq!(pred_list.len(), 2);
        assert_eq!(pred_list.predictions[0], data[1]);

        pred_list.add(data[2].clone());
        assert_eq!(pred_list.len(), 3);
        assert_eq!(pred_list.predictions[2], data[2]);

        pred_list.add(data[3].clone());
        assert_eq!(pred_list.len(), 4);
        assert_eq!(pred_list.predictions[1], data[3]);
    }

    #[rstest]
    fn test_get_best(data: [Prediction; 4]) {
        let mut pred_list = PredictionList::new();
        pred_list.add(data[0].clone());
        pred_list.add(data[1].clone());
        pred_list.add(data[2].clone());
        pred_list.add(data[3].clone());

        let expected = Vec::from([data[1].clone(), data[3].clone()]);
        assert_eq!(pred_list.get_best(), expected);
    }
}
