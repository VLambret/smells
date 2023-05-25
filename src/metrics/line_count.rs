use crate::analysis::{Analysis, MetricsValueType};
use crate::metrics::metric::IMetric;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct LinesCountMetric {}

impl IMetric for LinesCountMetric {
    fn analyze(&self, file_path: &Path) -> Result<u32, String> {
        let mut file = File::open(file_path).unwrap(); // TODO : remove unwrap
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap(); // TODO : remove unwrap
        Ok(content.lines().count() as u32)
    }
    fn get_key(&self) -> String {
        String::from("lines_count")
    }
}

impl LinesCountMetric {
    pub fn new() -> LinesCountMetric {
        LinesCountMetric {}
    }
}

pub fn summary_lines_count_metric(folder_contents: &[Analysis]) -> u32 {
    folder_contents
        .iter()
        .filter_map(get_lines_count_value)
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

fn get_lines_count_value(content: &Analysis) -> Option<u32> {
    content
        .metrics
        .get("lines_count")
        .and_then(|metric_value| match metric_value {
            Some(MetricsValueType::Score(score)) => Some(*score),
            Some(MetricsValueType::Error(_)) => None,
            None => None,
        })
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
