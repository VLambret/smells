use std::path::Path;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricKind{
    LinesCount,
    SocialComplexity,
    #[cfg(test)]
    FakeMetric,
    #[cfg(test)]
    BrokenMetric
}

impl Serialize for MetricKind{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let metric_name = match self{
            MetricKind::LinesCount => "lines_count",
            MetricKind::SocialComplexity => "social_complexity",
            #[cfg(test)]
            MetricKind::FakeMetric => "fake",
            #[cfg(test)]
            MetricKind::BrokenMetric => "broken"
        };
        serializer.serialize_str(metric_name)
    }
}

pub trait IMetric {
    fn analyze(&self, file_path: &Path) -> Result<u32, String>;
    fn get_key(&self) -> MetricKind;
}