use crate::metrics::metric::MetricScoreType::{Error, Score};
use crate::metrics::metric::{
    AnalysisError, IMetric, IMetricValue, MetricScoreType, MetricValueType,
};
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
    pub line_count: Result<u64, AnalysisError>,
}

impl IMetricValue for LinesCountValue {
    fn get_key(&self) -> &'static str {
        "lines_count"
    }

    fn get_score(&self) -> MetricScoreType {
        match &self.line_count {
            Ok(value) => Score(*value),
            Err(error) => Error(error.clone()),
        }
    }

    fn get_value(&self) -> Result<MetricValueType, AnalysisError> {
        match &self.line_count {
            Ok(line_count_value) => Ok(MetricValueType::Number(*line_count_value)),
            Err(line_count_error) => Err(line_count_error.clone()),
        }
    }

    fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
        let line_count_value: Result<u64, AnalysisError> = {
            match (self.line_count.as_ref(), other.get_value().as_ref()) {
                (Err(_), Err(_)) => Err(String::from("Analysis error")),
                (Err(_), _) => match other.get_value() {
                    Ok(MetricValueType::Number(value)) => Ok(value),
                    _ => Ok(0),
                },
                (_, Err(_)) => match self.get_value() {
                    Ok(MetricValueType::Number(value)) => Ok(value),
                    _ => Ok(0),
                },
                _ => {
                    let self_line_count = self.get_value().unwrap();
                    let other_line_count = other.get_value().unwrap();
                    match self_line_count + other_line_count {
                        MetricValueType::Number(value) => Ok(value),
                        _ => Ok(0),
                    }
                }
            }
        };
        Box::new(LinesCountValue {
            line_count: line_count_value,
        })
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
