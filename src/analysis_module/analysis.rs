use crate::data_sources::file_explorer::IFileExplorer;
use crate::metrics::metric::{AnalysisError, IMetric, IMetricValue, MetricScoreType};
use maplit::btreemap;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/* **************************************************************** */

#[derive(Debug)]
struct FileAnalysis {
    file_path: PathBuf,
    metrics: Vec<Box<dyn IMetricValue>>,
}

#[derive(Debug, PartialEq)]
pub struct TopAnalysis {
    pub file_name: String,
    pub metrics: BTreeMap<&'static str, Result<MetricScoreType, AnalysisError>>,
    pub folder_content: Option<BTreeMap<String, TopAnalysis>>,
}

#[derive(Debug, Clone)]
pub struct HierarchicalAnalysis {
    pub file_name: String,
    pub metrics: Vec<Box<dyn IMetricValue>>,
    pub folder_content: Option<BTreeMap<String, HierarchicalAnalysis>>,
}

impl HierarchicalAnalysis {
    fn new(file_analysis: &FileAnalysis) -> HierarchicalAnalysis {
        let top_parent = get_top_parent(&file_analysis.file_path);
        if let Some(some_top_parent) = top_parent {
            HierarchicalAnalysis {
                file_name: some_top_parent.to_string_lossy().to_string(),
                metrics: file_analysis.metrics.clone(),
                folder_content: build_folder_content_one_level_below(file_analysis),
            }
        } else {
            HierarchicalAnalysis {
                file_name: file_analysis.file_path.to_string_lossy().to_string(),
                metrics: file_analysis.metrics.clone(),
                folder_content: None,
            }
        }
    }
}

fn build_folder_content_one_level_below(
    file_analysis: &FileAnalysis,
) -> Option<BTreeMap<String, HierarchicalAnalysis>> {
    if let Some(file_path_without_top_parent) = remove_top_parent(&file_analysis.file_path) {
        let one_level_below_file_analysis = FileAnalysis {
            file_path: file_path_without_top_parent.clone(),
            metrics: file_analysis.metrics.clone(),
        };
        if let Some(current_directory) = get_top_parent(&file_path_without_top_parent) {
            Some(
                btreemap! {current_directory.to_string_lossy().to_string() => HierarchicalAnalysis::new(&one_level_below_file_analysis)},
            )
        } else {
            file_analysis.file_path.file_name()
            .map(|file_name| btreemap! {file_name.to_string_lossy().to_string() => HierarchicalAnalysis::new(&one_level_below_file_analysis)})
        }
    } else {
        None
    }
}

fn remove_top_parent(current_file: &Path) -> Option<PathBuf> {
    if let Some(top_parent) = get_top_parent(current_file) {
        if let Ok(file_without_top_parent) = current_file.strip_prefix(top_parent) {
            Some(PathBuf::from(file_without_top_parent))
        } else {
            None
        }
    } else {
        None
    }
}

fn get_top_parent(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .take_while(|&parent| parent != Path::new(""))
        .last()
        .map(PathBuf::from)
}

/* **************************************************************** */

pub fn do_internal_analysis(
    root: &Path,
    file_explorer: &dyn IFileExplorer,
    metrics: &[Box<dyn IMetric>],
) -> TopAnalysis {
    let root_analysis = HierarchicalAnalysis {
        file_name: root
            .file_name()
            .unwrap_or(PathBuf::from("").as_ref())
            .to_string_lossy()
            .to_string(),
        metrics: vec![],
        folder_content: Some(btreemap! {}),
    };

    let files_to_analyse = file_explorer.discover();

    let file_analyses = &analyse_all_files(files_to_analyse, metrics);

    let updated_root_analysis = build_hierarchical_analysis_structure(
        root_analysis,
        &keep_only_last_root_directory_in_analyses_file_names(file_analyses, root.to_path_buf()),
    );
    build_top_analysis_structure(updated_root_analysis)
}

fn keep_only_last_root_directory_in_analyses_file_names(
    file_analyses: &[FileAnalysis],
    root: PathBuf,
) -> Vec<FileAnalysis> {
    file_analyses
        .iter()
        .filter_map(|file_analysis| {
            let file_path = file_analysis.file_path.to_string_lossy().to_string();
            file_path
                .strip_prefix(&root.to_string_lossy().to_string())
                .map(|file_path_without_root| {
                    let root_filename = root
                        .file_name()
                        .unwrap_or("".as_ref())
                        .to_string_lossy()
                        .to_string();
                    let file_name_with_root_file_name =
                        PathBuf::from(root_filename + file_path_without_root);
                    FileAnalysis {
                        file_path: file_name_with_root_file_name,
                        metrics: file_analysis.metrics.clone(),
                    }
                })
        })
        .collect()
}

fn build_hierarchical_analysis_structure(
    root_analysis: HierarchicalAnalysis,
    file_analyses: &[FileAnalysis],
) -> HierarchicalAnalysis {
    let mut updated_root_analysis = root_analysis;
    for (_ha_counter, current_file_analysis) in file_analyses.iter().enumerate() {
        let current_file_top_analysis = HierarchicalAnalysis::new(current_file_analysis);
        updated_root_analysis =
            combine_hierarchical_analysis(updated_root_analysis, current_file_top_analysis);
    }

    updated_root_analysis
}

fn build_top_analysis_structure(hierarchical_analysis: HierarchicalAnalysis) -> TopAnalysis {
    let metrics: BTreeMap<_, _> = hierarchical_analysis
        .metrics
        .iter()
        .map(|metric| (metric.get_key(), metric.get_score()))
        .collect();

    let folder_content: Option<BTreeMap<String, TopAnalysis>> =
        hierarchical_analysis.folder_content.map(|content_entries| {
            content_entries
                .into_iter()
                .map(|content_entry| {
                    let (analysis_key, analysis) = content_entry;
                    (analysis_key, build_top_analysis_structure(analysis))
                })
                .collect()
        });

    TopAnalysis {
        file_name: hierarchical_analysis.file_name,
        metrics,
        folder_content,
    }
}

fn analyse_all_files(
    files_to_analyse: Vec<PathBuf>,
    metrics: &[Box<dyn IMetric>],
) -> Vec<FileAnalysis> {
    let file_analyses = files_to_analyse
        .iter()
        .map(|file| analyse_single_file(file, metrics))
        .collect();
    file_analyses
}

fn analyse_single_file(current_file: &PathBuf, metrics: &[Box<dyn IMetric>]) -> FileAnalysis {
    let result_file_metrics = get_file_metrics_value(current_file, metrics);
    FileAnalysis {
        file_path: current_file.to_owned(),
        metrics: result_file_metrics,
    }
}

fn get_file_metrics_value(
    current_file: &Path,
    metrics: &[Box<dyn IMetric>],
) -> Vec<Box<dyn IMetricValue>> {
    metrics
        .iter()
        .filter_map(|metric| metric.analyse(current_file))
        .collect::<Vec<Box<dyn IMetricValue>>>()
}

fn combine_folder_content(
    root_content_entries: Option<BTreeMap<String, HierarchicalAnalysis>>,
    other_content_entries: Option<BTreeMap<String, HierarchicalAnalysis>>,
) -> Option<BTreeMap<String, HierarchicalAnalysis>> {
    let mut updated_content: HashMap<String, HierarchicalAnalysis> = HashMap::new();

    if let Some(root_content_entries) = root_content_entries.to_owned() {
        for (root_content_entry_key, root_content_entry_analysis) in root_content_entries {
            if let Some(other_analysis) = other_content_entries
                .as_ref()
                .and_then(|entries| entries.get(&root_content_entry_key))
            {
                let updated_current_analysis = combine_hierarchical_analysis(
                    root_content_entry_analysis,
                    other_analysis.clone(),
                );
                updated_content.insert(root_content_entry_key, updated_current_analysis);
            } else {
                updated_content.insert(root_content_entry_key, root_content_entry_analysis);
            }
        }
    }

    if let Some(other_content_entries) = other_content_entries {
        for (other_content_entry_key, other_content_entry_analysis) in other_content_entries {
            if let Some(root_content_entries) = root_content_entries.as_ref() {
                if !root_content_entries.contains_key(&other_content_entry_key) {
                    updated_content.insert(other_content_entry_key, other_content_entry_analysis);
                }
            }
        }
    }

    Some(updated_content.into_iter().collect())
}

fn combine_hierarchical_analysis(
    root_analysis: HierarchicalAnalysis,
    other_analysis: HierarchicalAnalysis,
) -> HierarchicalAnalysis {
    HierarchicalAnalysis {
        file_name: combine_filenames(root_analysis.file_name, other_analysis.file_name),
        metrics: combine_metrics(root_analysis.metrics, other_analysis.metrics),
        folder_content: combine_folder_content(
            root_analysis.folder_content,
            other_analysis.folder_content,
        ),
    }
}

fn combine_filenames(current_analysis_name: String, _other: String) -> String {
    current_analysis_name
}

fn combine_metrics(
    current_metrics: Vec<Box<dyn IMetricValue>>,
    other_metrics: Vec<Box<dyn IMetricValue>>,
) -> Vec<Box<dyn IMetricValue>> {
    let metrics_from_current: Vec<_> = current_metrics
        .iter()
        .filter_map(|current_metric| {
            other_metrics
                .iter()
                .find(|other_metric| current_metric.get_key() == other_metric.get_key())
                .map(|other_metric| current_metric.aggregate(other_metric.clone()))
                .or_else(|| Some(current_metric.clone()))
        })
        .collect();

    let metrics_from_other: Vec<_> = other_metrics
        .iter()
        .filter(|other_metric| {
            !current_metrics
                .iter()
                .any(|current_metric| current_metric.get_key() == other_metric.get_key())
        })
        .cloned()
        .collect();

    metrics_from_current
        .into_iter()
        .chain(metrics_from_other.into_iter())
        .collect()
}

/* **************************************************************** */
#[cfg(test)]
mod analyse1_test {
    use super::*;
    use crate::metrics::line_count::LinesCountValue;

    #[test]
    fn build_hier_analysis_from_file_analysis_test() {
        // Given
        let first_file_analysis = FileAnalysis {
            file_path: PathBuf::from("root")
                .join("dir1")
                .join("dir2")
                .join("file1"),
            metrics: vec![],
        };
        // When
        let first_file_hierarchical_analysis = HierarchicalAnalysis::new(&first_file_analysis);
        println!("{:?}", first_file_hierarchical_analysis);
    }

    #[test]
    fn combine_hierarchical_analysis_test() {
        let first_analysis = HierarchicalAnalysis::new(&FileAnalysis {
            file_path: PathBuf::from("root")
                .join("dir1")
                .join("dir2")
                .join("dir3")
                .join("file1"),
            metrics: vec![Box::new(LinesCountValue { line_count: Ok(5) })],
        });
        let second_analysis = HierarchicalAnalysis::new(&FileAnalysis {
            file_path: PathBuf::from("root")
                .join("dir1")
                .join("dir2")
                .join("dir3b")
                .join("file2"),
            metrics: vec![Box::new(LinesCountValue { line_count: Ok(3) })],
        });
        println!(
            "{:?}",
            combine_hierarchical_analysis(first_analysis, second_analysis)
        );
    }
}

#[cfg(test)]
mod analyse_all_files_test {
    use super::*;
    use crate::analysis_module::analysis::internal_analysis_unit_tests::{
        BrokenMetric, FakeMetric,
    };
    use crate::data_sources::file_explorer::FakeFileExplorer;
    use crate::metrics::metric::MetricScoreType::Score;

    #[test]
    fn analysis_with_0_file_should_return_empty_hashmap() {
        // Given
        let fake_file_explorer: Box<dyn IFileExplorer> = Box::new(FakeFileExplorer::_new(vec![]));

        //when
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &[Box::new(FakeMetric::new(2))],
        );

        //then
        assert_eq!(analyses.len(), 0);
    }

    #[test]
    fn analysis_with_1_file_and_brokenmetric_should_return_an_error() {
        // Given
        let files_to_analyze = vec![PathBuf::from("root").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));

        // When
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &[Box::new(BrokenMetric::new())],
        );

        // Then
        assert_eq!(
            analyses.get(0).unwrap().file_path,
            PathBuf::from("root").join("file1")
        );
        assert_eq!(
            analyses
                .get(0)
                .unwrap()
                .metrics
                .first()
                .unwrap()
                .get_score(),
            Err(String::from("Analysis error"))
        );
    }

    #[test]
    fn analysis_with_1_file_should_return_one_analysis() {
        // Given
        let files_to_analyze = vec![PathBuf::from("root").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));

        // When
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &[Box::new(FakeMetric::new(2))],
        );

        // Then
        assert_eq!(
            analyses.get(0).unwrap().file_path,
            PathBuf::from("root").join("file1")
        );
        assert_eq!(
            analyses
                .get(0)
                .unwrap()
                .metrics
                .first()
                .unwrap()
                .get_score(),
            Ok(Score(2))
        );
    }

    #[test]
    fn analysis_with_2_files_should_return_two_analysis() {
        // Given
        let files_to_analyze = vec![
            PathBuf::from("root").join("file1"),
            PathBuf::from("root").join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));

        // When
        let analyses = analyse_all_files(
            fake_file_explorer.discover(),
            &[Box::new(FakeMetric::new(2))],
        );

        // Then
        assert_eq!(analyses.len(), 2);
    }
}

#[cfg(test)]
mod internal_analysis_unit_tests {
    use super::*;
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::metric::MetricScoreType::Score;
    use crate::metrics::metric::{AnalysisError, MetricScoreType, MetricValueType};
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
        fn analyse(&self, _file_path: &Path) -> Option<Box<dyn IMetricValue>> {
            Some(Box::new(FakeMetricValue {
                metric_key: self.metric_key,
                value: self.value,
            }))
        }
    }

    #[derive(Debug, Clone)]
    struct FakeMetricValue {
        metric_key: &'static str,
        value: u64,
    }

    impl IMetricValue for FakeMetricValue {
        fn get_key(&self) -> &'static str {
            self.metric_key
        }

        fn get_score(&self) -> Result<MetricScoreType, AnalysisError> {
            Ok(Score(self.value))
        }

        fn get_value(&self) -> Result<MetricValueType, AnalysisError> {
            Ok(MetricValueType::Number(self.value.to_owned()))
        }

        fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
            let line_count_value = other.get_value().clone();
            let line_count = match line_count_value {
                Ok(MetricValueType::Number(count)) => count,
                _ => 0,
            };
            Box::new(FakeMetricValue {
                metric_key: self.metric_key,
                value: self.value + line_count,
            })
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct BrokenMetric {
        pub metric_key: &'static str,
    }

    #[derive(Debug, Default, Clone)]
    struct BrokenMetricValue {}

    impl IMetric for BrokenMetric {
        fn analyse(&self, _file_path: &Path) -> Option<Box<dyn IMetricValue>> {
            Some(Box::<BrokenMetricValue>::default())
        }
    }

    impl BrokenMetric {
        pub fn new() -> BrokenMetric {
            BrokenMetric::default()
        }
    }

    impl IMetricValue for BrokenMetricValue {
        fn get_key(&self) -> &'static str {
            "broken"
        }

        fn get_score(&self) -> Result<MetricScoreType, AnalysisError> {
            Err(String::from("Analysis error"))
        }

        fn get_value(&self) -> Result<MetricValueType, AnalysisError> {
            Err(String::from("Analysis error"))
        }

        fn aggregate(&self, _other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
            Box::new(BrokenMetricValue {})
        }
    }

    fn build_analysis_structure(
        root_name: String,
        metrics: BTreeMap<&'static str, Result<MetricScoreType, AnalysisError>>,
        content: BTreeMap<String, TopAnalysis>,
    ) -> TopAnalysis {
        TopAnalysis {
            file_name: root_name,
            metrics,
            folder_content: Some(content),
        }
    }

    #[test]
    fn analyse_internal_with_empty_root_and_empty_metrics() {
        // Given
        let root = PathBuf::from("root");
        let fake_file_explorer: Box<dyn IFileExplorer> = Box::new(FakeFileExplorer::_new(vec![]));

        // When
        let actual_result_analysis = do_internal_analysis(&root, &*fake_file_explorer, &[]);

        // Then
        let expected_result_analysis = build_analysis_structure(
            root.to_string_lossy().to_string(),
            btreemap! {},
            btreemap! {},
        );

        assert_eq!(expected_result_analysis, actual_result_analysis);
    }
    #[test]
    fn analyse_internal_with_2_files_and_empty_metrics() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(&root).join("file1"),
            PathBuf::from(&root).join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics = vec![];

        // When
        let actual_result_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

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
    fn analyse_internal_with_1_file_and_fakemetric4_and_fakemetric10() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> =
            vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake4", Ok(Score(4)));
        expected_metrics.insert("fake10", Ok(Score(10)));

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
    fn analyse_internal_with_1_file_and_brokenmetric() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        let error_value = Err("Analysis error".to_string());
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

    // agreggate tests
    #[test]
    fn internal_analyse_with_empty_root_and_fakemetric0() {
        // Given
        let files_to_analyze = vec![];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(0))];

        // When
        let actual_root_analysis =
            do_internal_analysis(&PathBuf::from("empty_root"), &*fake_file_explorer, &metrics);

        // Then
        let expected_root_analysis = TopAnalysis {
            file_name: String::from("empty_root"),
            metrics: btreemap! {},
            folder_content: Some(BTreeMap::new()),
        };

        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    fn internal_analyse_with_1_file_and_fakemetric1() {
        // Given
        let root_name = "root_with_1_file";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Ok(Score(1)));

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
    fn analyse_internal_of_a_file_in_a_folder_with_fakemetric1() {
        // Given
        let root_name = "root_with_1_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(root_name).join("folder1").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Ok(Score(1)));

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
    fn analyse_internal_of_2_file_in_1_folder_in_1_subfolder_in_root_with_empty_metrics() {
        let root_name = "root_with_2_file_in_1_folder_in_1_subfolder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(root_name)
                .join("folder1")
                .join("subfolder1")
                .join("file1"),
            PathBuf::from(root_name)
                .join("folder1")
                .join("subfolder1")
                .join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(files_to_analyze));

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &vec![]);

        // Then

        let expected_file1_analysis = TopAnalysis {
            file_name: String::from("file1"),
            metrics: btreemap! {},
            folder_content: None,
        };
        let expected_file2_analysis = TopAnalysis {
            file_name: String::from("file2"),
            metrics: btreemap! {},
            folder_content: None,
        };

        let expected_subfolder1_analysis_content = btreemap! { expected_file1_analysis.file_name.clone() => expected_file1_analysis,
        expected_file2_analysis.file_name.clone() => expected_file2_analysis};

        let expected_subfolder1_analysis = TopAnalysis {
            file_name: String::from("subfolder1"),
            metrics: btreemap! {},
            folder_content: Some(expected_subfolder1_analysis_content),
        };

        let expected_folder1_analysis_content = btreemap! {expected_subfolder1_analysis.file_name.clone() => expected_subfolder1_analysis};

        let expected_folder1_analysis = TopAnalysis {
            file_name: String::from("folder1"),
            metrics: btreemap! {},
            folder_content: Some(expected_folder1_analysis_content),
        };

        let expected_root_analysis_content =
            btreemap! {expected_folder1_analysis.file_name.clone() => expected_folder1_analysis};

        let expected_root_analysis = TopAnalysis {
            file_name: String::from(root_name),
            metrics: btreemap! {},
            folder_content: Some(expected_root_analysis_content),
        };

        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
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
            Box::new(FakeFileExplorer::_new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Ok(Score(1)));

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
        expected_folder_metrics.insert("fake1", Ok(Score(2)));

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

    #[test]
    fn analyse_with_a_composed_root_should_only_use_root_filename() {
        let root = PathBuf::from("user")
            .join("dir")
            .join("root_with_1_file_in_1_folder");
        let file_to_analyze = vec![root.join("folder1").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer> =
            Box::new(FakeFileExplorer::_new(file_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, &*fake_file_explorer, &metrics);

        // Then
        assert_eq!(
            root.file_name().unwrap(),
            PathBuf::from(actual_root_analysis.file_name)
        );
        assert_eq!(
            "folder1",
            actual_root_analysis
                .folder_content
                .unwrap()
                .get_key_value("folder1")
                .unwrap()
                .1
                .file_name
        );
    }
}
