pub mod models{
    use std::collections::HashMap;
    use std::fmt;
    use std::io::ErrorKind;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum Analysis{
        FileAnalysis(FileAnalysis),
        FolderAnalysis(FolderAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum AnalysisTest{
        FileAnalysisTest(FileAnalysisTest),
        FolderAnalysisTest(RootAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub struct FolderAnalysis {
        pub folder_key: String,
        pub metrics: Metrics,
        pub folder_content: Vec<Analysis>,
    }

    pub type AnalysisError = String;

    // TODO: rename variants
    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub enum MetricsValueType {
        Score(u32),
        Error(AnalysisError),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum MetricOrError {
        MetricsValueType(MetricsValueType),
        Error(String),
    }


/*    impl serde::Serialize for MetricOrError {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
        {
            match self {
                MetricOrError::MetricsValueType(metric) => serializer.serialize_newtype_variant("MetricOrError", 0, "Metric", metric),
                MetricOrError::Error(err) => serializer.serialize_newtype_variant("MetricOrError", 1, "Error", err),
            }
        }
    }*/


    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub struct RootAnalysis {
        pub folder_key: String,
        pub metrics: HashMap<String, MetricsValueType>,
        pub folder_content: Vec<AnalysisTest>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub struct FileAnalysisTest {
        pub file_key: String,
        pub metrics: HashMap<String,MetricsValueType>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub struct FileAnalysis {
        pub file_key: String,
        pub metrics: Metrics,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Copy,PartialEq)]
    pub struct Metrics{
        pub lines_count: usize,
        pub social_complexity: u32
    }

    pub struct MetricsTest {
        code : u32,
        nom : Metrics,
    }
}

pub mod public_interface{
    use std::path::PathBuf;
    use structopt::StructOpt;
    use crate::analysis::internal_process::analyse_root;
    use crate::analysis::models::FolderAnalysis;

    #[derive(Debug, StructOpt)]
    pub struct CmdArgs{
        #[structopt(default_value=".")]
        pub path: PathBuf,
    }

    pub fn do_analysis(root: PathBuf) -> FolderAnalysis{
        analyse_root(root)
    }
}

mod internal_process{
    use std::collections::HashMap;
    use std::fs::{DirEntry, File, read_dir};
    use std::path::PathBuf;
    use crate::analysis::models::{Analysis, FileAnalysis, FolderAnalysis, RootAnalysis, Metrics, MetricsValueType, FileAnalysisTest, AnalysisTest, MetricOrError};
    use crate::metrics::{line_count, social_complexity};
    use crate::metrics::metric::IMetric;

    fn analyse_folder(item: PathBuf) -> FolderAnalysis {
        let folder_content: Vec<Analysis> = sort_files_of_a_path(&item)
            .iter()
            .filter(|f| can_file_be_analysed(&f.path()))
            .map(|f| analyse(&f))
            .collect();

        let metrics_content = Metrics {
            lines_count: line_count::summary_lines_count_metric(&folder_content),
            social_complexity: social_complexity::social_complexity(".") // root_path to find the repo
        };
        let root_analysis = FolderAnalysis {
            folder_key: extract_analysed_item_key(&item),
            metrics: metrics_content,
            folder_content
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

    pub fn analyse_root(root: PathBuf) -> FolderAnalysis{
        analyse_folder(root)
    }

    pub fn internal_analyse_root(files : Vec<PathBuf>, metrics : Vec<Box<dyn IMetric>>) -> RootAnalysis {
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
                file_key: file.to_string_lossy().to_string(),
                metrics: result_file_metrics.clone(),
            });
            result_files_analysis.push(file_analysis_test);
        }

        RootAnalysis {
            folder_key : "folder_to_analyze".to_string(),
            metrics : result_file_metrics,
            folder_content : result_files_analysis
        }
    }

    // sort files based on the entry names
    fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry>{
        // TODO: handle unwrap()
        let existing_proof = item.exists();
        let existing_proof2 = (PathBuf::from("tests").join("data").join("empty_folder")).exists();
        let dir_result = read_dir(&item);
        let dir = dir_result.unwrap();
        let mut entries: Vec<_> = dir.map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        entries
    }

    // create the file content for the analysis
    fn analyse_file(entry: &DirEntry) -> FileAnalysis{
        // TODO: handle unwrap()
        let path = entry.path();
        let mut file = File::open(&path).unwrap();
        // TODO: remove expect and make metric optional to handle errors when an executable is analyzed
        let metrics = Metrics {
            lines_count: line_count::compute_lines_count_metric(&mut file).expect("TODO: make metric optional"),
            social_complexity: social_complexity::social_complexity(".") // root_path to find the repo
        };

        FileAnalysis {
            file_key: extract_analysed_item_key(&path),
            metrics
        }
    }

    fn can_file_be_analysed(item: &PathBuf) -> bool{
        let file_name = match item.file_name(){
            Some(file) => file,
            _ => return false
        };
        !file_name.to_string_lossy().starts_with(".")
    }

    fn extract_analysed_item_key(item: &PathBuf) -> String{
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
    use std::collections::HashMap;
    use std::io::{Error, ErrorKind};
    use std::path::PathBuf;
    use crate::analysis::internal_process::internal_analyse_root;
    use crate::analysis::models::{AnalysisTest, FileAnalysisTest, RootAnalysis, MetricsValueType, MetricOrError, AnalysisError};
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::metric::{BrokenMetric, FakeMetric, IMetric, LinesCountMetric};

    #[test]
    fn test_internal_analyse_root_without_files_and_empty_metrics_should_return_an_empty_analysis() {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files_to_analyze = vec![];
        let fake_file_explorer = FakeFileExplorer::new(files_to_analyze);
        let metrics = vec![];
        // When
        let actual_result_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);
        // Then
        let expected_result_analysis = RootAnalysis {
            folder_key: String::from("folder_to_analyze"),
            metrics: HashMap::new(),
            folder_content: vec![],
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn internal_analyse_root_with_2_files_and_empty_metrics_should_return_an_empty_metric_for_these_files() {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1"), PathBuf::from("f2")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics = vec![];

        // When
        let actual_result_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);

        // Then
        let first_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: String::from("f1"),
            metrics: HashMap::new()
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
        let actual_root_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));

        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "f1".to_string(),
            metrics: expected_metrics.clone()
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn internal_analyse_root_with_1_file_and_fake_metric4_and_fake_metric10_should_return_1_analysis_with_4_and_10() {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));
        expected_metrics.insert(String::from("fake10"), MetricsValueType::Score(10));

        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: "f1".to_string(),
            metrics: expected_metrics.clone()
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn internal_analyse_root_with_1_file_and_broken_metric_should_return_1_analysis_with_an_error() {
        // Given
        let root = PathBuf::from("folder_to_analyze");
        let files = vec![PathBuf::from("f1")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        let error_value = MetricsValueType::Error("Analysis error".to_string());
        expected_metrics.insert(String::from("broken"),error_value);

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
    fn internal_analyse_root_with_one_5_lines_file_should_return_1_LinesCount_analysis_with_5_lines() {
        // Given
        let root = PathBuf::from("tests").join("data").join("folder_with_multiple_files");
        let files = vec![PathBuf::from("tests").join("data").join("folder_with_multiple_files").join("file5.txt")];
        let fake_file_explorer = FakeFileExplorer::new(files);
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];

        // When
        let actual_root_analysis = internal_analyse_root(fake_file_explorer.discover(&root), metrics);
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("lines_count"),MetricsValueType::Score(5));
        let expected_file_analysis = AnalysisTest::FileAnalysisTest(FileAnalysisTest {
            file_key: r"tests\data\folder_with_multiple_files\file5.txt".to_string(),
            metrics: expected_metrics.clone()
        });
        let expected_root_analysis = RootAnalysis {
            folder_key: "folder_to_analyze".to_string(),
            metrics: expected_metrics,
            folder_content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }
}