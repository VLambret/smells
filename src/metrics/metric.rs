use git2::Error as git2Error;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::ops::Add;
use std::path::Path;

/* **************************************************************** */

#[derive(Debug)]
pub enum SmellsError {
    AnalysisError(AnalysisError),
    ResultError(ResultError),
    OptionError(OptionError),
    GitError(String),
}

pub type AnalysisError = String;

impl From<git2Error> for SmellsError {
    fn from(other: git2Error) -> SmellsError {
        SmellsError::GitError(
            String::from("Error while retrieving social complexity git objects: \n")
                + other.message(),
        )
    }
}

#[derive(Debug)]
pub struct ResultError {
    details: String,
}

impl ResultError {
    pub fn new() -> ResultError {
        ResultError {
            details: String::from("Result is Error"),
        }
    }
}

impl fmt::Display for ResultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ResultError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
pub struct OptionError {
    details: String,
}

impl OptionError {
    pub fn new() -> OptionError {
        OptionError {
            details: String::from("Option is None"),
        }
    }
}

impl fmt::Display for OptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for OptionError {
    fn description(&self) -> &str {
        &self.details
    }
}

/* **************************************************************** */

pub trait IMetric: Debug {
    fn analyse(&self, file_path: &Path) -> Option<Box<dyn IMetricValue>>;
}

//TODO: change to a simple type, or will there be other types of MetricScoreType ?
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
