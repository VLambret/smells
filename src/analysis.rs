pub mod models{
    use serde::{Serialize, Deserialize};
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum Analysis{
        FileAnalysis(FileAnalysis),
        FolderAnalysis(FolderAnalysis),
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct FolderAnalysis {
        pub folder_key: String,
        pub metrics: Metrics,
        pub folder_content: Vec<Analysis>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct FileAnalysis {
        pub file_key: String,
        pub metrics: Metrics,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct Metrics{
        pub lines_count: usize
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
    use std::fs::{DirEntry, File, read_dir};
    use std::path::PathBuf;
    use crate::analysis::models::{Analysis, FileAnalysis, FolderAnalysis, Metrics};
    use crate::metrics::line_count;

    fn analyse_folder(item: PathBuf) -> FolderAnalysis {
        let folder_content: Vec<Analysis> = sort_files_of_a_path(&item)
            .iter()
            .filter(|f| can_file_be_analysed(&f.path()))
            .map(|f| analyse(&f))
            .collect();

        let metrics_content = Metrics {
            lines_count: line_count::summary_lines_count_metric(&folder_content)
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

    // sort files based on the entry names
    fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry>{
        // TODO: handle unwrap()
        let dir = read_dir(&item).unwrap();
        let mut entries: Vec<_> = dir.map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        entries
    }

    // create the file content for the analysis
    fn analyse_file(entry: &DirEntry) -> FileAnalysis{
        // TODO: handle unwrap()
        let path = entry.path();
        let file = File::open(&path).unwrap();
        let metrics = Metrics {
            lines_count: line_count::compute_lines_count_metric(&file)
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