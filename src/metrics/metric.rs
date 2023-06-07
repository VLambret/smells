use std::fmt::Debug;
use std::path::Path;

pub trait IMetricAggregatable {
    fn get_score(&self) -> Result<u32, String>;
}

pub trait IMetric: Debug {
    fn analyze(&self, file_path: &Path) -> Box<dyn IMetricAggregatable>;
    fn get_key(&self) -> &'static str;
}
