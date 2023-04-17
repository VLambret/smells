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
        pub lines_count: usize,
        pub social_complexity: u32
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
        println!("analyse root {}", root.display());
        analyse_root(root)
    }
}

mod internal_process{
    use std::env;
    use std::fs::{DirEntry, File, read_dir};
    use std::path::PathBuf;
    use crate::analysis::models::{Analysis, FileAnalysis, FolderAnalysis, Metrics};
    use crate::metrics::{line_count, social_complexity};

    fn analyse_folder(item: PathBuf) -> FolderAnalysis {
        // println!("Analyzing folder: {:?}", item);
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
        //println!("Analyzing folder root: {:?}", root.display());
        analyse_folder(root)
    }

    // sort files based on the entry names
    fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry>{
        // TODO: handle unwrap()
        //println!("sort_files_of_a_path: {:?}", item);
        let existing_proof = item.exists();
        let existing_proof2 = (PathBuf::from("tests").join("data").join("empty_folder")).exists();
        println!("{}", env::current_dir().unwrap().display());
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
        println!("analyse_file: {:?}", path);
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