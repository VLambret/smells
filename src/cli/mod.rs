use structopt::StructOpt;
use std::path::PathBuf;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::fs::{DirEntry, File, read_dir};
use serde::{Serialize, Deserialize};

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum Analysis{
    FileAnalysis(FileAnalysis),
    FolderAnalysis(FolderAnalysis),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FolderAnalysis {
    folder_key: String,
    metrics: Metrics,
    folder_content: Vec<Analysis>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileAnalysis {
    file_key: String,
    metrics: Metrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Metrics{
    lines_metric: usize
}

pub fn smells(){
    let args = CmdArgs::from_args();
    do_analysis(args.path);
}  

fn do_analysis(item: PathBuf){
    print_analysis(analyse(item));
}

fn analyse(item: PathBuf) -> FolderAnalysis {
    let mut folder_contents = Vec::new();

    if !folder_is_empty(&item) && !analysed_item_is_in_current_folder(&item) {
        for entry in sort_files_of_a_path(&item){
            folder_contents.push(initialize_file_content(entry));
        }
    }

    let metrics_content = Metrics {
        lines_metric: summary_lines_metric(&folder_contents)
    };

    FolderAnalysis {
        folder_key: extract_analysed_item_key(&item),
        metrics: metrics_content,
        folder_content: folder_contents
    }
}

// sort files based on the entry names
fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry>{
    // TODO: handle unwrap()
    let dir = read_dir(&item).unwrap();
    let mut entries: Vec<_> = dir.map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    entries
}

// create the file content
fn initialize_file_content(entry: DirEntry) -> Analysis{

    let path = entry.path();
    let metrics = Metrics {
        lines_metric: compute_lines_count_metric(&path)
    };

    Analysis::FileAnalysis(FileAnalysis {
        file_key: extract_analysed_item_key(&path),
        metrics
    })
}

fn analysed_item_is_in_current_folder(item: &PathBuf) -> bool{
    if let Ok(_path) = item.read_dir(){
        let mut current_path = PathBuf::new();
        current_path.push(".");
        if item.to_path_buf() == current_path {
            return true;
        }
    }
    false
}

fn folder_is_empty(folder: &PathBuf) -> bool {
    if let Ok(mut folder_entry) = std::fs::read_dir(folder) {
        return folder_entry.next().is_none();
    }
    false
}

fn extract_analysed_item_key(item: &PathBuf) -> String{
    let item_as_os_str = item.as_os_str();
    let item_key = match item.file_name() {
        Some(item_name) => item_name.to_owned(),
        _ => item_as_os_str.to_owned(),
    };
    item_key.to_string_lossy().into_owned()
}

fn summary_lines_metric(folder_contents: &Vec<Analysis>) -> usize {
    folder_contents
        .iter()
        .filter_map(|content| {
            if let Analysis::FileAnalysis(file) = content {
                Some(file.metrics.lines_metric)
            } else {
                None
            }
        })
        .sum()
}

fn compute_lines_count_metric(file_path: &PathBuf) -> usize {
    // TODO: handle the except
    let file = File::open(file_path).expect("failed to open file");
    let reader = BufReader::new(file);
    reader.lines().count()
}

fn build_json_folder_analysis(folder: &FolderAnalysis) -> Value{
    let mut folder_content_json = Vec::new();
    for item in &folder.folder_content{
        let json_item = match item{
            Analysis::FolderAnalysis(sub_folder) => build_json_folder_analysis(sub_folder),
            Analysis::FileAnalysis(sub_file) => build_json_file_analysis(sub_file)
        };
        folder_content_json.push(json_item);
    }
   json!(
       {
           folder.folder_key.to_owned():{
           "metrics": folder.metrics,
           "folder_content": folder_content_json
            }
       }
   )
}

fn build_json_file_analysis(file: &FileAnalysis) -> Value{
    json!(
        {
            file.file_key.to_owned():{
            "metrics": file.metrics
            }
        }
    )
}

// print analysis result json
fn print_analysis(analysis: FolderAnalysis){
    let json_result_analysis = build_json_folder_analysis(&analysis);
    print_formatted_json(&serde_json::to_string(&json_result_analysis).unwrap());
}

fn print_formatted_json(json_output: &String){
    // TODO: remonter le from_str
    match serde_json::from_str::<Value>(json_output){
        Ok(converted_json_output) => {
            match serde_json::to_string_pretty(&converted_json_output){
                Ok(pretty_json) => {
                    print!("{}", pretty_json);
                }
                Err(..) => {
                    // if formatting fails we print the original version
                    println!("{}", json_output);
                }
            }
        }
        Err(e) => {
            eprintln!("Error for serializing JSON: {}", e);
        }
    }
}