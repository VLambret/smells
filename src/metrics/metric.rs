use std::path::Path;

pub trait IMetric {
    fn analyze(&self, file_path: &Path) -> Result<u32, String>;
    fn get_key(&self) -> &'static str;
}
