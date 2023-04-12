#[cfg(test)]
mod tests{
    struct FakeMetric();

    impl FakeMetric {
        fn folder(metrics: [FakeMetric]) -> FakeMetric {
            todo!()
        }
    }

    impl FakeMetric {
        fn create_from(file: &str) -> FakeMetric {
            FakeMetric()
        }

        fn get_value(&self) -> u32 {
            return 1;
        }

        fn combine_with(&self, other: FakeMetric) {
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
        let file: &str = "toto";
        let m1: FakeMetric = FakeMetric::create_from(file);
        let m2: FakeMetric = FakeMetric::create_from(file);

        let folder_metric: FakeMetric = FakeMetric::folder([m1, m2]);

        assert!(m1.get_value() == 1);
        assert!(m2.get_value() == 1);
    }

}