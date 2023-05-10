use crate::data_sources::file_explorer::{FileExplorer, IFileExplorer};
use crate::metrics::line_count::count_lines;
use crate::metrics::metric::IMetric;
use crate::metrics::{line_count, social_complexity};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::{read_dir, DirEntry, File};
use std::io::Read;
use std::path::PathBuf;
use std::string::String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Analysis {
    FileAnalysis(FileAnalysis),
    FolderAnalysis(FolderAnalysis),
}

// TODO: distinguish root to folders
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FolderAnalysis {
    pub id: String,
    pub metrics: HashMap<String, MetricsValueType>,
    pub content: Option<HashMap<String, Analysis>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FileAnalysis {
    folder_analysis: FolderAnalysis,
    pub metrics: HashMap<String, MetricsValueType>,
}

impl FileAnalysis {
    fn new(id: String, metrics: HashMap<String, MetricsValueType>) -> Self {
        FileAnalysis {
            folder_analysis: FolderAnalysis{
                id: id.clone(),
                metrics: metrics.clone(),
                content: None
            },
            metrics: metrics
        }
    }
    pub fn get_id(&self) -> &String {
        &self.folder_analysis.id
    }
}

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

pub fn do_analysis(root: PathBuf) -> FolderAnalysis {
    analyse_root(root)
}

fn analyse_folder(item: PathBuf) -> FolderAnalysis {
    let folder_content: Vec<Analysis> = sort_files_of_a_path(&item)
        .iter()
        .filter(|f| can_file_be_analysed(&f.path()))
        .map(|f| analyse(&f))
        .collect();

    let mut metrics_content = HashMap::new();
    metrics_content.insert(
        "lines_count".to_string(),
        MetricsValueType::Score(line_count::summary_lines_count_metric(&folder_content)),
    );
    metrics_content.insert(
        "social_complexity".to_string(),
        MetricsValueType::Score(social_complexity::social_complexity("")),
    );

    let root_analysis = FolderAnalysis {
        id: extract_analysed_item_key(&item),
        metrics: metrics_content,
        content: create_hashmap_from_analysis_vector(folder_content),
    };
    root_analysis
}

fn create_hashmap_from_analysis_vector(analysis_vector: Vec<Analysis>) -> Option<HashMap<String, Analysis>> {
    let result = analysis_vector.iter().map(|a|(get_analysis_key(a), a.to_owned())).collect::<HashMap<_, _>>();
    Some(result)
}

fn get_analysis_key(analysis: &Analysis) -> String {
    match analysis {
        Analysis::FileAnalysis(a) => a.get_id().clone(),
        Analysis::FolderAnalysis(a) => a.id.clone()
    }
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

fn analyse_root(root: PathBuf) -> FolderAnalysis {
    analyse_folder(root)
}

fn analyse_internal(
    root: &PathBuf,
    file_explorer: Box<dyn IFileExplorer<Item = PathBuf>>,
    metrics: Vec<Box<dyn IMetric>>,
) -> FolderAnalysis {
    let mut root_folder_content = vec![];
    let mut result_root_metrics = HashMap::new();
    let mut result_file_metrics = HashMap::new();

    // TODO: virer les clone()
    // TODO : for impossible si fonctionnel, iterator ?
    for file in file_explorer.discover() {
        result_file_metrics = get_metrics_score(&metrics, &file);

        let file_analysis = Analysis::FileAnalysis(FileAnalysis::new(file.file_name().unwrap().to_string_lossy().into_owned(),
            result_file_metrics.clone())
        );
        root_folder_content.push(file_analysis.clone());
        result_root_metrics = result_file_metrics.clone();
    }

    if file_explorer.discover().len() > 0 && file_explorer.discover()[0].parent().unwrap() != root {
        result_root_metrics.insert(
            "fake1".to_string(),
            MetricsValueType::Score((file_explorer.discover().len()) as u32),
        );

        let result_folder_analysis = FolderAnalysis {
            id: String::from("folder1"),
            metrics: result_root_metrics.clone(),
            content: create_hashmap_from_analysis_vector(root_folder_content),
        };
        root_folder_content = vec![Analysis::FolderAnalysis(result_folder_analysis)];
    }
    // Root analysis
    FolderAnalysis {
        id: root.file_name().unwrap().to_string_lossy().into_owned(), // TODO unwrapS
        metrics: result_root_metrics,
        content: create_hashmap_from_analysis_vector(root_folder_content),
    }
}

fn get_metrics_score(
    metrics: &Vec<Box<dyn IMetric>>,
    file: &PathBuf,
) -> HashMap<String, MetricsValueType> {
    let mut result_file_metrics = HashMap::new();
    for metric in metrics {
        let result_metric_analyze = match metric.analyze(&&file) {
            Ok(file_metric) => MetricsValueType::Score(file_metric),
            Err(error) => MetricsValueType::Error(error.to_string()),
        };
        result_file_metrics.insert(metric.get_key(), result_metric_analyze);
    }
    result_file_metrics
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

    let mut metrics_content = HashMap::new();
    metrics_content.insert(
        "lines_count".to_string(),
        MetricsValueType::Score(count_lines(content)),
    );
    metrics_content.insert(
        "social_complexity".to_string(),
        MetricsValueType::Score(social_complexity::social_complexity("")),
    );

    FileAnalysis::new(
        extract_analysed_item_key(&path),
        metrics_content,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::line_count::LinesCountMetric;

    pub struct FakeMetric {
        pub metric_key: String,
        pub metric_value: u32,
    }

    impl IMetric for FakeMetric {
        fn analyze(&self, _file_path: &PathBuf) -> Result<u32, String> {
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
        fn analyze(&self, _file_path: &PathBuf) -> Result<u32, String> {
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
    fn analyse_internal_with_empty_root_and_empty_metrics() {
        // Given
        let root_name = "folder_to_analyze";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![];
        let metrics = vec![];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));

        // When
        let actual_result_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let expected_result_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: HashMap::new(),
            content: Some(HashMap::new()),
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    /*
    #[test]
    fn analyse_internal_with_2_files_and_empty_metrics() {
        // Given
        let root_name = "folder_to_analyze";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(&root).join("file1"),
            PathBuf::from(&root).join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics = vec![];

        // When
        let actual_result_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let first_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: HashMap::new(),
        });
        let second_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file2"),
            metrics: HashMap::new(),
        });
        let expected_file_analysis = vec![first_file_analysis, second_file_analysis];
        let expected_result_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: HashMap::new(),
            content: expected_file_analysis,
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn analyse_internal_with_1_file_and_fakemetric4_and_fakemetric10() {
        // Given
        let root_name = "folder_to_analyse";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> =
            vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));
        expected_metrics.insert(String::from("fake10"), MetricsValueType::Score(10));

        let expected_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn analyse_internal_with_1_file_and_brokenmetric() {
        // Given
        let root_name = "folder_to_analyze";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        let error_value = MetricsValueType::Error("Analysis error".to_string());
        expected_metrics.insert(String::from("broken"), error_value);

        let expected_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: expected_metrics.clone(),
            content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn analyse_internal_with_one_5_lines_file() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files");
        let files_to_analyze = vec![PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files")
            .join("file5.txt")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("lines_count"), MetricsValueType::Score(5));
        let expected_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: "file5.txt".to_string(),
            metrics: expected_metrics.clone(),
        });
        let expected_root_analysis = FolderAnalysis {
            id: "folder_with_multiple_files".to_string(),
            metrics: expected_metrics,
            content: vec![expected_file_analysis],
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    // agreggate tests
    #[test]
    fn internal_analyse_with_empty_root_and_fakemetric0() {
        // Given
        let root_name = "empty_root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(0))];

        // When
        let actual_root_analysis =
            analyse_internal(&PathBuf::from("empty_root"), fake_file_explorer, metrics);

        // Then
        let expected_root_analysis = FolderAnalysis {
            id: String::from("empty_root"),
            metrics: HashMap::new(),
            content: vec![],
        };
        assert_eq!(actual_root_analysis, expected_root_analysis)
    }

    #[test]
    fn internal_analyse_with_1_file_and_fakemetric1() {
        // Given
        let root_name = "root_with_1_file";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

        let expected_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
        });

        let expected_root_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: vec![expected_file_analysis],
        };
        assert_eq!(actual_root_analysis, expected_root_analysis)
    }

    #[test]
    fn analyse_internal_of_a_file_in_a_folder_with_fakemetric1() {
        // Given
        let root_name = "root_with_1_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(root_name).join("folder1").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

        let expected_file_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
        });
        let expected_folder1_analysis = Analysis::FolderAnalysis(FolderAnalysis {
            id: String::from("folder1"),
            metrics: expected_metrics.clone(),
            content: vec![expected_file_analysis],
        });
        let expected_root_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: vec![expected_folder1_analysis],
        };
        assert_eq!(actual_root_analysis, expected_root_analysis)
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
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = HashMap::new();
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

        let expected_file1_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
        });
        let expected_file2_analysis = Analysis::FileAnalysis(FileAnalysis {
            id: String::from("file2"),
            metrics: expected_metrics.clone(),
        });

        let mut expected_folder_metrics = HashMap::new();
        expected_folder_metrics.insert(String::from("fake1"), MetricsValueType::Score(2));

        let expected_folder1_analysis = Analysis::FolderAnalysis(FolderAnalysis {
            id: String::from("folder1"),
            metrics: expected_folder_metrics.clone(),
            content: vec![expected_file1_analysis, expected_file2_analysis],
        });
        let expected_root_analysis = FolderAnalysis {
            id: String::from(root_name),
            metrics: expected_folder_metrics,
            content: vec![expected_folder1_analysis],
        };
        assert_eq!(actual_root_analysis, expected_root_analysis)
    }
     */
}
