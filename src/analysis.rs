pub mod models {
    use serde::ser::SerializeStruct;
    use serde::{Deserialize, Serialize, Serializer};
    use std::collections::HashMap;
    use std::fmt;
    use std::io::ErrorKind;

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum Analysis {
        FileAnalysis(FileAnalysis),
        FolderAnalysis(FolderAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum AnalysisTest {
        FileAnalysisTest(FileAnalysisTest),
        FolderAnalysisTest(RootAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct FolderAnalysis {
        pub folder_key: String,
        pub metrics: HashMap<String, MetricsValueType>,
        pub folder_content: Vec<Analysis>,
    }

    /*    impl Serialize for FolderAnalysis {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            let mut state = serializer.serialize_struct("FolderAnalysis", 3)?;
            state.serialize_field("folder_key", &self.folder_key)?;
            state.serialize_field("metrics", &self.metrics)?;
            state.serialize_field("folder_content", &self.folder_content)?;
            state.end()
        }
    }*/

    pub type AnalysisError = String;

    // TODO: rename variants
    #[derive(Debug, Deserialize, Clone, PartialEq)]
    pub enum MetricsValueType {
        Score(u32),
        Error(AnalysisError),
    }

    impl Serialize for MetricsValueType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                MetricsValueType::Score(score) => serializer.serialize_u32(*score),
                MetricsValueType::Error(error) => serializer.serialize_str(error),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum MetricOrError {
        MetricsValueType(MetricsValueType),
        Error(String),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct RootAnalysis {
        pub folder_key: String,
        pub metrics: HashMap<String, MetricsValueType>,
        pub folder_content: Vec<AnalysisTest>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct FileAnalysisTest {
        pub file_key: String,
        pub metrics: HashMap<String, MetricsValueType>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct FileAnalysis {
        pub file_key: String,
        pub metrics: Metrics,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
    pub struct Metrics {
        pub lines_count: usize,
        pub social_complexity: u32,
    }

    pub struct MetricsTest {
        code: u32,
        nom: Metrics,
    }
}

pub mod public_interface {
    use crate::analysis::internal_process::analyse_root;
    use crate::analysis::models::FolderAnalysis;
    use std::path::PathBuf;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    pub struct CmdArgs {
        #[structopt(default_value = ".")]
        pub path: PathBuf,
    }

    pub fn do_analysis(root: PathBuf) -> FolderAnalysis {
        analyse_root(root)
    }
}

mod internal_process {
    use crate::analysis::models::{
        Analysis, AnalysisTest, FileAnalysis, FileAnalysisTest, FolderAnalysis, MetricOrError,
        Metrics, MetricsValueType, RootAnalysis,
    };
    use crate::metrics::line_count::count_lines;
    use crate::metrics::metric::IMetric;
    use crate::metrics::social_complexity::social_complexity;
    use crate::metrics::{line_count, social_complexity};
    use std::collections::HashMap;
    use std::fs::{read_dir, DirEntry, File};
    use std::io::Read;
    use std::path::PathBuf;

    fn analyse_folder(item: PathBuf) -> FolderAnalysis {
        let folder_content: Vec<Analysis> = sort_files_of_a_path(&item)
            .iter()
            .filter(|f| can_file_be_analysed(&f.path()))
            .map(|f| analyse(&f))
            .collect();

        let mut metrics_content = HashMap::new();
        let line_count_metric =
            MetricsValueType::Score(line_count::summary_lines_count_metric(&folder_content) as u32);

        let social_complexity_metric =
            MetricsValueType::Score(line_count::summary_lines_count_metric(&folder_content) as u32);

        metrics_content.insert("lines_count".to_string(), line_count_metric);
        metrics_content.insert("social_complexity".to_string(), social_complexity_metric);

        let root_analysis = FolderAnalysis {
            folder_key: extract_analysed_item_key(&item),
            metrics: metrics_content,
            folder_content,
        };
        root_analysis
    }

    fn analyse(entry: &DirEntry) -> Analysis {
        let analysis: Analysis;
        if entry.path().is_file() {
            analysis = Analysis::FileAnalysis(analyse_file(entry));
        } else {
            analysis = Analysis::FolderAnalysis(analyse_folder(entry.path()));
        }
        analysis
    }

    pub fn analyse_root(root: PathBuf) -> FolderAnalysis {
        analyse_folder(root)
    }

    pub fn internal_analyse_root(
        root: &PathBuf,
        files: Vec<PathBuf>,
        metrics: Vec<Box<dyn IMetric>>,
    ) -> RootAnalysis {
        let mut result_file_metrics = HashMap::new();
        let mut result_files_analysis = Vec::new();

        for file in files {
            for metric in &metrics {
                let result_metric_analyze = match metric.analyze(&file) {
                    Ok(file_metric) => MetricsValueType::Score(file_metric),
                    Err(error) => MetricsValueType::Error(error.to_string()),
                };
                result_file_metrics.insert(metric.get_key(), result_metric_analyze);
            }
            let file_analysis_test = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
                file_key: file.file_name().unwrap().to_string_lossy().into_owned(), // TODO unwrap
                metrics: result_file_metrics.clone(),
            });
            result_files_analysis.push(file_analysis_test);
        }

        RootAnalysis {
            folder_key: root.file_name().unwrap().to_string_lossy().into_owned(), // TODO unwrapS
            metrics: result_file_metrics,
            folder_content: result_files_analysis,
        }
    }

    // sort files based on the entry names
    fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry> {
        // TODO: handle unwrap()
        let dir_result = read_dir(&item);
        let dir = dir_result.unwrap();
        let mut entries: Vec<_> = dir.map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        entries
    }

    // create the file content for the analysis
    fn analyse_file(entry: &DirEntry) -> FileAnalysis {
        // TODO: handle unwrap()
        let path = entry.path();
        let mut file = File::open(&path).unwrap();
        // TODO: remove expect and make metric optional to handle errors when an executable is analyzed
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let metrics = Metrics {
            lines_count: count_lines(content) as usize,
            social_complexity: social_complexity::social_complexity("."), // root_path to find the repo
        };

        FileAnalysis {
            file_key: extract_analysed_item_key(&path),
            metrics,
        }
    }

    fn can_file_be_analysed(item: &PathBuf) -> bool {
        let file_name = match item.file_name() {
            Some(file) => file,
            _ => return false,
        };
        !file_name.to_string_lossy().starts_with(".")
    }

    fn extract_analysed_item_key(item: &PathBuf) -> String {
        let item_as_os_str = item.as_os_str();
        let item_key = match item.file_name() {
            Some(item_name) => item_name.to_owned(),
            _ => item_as_os_str.to_owned(),
        };
        item_key.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::internal_process::internal_analyse_root;
    use crate::analysis::models::{
        AnalysisError, AnalysisTest, FileAnalysisTest, MetricOrError, MetricsValueType,
        RootAnalysis,
    };
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::line_count::LinesCountMetric;
    use crate::metrics::metric::IMetric;
    use std::collections::HashMap;
    use std::io::{Error, ErrorKind};
    use std::path::PathBuf;

    pub struct FakeMetric {
        pub(crate) metric_key: String,
        pub(crate) metric_value: u32,
    }

    impl IMetric for FakeMetric {
        fn analyze(&self, file_path: &PathBuf) -> Result<u32, String> {
            Ok(self.metric_value)
        }
        fn get_key(&self) -> String {
            self.metric_key.to_owned()
        }
    }

    impl FakeMetric {
        pub fn new(metric_value: u32) -> FakeMetric {
            FakeMetric {
                metric_key: format!("fake{}", metric_value),
                metric_value,
            }
        }
    }

    pub struct BrokenMetric {
        pub metric_key: String,
    }

    impl IMetric for BrokenMetric {
        fn analyze(&self, file_path: &PathBuf) -> Result<u32, String> {
            Err(String::from("Analysis error"))
        }
        fn get_key(&self) -> String {
            self.metric_key.to_owned()
        }
    }

    impl BrokenMetric {
        pub fn new() -> BrokenMetric {
            BrokenMetric {
                metric_key: String::from("broken"),
            }
        }
    }

    #[test]
    fn test_internal_analyse_root_without_files_and_empty_metrics_should_return_an_empty_analysis()
    {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files_to_analyze = vec![];
        let fake_file_explorer = FakeFileExplorer::new(files_to_analyze);
        let metrics = vec![];
        // When
        let actual_result_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);
        // Then
        let expected_result_analysis = RootAnalysis {
            folder_key: String::from("folder_to_analyze"),
            metrics: HashMap::new(),
            folder_content: vec![],
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn internal_analyse_root_with_2_files_and_empty_metrics_should_return_an_empty_metric_for_these_files(
    ) {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1"), PathBuf::from("f2")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics = vec![];

        // When
        let actual_result_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);

        // Then
        let first_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: String::from("f1"),
            metrics: HashMap::new(),
        });
        let second_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: String::from("f2"),
            metrics: HashMap::new(),
        });
        let expected_file_analysis = vec![first_file_analysis, second_file_analysis];
        let expected_result_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: HashMap::new(),
            folder_content: expected_file_analysis,
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn internal_analyse_root_with_1_file_and_fake_metric4_should_return_1_analysis_with_4() {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(4))];

        // When
        let actual_root_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));

        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "f1".to_string(),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn internal_analyse_root_with_1_file_and_fake_metric4_and_fake_metric10_should_return_1_analysis_with_4_and_10(
    ) {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> =
            vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));
        expected_metrics.insert(String::from("fake10"), MetricsValueType::Score(10));

        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "f1".to_string(),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn internal_analyse_root_with_1_file_and_broken_metric_should_return_1_analysis_with_an_error()
    {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        let error_value = MetricsValueType::Error("Analysis error".to_string());
        expected_metrics.insert(String::from("broken"), error_value);

        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "f1".to_string(),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics.clone(),
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn internal_analyse_root_with_one_5_lines_file_should_return_one_LinesCount_analysis_with_5() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files");
        let files = vec![PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files")
            .join("file5.txt")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];

        // When
        let actual_root_analysis =
            internal_analyse_root(&root, fake_file_explorer.discover(&root), metrics);

        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("lines_count"), MetricsValueType::Score(5));
        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "file5.txt".to_string(),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_with_multiple_files".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }
}
