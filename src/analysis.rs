pub mod models{
    use serde::{Serialize, Deserialize};
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum Analysis{
        FileAnalysis(FileAnalysis),
        FolderAnalysis(FolderAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
    pub struct FolderAnalysis {
        pub folder_key: String,
        pub metrics: Metrics,
        pub folder_content: Vec<Analysis>,
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

    pub struct MetricsValue {
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
    use std::env;
    use std::fs::{DirEntry, File, read_dir};
    use std::path::PathBuf;
    use std::ptr::null;
    use predicates::str::is_empty;
    use crate::analysis::models::{Analysis, FileAnalysis, FolderAnalysis, Metrics};
    use crate::metrics::{line_count, social_complexity};

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

    pub fn internal_analyse_root(files : Vec<PathBuf>, metrics : HashMap<String, Metrics> ) -> FolderAnalysis {
        let mut value_lines_count_result_metrics : Option<usize> = None;
        let mut value_social_complexity_result_metrics : Option<u32> = None;
        let folder_key : String;

        if metrics.is_empty() {
            value_lines_count_result_metrics = Some(0);
            value_social_complexity_result_metrics = Some(0);
        };
        let result_metrics = Metrics {
            lines_count: value_lines_count_result_metrics.unwrap_or_else(|| {
                panic!("The parameter 'lines_count' is required.")
            }),
            social_complexity: value_social_complexity_result_metrics.unwrap_or_else(|| {
                panic!("The parameter 'social_complexity' is required.")
            }),
        };

        match files.len() {
            0 => folder_key = "".to_string(),
            _ => folder_key = files[0].display().to_string(),
        }

        let result_folder_analysis = FolderAnalysis{
            folder_key,
            metrics : result_metrics,
            folder_content : vec![]
        };
        result_folder_analysis
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
    use std::path::PathBuf;
    use crate::analysis::internal_process::internal_analyse_root;
    use crate::analysis::models::{Analysis, FolderAnalysis, Metrics};

    #[test]
    fn test_internal_analyse_root_without_files_and_empty_metrics_should_return_an_empty_analysis() {
        // Given
        let files_to_analyze = vec![];
        let metrics = HashMap::new();
        // When
        let actual_result_analysis = internal_analyse_root(files_to_analyze, metrics);
        // Then
        let expected_result_analysis = FolderAnalysis {
            folder_key: "".to_string(),
            metrics: Metrics {
                lines_count: 0,
                social_complexity: 0
            },
            folder_content: vec![],
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn test_internal_analyse_root_with_1_empty_folder_and_empty_metrics_should_return_an_empty_metric_for_this_folder() {
        // Given
        let files_to_analyze = vec![PathBuf::from("test_folder")];
        let metrics = HashMap::new();
        // When
        let actual_result_analysis = internal_analyse_root(files_to_analyze, metrics);
        // Then
        let expected_result_analysis = FolderAnalysis {
            folder_key: "test_folder".to_string(),
            metrics: Metrics {
                lines_count: 0,
                social_complexity: 0
            },
            folder_content: vec![],
        };
        assert_eq!(actual_result_analysis, expected_result_analysis);
    }

    #[test]
    fn test_internal_analyse_root_with_2_empty_folders_and_empty_metrics_should_return_an_empty_metric_for_these_folders() {
        // Given
        let files_to_analyze = vec![PathBuf::from("test_folder"), PathBuf::from("test_folder2")];
        let metrics = HashMap::new();
        // When
        let actual_result_analysis = internal_analyse_root(files_to_analyze, metrics);
        // Then
        let expected_result_analysis = FolderAnalysis {

        }
    }
}