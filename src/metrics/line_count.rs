use crate::metrics::metric::MetricResultType::{Error, Score};
use crate::metrics::metric::{AnalysisError, IMetric, IMetricValue, MetricResultType};
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Default)]
pub struct LinesCountMetric {}

impl IMetric for LinesCountMetric {
    fn analyse(&self, file_path: &Path) -> Box<dyn IMetricValue> {
        let file = File::open(file_path);
        let mut content = String::new();
        let line_count = match file {
            Ok(mut file) => {
                if file.read_to_string(&mut content).is_err() {
                    Err(AnalysisError::from("Analysis error"))
                } else {
                    Ok(content.lines().count() as u64)
                }
            }
            Err(_) => Err(AnalysisError::from("Analysis error")),
        };
        Box::new(LinesCountValue { line_count })
    }
}

impl LinesCountMetric {
    pub fn new() -> LinesCountMetric {
        LinesCountMetric::default()
    }
}

#[derive(Debug, Clone)]
pub struct LinesCountValue {
    pub(crate) line_count: Result<u64, AnalysisError>,
}

impl IMetricValue for LinesCountValue {
    fn get_key(&self) -> &'static str {
        "lines_count"
    }

    fn get_score(&self) -> MetricResultType {
        match &self.line_count {
            Ok(value) => Score(*value),
            Err(error) => Error(error.clone()),
        }
    }

    fn get_line_count_for_test(&self) -> Result<u64, AnalysisError> {
        self.line_count.clone()
    }

    fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
        // other: Self
        if self.line_count.is_err() && other.get_line_count_for_test().is_err() {
            Box::new(LinesCountValue {
                line_count: Err(String::from("Analysis error")),
            })
        } else if self.line_count.is_err() {
            Box::new(LinesCountValue {
                line_count: other.get_line_count_for_test().clone(),
            })
        } else if other.get_line_count_for_test().is_err() {
            Box::new(LinesCountValue {
                line_count: self.line_count.clone(),
            })
        } else {
            Box::new(LinesCountValue {
                line_count: Ok(self.line_count.as_ref().unwrap()
                    + other.get_line_count_for_test().as_ref().unwrap()),
            })
        }
    }

    fn create_clone_with_value_zero(&self) -> Box<dyn IMetricValue> {
        Box::new(LinesCountValue { line_count: Ok(0) })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest(
        input,
        expected,
        case("", 0),
        case("line1", 1),
        case("line1\nline2", 2),
        case("line1\nline2\nline3", 3),
        case("\n", 1),
        case("\n\n\n", 3)
    )]
    fn test_count_lines(input: &str, expected: u32) {
        let content = input.to_owned();
        let line_count = content.lines().count() as u32;
        assert_eq!(line_count, expected);
    }
}
