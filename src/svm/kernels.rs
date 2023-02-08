// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.
use std::fmt::Debug;

use crate::errors::NrpsError;
use crate::svm::vectors::{FeatureVector, SupportVector, Vector};

pub trait Kernel {
    fn compute(&self, vec1: &SupportVector, vec2: &FeatureVector) -> Result<f64, NrpsError>;
}

impl Debug for dyn Kernel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Kernel")
    }
}

#[derive(Debug)]
pub struct LinearKernel {}

impl Kernel for LinearKernel {
    fn compute(&self, vec1: &SupportVector, vec2: &FeatureVector) -> Result<f64, NrpsError> {
        vec1.similarity(vec2)
    }
}

#[derive(Debug)]
pub struct RBFKernel {
    gamma: f64,
}

impl RBFKernel {
    pub fn new(gamma: f64) -> Self {
        RBFKernel { gamma }
    }
}

impl Kernel for RBFKernel {
    fn compute(&self, vec1: &SupportVector, vec2: &FeatureVector) -> Result<f64, NrpsError> {
        Ok((-self.gamma * vec1.square_dist(vec2)?).exp())
    }
}
