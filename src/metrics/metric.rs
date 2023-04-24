use std::fs::File;
use std::path::PathBuf;
use crate::metrics::line_count;

pub trait IMetric {
    fn analyze(&self, file_path : &PathBuf) -> Result<u32, String>;
    fn get_key(&self) -> String;
}
pub struct LinesCountMetric {
    metric_key: String,
    metric_value: u32,
}

impl IMetric for LinesCountMetric {
    fn analyze(&self, file_path : &PathBuf) -> Result<u32, String> {
        let mut file = File::open(file_path).unwrap(); // TODO : remove unwrap
        Ok(line_count::compute_lines_count_metric(&mut file).expect("TODO: make metric optional") as u32)
    }
    fn get_key(&self) -> String {
        self.metric_key.clone()
    }
}

impl LinesCountMetric {
    pub fn new() -> LinesCountMetric {
        LinesCountMetric {
            metric_key : "lines_count".to_string(),
            metric_value : 0,
        }
    }
}

pub struct FakeMetric{
    pub(crate) metric_key: String,
    pub(crate) metric_value: u32
}
impl IMetric for FakeMetric {
    fn analyze(&self, file_path : &PathBuf) -> Result<u32, String> {
        Ok(self.metric_value)
    }
    fn get_key(&self) -> String {
        self.metric_key.to_owned()
    }
}
impl FakeMetric {
    pub fn new(metric_value :u32) -> FakeMetric {
        FakeMetric {
            metric_key: format!("fake{}", metric_value),
            metric_value
        }
    }
}

pub struct BrokenMetric{
    pub metric_key: String
}
impl IMetric for BrokenMetric {
    fn analyze(&self, file_path : &PathBuf) -> Result<u32, String> {
        Err(String::from("Analysis error"))
    }
    fn get_key(&self) -> String { self.metric_key.to_owned() }
}
impl BrokenMetric {
    pub fn new() -> BrokenMetric {
        BrokenMetric {
            metric_key: String::from("broken")
        }
    }
}




/*#[cfg(test)]
mod tests{

    struct FakeMetric{
        //measurement_attribute: MeasurementAttribute
        measurement_attribute: u32
    }

    impl FakeMetric {
        fn create_from(file: &str) -> FakeMetric {
            let file_size = file.len() as u32;
            FakeMetric{ measurement_attribute: file_size}
        }

        fn get_value(&self) -> u32 {
            return self.measurement_attribute;
        }

        fn combine_with(&self, other: FakeMetric) {
        }

        fn folder(metrics: Box<[FakeMetric]>) -> FakeMetric {
            let mut folder_metric = FakeMetric{ measurement_attribute: 0};
            for metric in metrics.iter(){
                folder_metric.add(metric);
            }
            folder_metric
        }

        fn add(&mut self, metric: &FakeMetric) {
            self.measurement_attribute += metric.measurement_attribute
        }
    }

    // TODO: File -> Metric
    // TODO: Metric -> Value
    // TODO: Metric -> measured_attributes
    // TODO: measured_attributes + measured_attributes -> measured_attributes
    // TODO: folders: Metric[] -> Metric

    #[test]
    fn fake_metric()
    {
        let m1: FakeMetric = FakeMetric::create_from("toto");
        let m2: FakeMetric = FakeMetric::create_from("tututu");

        assert_eq!(m1.get_value(), 4);
        assert_eq!(m2.get_value(), 6);

        let folder_metric: FakeMetric = FakeMetric::folder(Box::from([m1, m2]));
        assert_eq!(folder_metric.get_value(), 10);
    }

}*/