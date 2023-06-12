use serde::Serialize;
use std::fmt::Debug;
use std::path::Path;

pub type AnalysisError = String;

pub trait IMetric: Debug {
    fn analyse(&self, file_path: &Path) -> Box<dyn IMetricValue>;
}

// Ou Result<MST, String> ?
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum MetricResultType {
    Score(u64),
    Error(AnalysisError),
}

pub trait IMetricValue: Debug {
    fn get_key(&self) -> &'static str;
    fn get_score(&self) -> MetricResultType;
}
