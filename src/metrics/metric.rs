use serde::{Serialize, Serializer};
use std::fmt::Debug;
use std::ops::Add;
use std::path::Path;

pub type AnalysisError = String;

pub trait IMetric: Debug {
    fn analyse(&self, file_path: &Path) -> Box<dyn IMetricValue>;
}

// TODO: Ou Result<MST, String> ?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricScoreType {
    Score(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricValueType {
    Number(u64),
    Authors(Vec<String>),
}

impl Add for MetricValueType {
    type Output = MetricValueType;

    fn add(self, other: MetricValueType) -> MetricValueType {
        match (self, other) {
            (MetricValueType::Number(n1), MetricValueType::Number(n2)) => {
                MetricValueType::Number(n1 + n2)
            }
            (MetricValueType::Authors(mut authors1), MetricValueType::Authors(authors2)) => {
                authors1.extend(authors2);
                MetricValueType::Authors(authors1)
            }
            (MetricValueType::Number(n), MetricValueType::Authors(mut authors)) => {
                authors.push(n.to_string());
                MetricValueType::Authors(authors)
            }
            (MetricValueType::Authors(mut authors), MetricValueType::Number(n)) => {
                authors.insert(0, n.to_string());
                MetricValueType::Authors(authors)
            }
        }
    }
}

pub trait IMetricValue: Debug + IMetricValueClone {
    fn get_key(&self) -> &'static str;
    fn get_score(&self) -> Result<MetricScoreType, AnalysisError>;
    fn get_value(&self) -> Result<MetricValueType, AnalysisError>;
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
