// License: GNU Affero General Public License v3 or later
// A copy of GNU AGPL v3 should have been included in this software package in LICENSE.txt.

use std::io::{self, BufRead, BufReader, Lines, Read};

use crate::encodings::{encode, FeatureEncoding};
use crate::errors::NrpsError;
use crate::predictors::predictions::PredictionCategory;
use crate::svm::kernels::{Kernel, LinearKernel, RBFKernel};
use crate::svm::vectors::{FeatureVector, SupportVector};

#[derive(Debug)]
pub enum KernelType {
    Linear,
    Polynomial,
    RBF,
    Sigmoid,
    Custom,
}

#[derive(Debug)]
pub struct SVMlightModel {
    pub name: String,
    pub category: PredictionCategory,
    pub vectors: Vec<SupportVector>,
    pub bias: f64,
    pub encoding: FeatureEncoding,
    pub kernel_type: KernelType,
    pub kernel: Box<dyn Kernel>,
}

impl SVMlightModel {
    pub fn new(
        name: String,
        category: PredictionCategory,
        vectors: Vec<SupportVector>,
        bias: f64,
        encoding: FeatureEncoding,
        kernel_type: KernelType,
        gamma: f64,
    ) -> Self {
        let kernel: Box<dyn Kernel>;
        match kernel_type {
            KernelType::Linear => kernel = Box::new(LinearKernel {}),
            KernelType::RBF => kernel = Box::new(RBFKernel::new(gamma)),
            _ => unimplemented!(),
        }
        SVMlightModel {
            name,
            category,
            vectors,
            bias,
            encoding,
            kernel_type,
            kernel,
        }
    }

    pub fn predict(&self, vec: &FeatureVector) -> Result<f64, NrpsError> {
        let res: Result<f64, NrpsError> = self.vectors.iter().try_fold(0.0, |sum, svec| {
            Ok(sum + svec.yalpha * self.kernel.compute(svec, vec)?)
        });
        Ok(res? - self.bias)
    }

    pub fn encode(&self, sequence: &String) -> Vec<f64> {
        encode(sequence, &self.encoding)
    }

    pub fn predict_seq(&self, sequence: &String) -> Result<f64, NrpsError> {
        let fvec = FeatureVector::new(self.encode(sequence));
        self.predict(&fvec)
    }

    pub fn from_handle<R>(
        handle: R,
        name: String,
        category: PredictionCategory,
        encoding: FeatureEncoding,
    ) -> Result<Self, NrpsError>
    where
        R: Read,
    {
        let mut line_iter = io::BufReader::new(handle).lines();
        line_iter.next(); // skip

        let kernel_type = match parse_int(&mut line_iter)? {
            0 => KernelType::Linear,
            2 => KernelType::RBF,
            _ => {
                return Err(NrpsError::InvalidFeatureLine(
                    "Failed to match kernel type".to_string(),
                ))
            }
        };

        line_iter.next(); // skip

        let gamma: f64 = parse_float(&mut line_iter)?;

        line_iter.next(); // skip
        line_iter.next(); // skip
        line_iter.next(); // skip

        let dimensions = parse_int(&mut line_iter)?;

        line_iter.next(); // skip
        let num_vecs = parse_int(&mut line_iter)?;

        let bias = parse_float(&mut line_iter)?;

        let mut vectors = Vec::with_capacity(num_vecs);

        while let Some(line_res) = line_iter.next() {
            let svec = SupportVector::from_line(line_res?, dimensions)?;
            vectors.push(svec);
        }

        Ok(SVMlightModel::new(
            name,
            category,
            vectors,
            bias,
            encoding,
            kernel_type,
            gamma,
        ))
    }
}

fn parse_float(line_iter: &mut Lines<BufReader<impl Read>>) -> Result<f64, NrpsError> {
    if let Some(line_result) = line_iter.next() {
        if let Some(raw_value) = line_result?.trim_end().splitn(2, "#").next() {
            return Ok(raw_value.trim().parse::<f64>()?);
        }
    }
    Err(NrpsError::InvalidFeatureLine(
        "Failed to read line".to_string(),
    ))
}

fn parse_int(line_iter: &mut Lines<BufReader<impl Read>>) -> Result<usize, NrpsError> {
    if let Some(line_result) = line_iter.next() {
        if let Some(raw_value) = line_result?.trim_end().splitn(2, "#").next() {
            return Ok(raw_value.trim().parse::<usize>()?);
        }
    }
    Err(NrpsError::InvalidFeatureLine(
        "Failed to read line".to_string(),
    ))
}
