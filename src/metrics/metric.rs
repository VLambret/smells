use std::path::PathBuf;

pub trait IMetric {
    fn analyze(&self, file_path: &PathBuf) -> Result<u32, String>;
    fn get_key(&self) -> String;
}
