use crate::metrics::metric::{IMetric, IMetricAggregatable};
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

#[derive(Debug)]
pub struct LinesCountMetric {}

pub struct LinesCountMetricAggregatable {
    line_count: u64,
}

impl LinesCountMetricAggregatable {
    pub(crate) fn new(line_count: u64) -> LinesCountMetricAggregatable {
        LinesCountMetricAggregatable { line_count }
    }
}

impl IMetricAggregatable for LinesCountMetricAggregatable {
    fn get_score(&self) -> Result<u32, String> {
        Ok(self.line_count as u32)
    }
}

impl IMetric for LinesCountMetric {
    fn analyze(&self, file_path: &Path) -> Box<dyn IMetricAggregatable> {
        let mut file = File::open(file_path).unwrap(); // TODO : remove unwrap
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap(); // TODO : remove unwrap
                                                    //Ok(content.lines().count() as u32)
        Box::new(LinesCountMetricAggregatable::new(
            content.lines().count() as u64
        ))
    }
    fn get_key(&self) -> &'static str {
        "lines_count"
    }
}
impl LinesCountMetric {
    pub fn new() -> LinesCountMetric {
        LinesCountMetric {}
    }
}

impl Add for LinesCountMetricAggregatable {
    type Output = LinesCountMetricAggregatable;

    fn add(self, other: Self) -> Self::Output {
        LinesCountMetricAggregatable {
            line_count: self.line_count + other.line_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::line_count::LinesCountMetricAggregatable;
    use crate::metrics::metric::IMetricAggregatable;
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

    #[test]
    fn add_two_metrics() {
        let m1 = LinesCountMetricAggregatable::new(3);
        let m2 = LinesCountMetricAggregatable::new(7);

        let m3: LinesCountMetricAggregatable = m1 + m2;

        assert_eq!(m3.get_score(), Ok(10));
    }
}
