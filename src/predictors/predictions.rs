// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::cmp::min;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq)]
pub struct StachPrediction {
    pub name: String,
    pub aa10_score: f64,
    pub aa10_sig: String,
    pub aa34_score: f64,
    pub aa34_sig: String,
}
impl PartialOrd for StachPrediction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Some(aa10_ord) = self.aa10_score.partial_cmp(&other.aa10_score) {
            if let Some(aa34_ord) = self.aa34_score.partial_cmp(&other.aa34_score) {
                return Some(aa10_ord.then(aa34_ord));
            }
        }
        None
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct StachPredictionList {
    predictions: Vec<StachPrediction>,
}

impl StachPredictionList {
    pub fn new() -> Self {
        let predictions = Vec::with_capacity(5);
        StachPredictionList { predictions }
    }

    pub fn add(&mut self, prediction: StachPrediction) {
        self.predictions.push(prediction);
        self.predictions.sort_by(|a, b| a.partial_cmp(&b).unwrap());
        self.predictions.reverse()
    }

    pub fn get_best_n(&self, count: usize) -> Vec<StachPrediction> {
        let mut predictions = Vec::with_capacity(count);
        let slice_end = min(count, self.predictions.len());
        if self.predictions.len() == 0 {
            return predictions;
        }

        predictions.extend_from_slice(&self.predictions[0..slice_end]);
        for pred in self.predictions[slice_end..].iter() {
            if pred.aa10_score < predictions[count - 1].aa10_score {
                break;
            }
            predictions.push(pred.clone())
        }

        predictions
    }

    pub fn get_best(&self) -> Vec<StachPrediction> {
        self.get_best_n(1)
    }

    pub fn len(&self) -> usize {
        self.predictions.len()
    }

    pub fn to_table(&self) -> String {
        let mut substrates: Vec<String> = Vec::with_capacity(self.len());
        let mut aa10_scores: Vec<f64> = Vec::with_capacity(self.len());
        let mut aa34_scores: Vec<f64> = Vec::with_capacity(self.len());

        for pred in self.get_best().iter() {
            substrates.push(pred.name.clone());
            aa10_scores.push(pred.aa10_score);
            aa34_scores.push(pred.aa34_score);
        }

        let substrate_string = substrates.join("|");
        let aa10_string = aa10_scores
            .iter()
            .map(|a| format!("{a:.2}"))
            .fold(String::from(""), |acc, new| format!("{acc}|{new}"))
            .trim_matches('|')
            .to_string();
        let aa34_string = aa34_scores
            .iter()
            .map(|a| format!("{a:.2}"))
            .fold(String::from(""), |acc, new| format!("{acc}|{new}"))
            .trim_matches('|')
            .to_string();

        format!("{substrate_string}\t{aa10_string}\t{aa34_string}")
    }
}

#[derive(Debug, PartialEq)]
pub struct ADomain {
    pub name: String,
    pub aa34: String,
    predictions: HashMap<PredictionCategory, PredictionList>,
    pub stach_predictions: StachPredictionList,
}

impl ADomain {
    pub fn new(name: String, aa34: String) -> Self {
        ADomain {
            name,
            aa34,
            predictions: HashMap::new(),
            stach_predictions: StachPredictionList::new(),
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
