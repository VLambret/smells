pub trait IMetric {
    fn new() -> Self;
    fn analyze() -> u32;
}

pub struct FakeMetric4;

impl IMetric for FakeMetric4 {
    fn new() -> Self {
        FakeMetric4
    }
    fn analyze() -> u32 {
        4
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