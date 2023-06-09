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

#[derive(Debug)]
struct LinesCountValue {
    line_count: Result<u64, AnalysisError>,
}

impl IMetricValue for LinesCountValue {
    fn get_score(&self) -> (String, MetricResultType) {
        let score = match &self.line_count {
            Ok(value) => Score(*value),
            Err(error) => Error(error.clone()),
        };

        (String::from("lines_count"), score)
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
