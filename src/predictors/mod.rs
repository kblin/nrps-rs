// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.
pub mod predictions;
pub mod stachelhaus;

use std::fs::File;
use std::path::Path;

use walkdir::WalkDir;

use crate::config::Config;
use crate::errors::NrpsError;
use crate::svm::models::SVMlightModel;
use predictions::{ADomain, Prediction, PredictionCategory};

#[derive(Debug)]
pub struct Predictor {
    pub models: Vec<SVMlightModel>,
}

impl Predictor {
    pub fn predict(&self, domains: &mut [ADomain]) -> Result<(), NrpsError> {
        for model in self.models.iter() {
            for domain in domains.iter_mut() {
                let score = model.predict_seq(&domain.aa34)?;
                if score > 0.0 {
                    let pred = Prediction {
                        name: model.name.to_string(),
                        score,
                    };
                    domain.add(model.category, pred);
                }
            }
        }
        Ok(())
    }
}

pub fn load_models(config: &Config) -> Result<Vec<SVMlightModel>, NrpsError> {
    let mut models = Vec::with_capacity(1000);

    for category_dir_res in WalkDir::new(config.model_dir())
        .min_depth(1)
        .max_depth(1)
        .sort_by_file_name()
    {
        let category_dir = category_dir_res?;
        let category = match category_dir.file_name().to_str().unwrap() {
            "NRPS3_THREE_CLUSTER" => PredictionCategory::ThreeClusterV3,
            "NRPS3_LARGE_CLUSTER" => PredictionCategory::LargeClusterV3,
            "NRPS3_SMALL_CLUSTER" => PredictionCategory::SmallClusterV3,
            "NRPS3_SINGLE_CLUSTER" => PredictionCategory::SingleV3,
            "NRPS2_THREE_CLUSTER" => PredictionCategory::ThreeClusterV2,
            "NRPS2_THREE_CLUSTER_FUNGAL" => PredictionCategory::ThreeClusterFungalV2,
            "NRPS2_LARGE_CLUSTER" => PredictionCategory::LargeClusterV2,
            "NRPS2_SMALL_CLUSTER" => PredictionCategory::SmallClusterV2,
            "NRPS2_SINGLE_CLUSTER" => PredictionCategory::SingleV2,
            "NRPS1_LARGE_CLUSTER" => PredictionCategory::LargeClusterV1,
            "NRPS1_SMALL_CLUSTER" => PredictionCategory::SmallClusterV1,
            _ => continue,
        };

        if !config.categories().contains(&category) {
            continue;
        }

        for model_file_res in WalkDir::new(category_dir.path())
            .min_depth(1)
            .max_depth(1)
            .sort_by_file_name()
        {
            let entry = model_file_res?;
            // macOS AppleDouble sidecars are metadata, not SVM models.
            if entry.file_name().to_string_lossy().starts_with("._") {
                continue;
            }
            let model_file = entry.path().to_path_buf();
            if let Some(ext) = model_file.extension() {
                if ext != "mdl" {
                    continue;
                }
            } else {
                continue;
            }
            let name = extract_name(&model_file);
            let handle = File::open(&model_file)?;
            models.push(SVMlightModel::from_handle(handle, name, category)?);
        }
    }

    Ok(models)
}

fn extract_name(filename: &Path) -> String {
    let square_brackets: &[_] = &['[', ']'];
    filename
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_matches(square_brackets)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_models_ignores_appledouble_sidecars() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("nrps-models-{}-{unique}", std::process::id()));
        let category = root.join("NRPS3_SINGLE_CLUSTER");
        fs::create_dir_all(&category).unwrap();
        let model = "SVM-light\n0\n0\n1\n0\n0\n0\n102\n0\n1\n0\n";
        fs::write(category.join("cluster1.mdl"), model).unwrap();
        fs::write(category.join("._cluster1.mdl"), b"\x00\x05\x16\x07").unwrap();
        let mut config = Config::new();
        config.set_model_dir(root.clone());
        let result = load_models(&config);
        fs::remove_dir_all(&root).unwrap();
        let models = result.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "cluster1");
    }
}
