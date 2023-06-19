use serde::{Serialize, Serializer};
use std::fmt::Debug;
use std::path::Path;

pub type AnalysisError = String;

pub trait IMetric: Debug {
    fn analyse(&self, file_path: &Path) -> Box<dyn IMetricValue>;
    fn initialized_to_null(&self) -> Box<dyn IMetricValue>;
}

// Ou Result<MST, String> ?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricResultType {
    Score(u64),
    Error(AnalysisError),
}

impl Serialize for MetricResultType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Score(value) => serializer.serialize_u64(*value),
            Self::Error(error) => serializer.serialize_str(error),
        }
    }
}

pub trait IMetricValue: Debug + IMetricValueClone {
    fn get_key(&self) -> &'static str;
    fn get_score(&self) -> Option<MetricResultType>;
    fn get_line_count_for_test(&self) -> Result<u64, AnalysisError>;
    fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue>;
    fn create_clone_with_value_zero(&self) -> Box<dyn IMetricValue>;
}

pub trait IMetricValueClone {
    fn clone_box(&self) -> Box<dyn IMetricValue>;
}

impl<T> IMetricValueClone for T
where
    T: IMetricValue + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn IMetricValue> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn IMetricValue> {
    fn clone(&self) -> Box<dyn IMetricValue> {
        self.clone_box()
    }
}
