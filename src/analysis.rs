use crate::data_sources::file_explorer::{FileExplorer, IFileExplorer};
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::{IMetric, IMetricValue, MetricResultType};
use crate::metrics::social_complexity::SocialComplexityMetric;
use maplit::hashmap;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/* **************************************************************** */

// garder cette structure et pas juste avoir une HashMap<Pathbuf, Metrics> ?
#[derive(Debug)]
struct FileAnalysis {
    file_path: PathBuf,
    metrics: Vec<Box<dyn IMetricValue>>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct TopAnalysis {
    pub file_name: String,
    pub metrics: BTreeMap<&'static str, Option<MetricResultType>>,
    pub folder_content: Option<BTreeMap<String, TopAnalysis>>,
}

/* **************************************************************** */

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    do_internal_analysis(
        &FileExplorer::new(&root),
        &vec![
            Box::new(LinesCountMetric::new()),
            Box::new(SocialComplexityMetric::new()),
        ],
    )
}

/* **************************************************************** */

pub fn do_internal_analysis(
    file_explorer: &dyn IFileExplorer,
    metrics: &Vec<Box<dyn IMetric>>,
) -> TopAnalysis {
    let file_analyses: HashMap<PathBuf, FileAnalysis> =
        analyse_all_files(file_explorer.discover(), metrics);
    let final_analysis: TopAnalysis = build_final_analysis_structure(file_analyses);
    final_analysis
}

fn analyse_all_files(
    files_to_analyse: Vec<PathBuf>,
    metrics: &Vec<Box<dyn IMetric>>,
) -> HashMap<PathBuf, FileAnalysis> {
    files_to_analyse
        .iter()
        .map(|file| {
            let file_analysis = analyse_single_file(file, metrics);
            (file.to_owned(), file_analysis)
        })
        .collect()
}

fn analyse_single_file(current_file: &PathBuf, metrics: &Vec<Box<dyn IMetric>>) -> FileAnalysis {
    let result_file_metrics = get_file_metrics_value(current_file, metrics);
    FileAnalysis {
        file_path: current_file.to_owned(),
        metrics: result_file_metrics,
    }
}

fn get_file_metrics_value(
    current_file: &PathBuf,
    metrics: &[Box<dyn IMetric>],
) -> Vec<Box<dyn IMetricValue>> {
    metrics
        .iter()
        .map(|metric| metric.analyse(current_file))
        .collect()
}

fn build_final_analysis_structure(file_analyses: HashMap<PathBuf, FileAnalysis>) -> TopAnalysis {
    todo!()
}

/* **************************************************************** */

#[cfg(test)]
mod implementation_test {
    use super::*;
    use crate::analysis::unit_tests::FakeMetric;
    use crate::data_sources::file_explorer::FakeFileExplorer;
    use crate::metrics::metric::MetricResultType::Score;

    #[test]
    fn analysis_with_0_file_should_return_empty_hashmap() {
        // Given
        let fake_file_explorer: Box<dyn IFileExplorer> = Box::new(FakeFileExplorer::new(vec![]));

        //when
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &vec![Box::new(FakeMetric::new(2))],
        );

        //then
        assert_eq!(analyses.len(), 0);
    }

    #[test]
    fn analysis_with_1_file_should_return_one_analysis() {
        // Given
        let files_to_analyze = vec![PathBuf::from("root").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));

        // When
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &vec![Box::new(FakeMetric::new(2))],
        );

        // Then
        assert_eq!(
            analyses.values().next().unwrap().file_path,
            PathBuf::from("root").join("file1")
        );
        assert_eq!(
            analyses
                .values()
                .next()
                .unwrap()
                .metrics
                .first()
                .unwrap()
                .get_score(),
            (String::from("fake2"), Score(2))
        );
    }

    #[test]
    fn analysis_with_2_files_should_return_one_analysis() {
        // Given
        let files_to_analyze = vec![
            PathBuf::from("root").join("file1"),
            PathBuf::from("root").join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));

        // When
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &vec![Box::new(FakeMetric::new(2))],
        );

        // Then
        assert_eq!(
            analyses
                .get(&*PathBuf::from("root").join("file1"))
                .unwrap()
                .metrics
                .first()
                .unwrap()
                .get_score(),
            (String::from("fake2"), Score(2))
        );

        assert_eq!(
            analyses
                .get(&*PathBuf::from("root").join("file2"))
                .unwrap()
                .metrics
                .first()
                .unwrap()
                .get_score(),
            (String::from("fake2"), Score(2))
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::metric::MetricResultType;
    use crate::metrics::metric::MetricResultType::{Error, Score};
    use maplit::btreemap;
    use std::fmt::Debug;
    use std::path::Path;

    #[derive(Debug, Default)]
    pub struct FakeMetric {
        metric_key: &'static str,
        value: u64,
    }

    impl FakeMetric {
        pub fn new(value: u64) -> FakeMetric {
            // It's important to note that this approach has some potential risks.
            // Storing a dynamically allocated &'static str like this may lead to memory leaks
            // if not managed properly, as it bypasses Rust's usual memory management guarantees.
            // Ensure that the FakeMetric instances are properly dropped when no longer needed
            // to avoid leaking memory.
            let metric_key = Box::leak(Box::new(format!("fake{}", value))) as &'static str;
            FakeMetric { metric_key, value }
        }
    }

    impl IMetric for FakeMetric {
        fn analyse(&self, _file_path: &Path) -> Box<dyn IMetricValue> {
            Box::new(FakeMetricValue {
                metric_key: self.metric_key,
                value: self.value,
            })
        }
    }

    #[derive(Debug)]
    struct FakeMetricValue {
        metric_key: &'static str,
        value: u64,
    }

    impl IMetricValue for FakeMetricValue {
        fn get_score(&self) -> (String, MetricResultType) {
            (String::from(self.metric_key), Score(self.value))
        }
    }

    #[derive(Debug, Default)]
    pub struct BrokenMetric {
        pub metric_key: &'static str,
    }

    #[derive(Debug, Default)]
    struct BrokenMetricValue {}

    impl IMetric for BrokenMetric {
        fn analyse(&self, _file_path: &Path) -> Box<dyn IMetricValue> {
            Box::<BrokenMetricValue>::default()
        }
    }

    impl BrokenMetric {
        pub fn new() -> BrokenMetric {
            BrokenMetric::default()
        }
    }

    impl IMetricValue for BrokenMetricValue {
        fn get_score(&self) -> (String, MetricResultType) {
            (
                String::from("broken"),
                Error(String::from("Analysis error")),
            )
        }
    }

    fn build_analysis_structure(
        root_name: String,
        metrics: BTreeMap<&'static str, Option<MetricResultType>>,
        content: BTreeMap<String, TopAnalysis>,
    ) -> TopAnalysis {
        TopAnalysis {
            file_name: root_name,
            metrics,
            folder_content: Some(content),
        }
    }

    #[test]
    #[ignore]
    fn analyse_internal_with_empty_root_and_empty_metrics() {
        // Given
        let root = PathBuf::from("root");
        let fake_file_explorer: Box<dyn IFileExplorer> = Box::new(FakeFileExplorer::new(vec![]));

        // When
        let actual_result_analysis = do_internal_analysis(&*fake_file_explorer, &vec![]);

        // Then
        let expected_result_analysis = build_analysis_structure(
            root.to_string_lossy().to_string(),
            BTreeMap::new(),
            BTreeMap::new(),
        );

        assert_eq!(expected_result_analysis, actual_result_analysis);
    }
    #[test]
    #[ignore]
    fn analyse_internal_with_2_files_and_empty_metrics() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(&root).join("file1"),
            PathBuf::from(&root).join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics = vec![];

        // When
        let actual_result_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let first_file_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: BTreeMap::new(),
            folder_content: None,
        };
        let second_file_analysis = TopAnalysis {
            file_name: String::from("file2"),
            metrics: BTreeMap::new(),
            folder_content: None,
        };
        let mut expected_file_analysis = BTreeMap::new();
        expected_file_analysis.insert(first_file_analysis.file_name.clone(), first_file_analysis);
        expected_file_analysis.insert(second_file_analysis.file_name.clone(), second_file_analysis);

        //let expected_file_analysis = vec![first_file_analysis, second_file_analysis];
        let expected_result_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: BTreeMap::new(),
            folder_content: Some(expected_file_analysis),
        };
        assert_eq!(expected_result_analysis, actual_result_analysis);
    }

    #[test]
    #[ignore]
    fn analyse_internal_with_1_file_and_fakemetric4_and_fakemetric10() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> =
            vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake4", Some(Score(4)));
        expected_metrics.insert("fake10", Some(Score(10)));

        let expected_file_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(
            expected_file_analysis.file_name.clone(),
            expected_file_analysis,
        );

        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    #[ignore]
    fn analyse_internal_with_1_file_and_brokenmetric() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        let error_value = Some(MetricResultType::Error("Analysis error".to_string()));
        expected_metrics.insert("broken", error_value);

        let expected_file_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(
            expected_file_analysis.file_name.clone(),
            expected_file_analysis,
        );

        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: expected_metrics.clone(),
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    #[ignore]
    fn analyse_internal_with_one_5_lines_file() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files");
        let files_to_analyze = vec![PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files")
            .join("file5.txt")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("lines_count", Some(Score(5)));

        let expected_file_analysis = TopAnalysis {
            file_name: "file5.txt".to_string(),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(
            expected_file_analysis.file_name.clone(),
            expected_file_analysis,
        );

        let expected_root_analysis = TopAnalysis {
            file_name: String::from("folder_with_multiple_files"),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    // agreggate tests
    #[test]
    #[ignore]
    fn internal_analyse_with_empty_root_and_fakemetric0() {
        // Given
        let files_to_analyze = vec![];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(0))];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let expected_root_analysis = TopAnalysis {
            file_name: String::from("empty_root"),
            metrics: btreemap! {"fake0" => None},
            folder_content: Some(BTreeMap::new()),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    #[ignore]
    fn internal_analyse_with_1_file_and_fakemetric1() {
        // Given
        let root_name = "root_with_1_file";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(Score(1)));

        let expected_file_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(
            expected_file_analysis.file_name.clone(),
            expected_file_analysis,
        );

        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    #[ignore]
    fn analyse_internal_of_a_file_in_a_folder_with_fakemetric1() {
        // Given
        let root_name = "root_with_1_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(root_name).join("folder1").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(Score(1)));

        let expected_file_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(
            expected_file_analysis.file_name.clone(),
            expected_file_analysis,
        );
        let expected_folder1_analysis = TopAnalysis {
            file_name: String::from("folder1"),
            metrics: expected_metrics.clone(),
            folder_content: Some(expected_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.file_name.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_root_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    #[ignore]
    fn analyse_internal_of_2_file_in_1_folder_in_root_with_fakemetric1_should_add_files_scores() {
        // Given
        /*        let fake_file_explorer = FakeFileExplorer('folder3/file1',
        'folder3/file2',
        'file3');*/
        let root_name = "root_with_2_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(root_name).join("folder1").join("file1"),
            PathBuf::from(root_name).join("folder1").join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(Score(1)));

        let expected_file1_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };
        let expected_file2_analysis = TopAnalysis {
            file_name: String::from("file2"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_folder_metrics = BTreeMap::new();
        expected_folder_metrics.insert("fake1", Some(Score(2)));

        let mut expected_folder1_analysis_content = BTreeMap::new();
        expected_folder1_analysis_content.insert(
            expected_file1_analysis.file_name.clone(),
            expected_file1_analysis,
        );
        expected_folder1_analysis_content.insert(
            expected_file2_analysis.file_name.clone(),
            expected_file2_analysis,
        );

        let expected_folder1_analysis = TopAnalysis {
            file_name: String::from("folder1"),
            metrics: expected_folder_metrics.clone(),
            folder_content: Some(expected_folder1_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.file_name.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: expected_folder_metrics,
            folder_content: Some(expected_root_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }
}
