use structopt::StructOpt;
use std::path::PathBuf;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnalysisResult{
    item_key: String,
    metrics: Metrics,
    folder_content: Option<Vec<AnalysisResult>>
}

#[derive(Serialize, Deserialize)]
struct FileMetrics {
    file: String,
    metrics: Metrics,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Metrics{
    lines_metric: u32
}

pub fn smells(){
    let args = CmdArgs::from_args();
    do_analysis(args.path);
}  

fn do_analysis(item: PathBuf){
    print_analysis(analyse(item));
}

fn analyse(item: PathBuf) -> AnalysisResult{

    let mut folder_contents = Vec::new();


    /*for entry in std::fs::read_dir(item.clone()).unwrap(){
        let path = entry.unwrap().path();
        println!("{:?}", path);
        let metrics = Metrics{
            lines_count: compute_lines_count_metric(&path)
        };

        let file = AnalysisResult{
            item_key: extract_file_name(&path),
            metrics,
            folder_content: None
        };

        folder_contents.push(file);
    }*/

    let metrics = Metrics{
        lines_metric: compute_lines_count_metric(&item)
    };

    let file = AnalysisResult{
        item_key: extract_file_name(&item),
        metrics,
        folder_content: None
    };

    let metrics_content = Metrics {
    lines_metric: compute_lines_count_metric(&item.clone())
    };

    AnalysisResult{
        item_key: extract_analysed_item_key(&item),
        metrics: metrics_content,
        folder_content: if folder_is_empty(&item) || analysed_item_is_in_current_folder(&item){
            None
        } else {
            folder_contents.push(file);
            Some(folder_contents)
        },
    }
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

fn extract_file_name(file: &PathBuf) -> String{
    let mut file_name = String::new();
    if let Ok(files) = std::fs::read_dir(&file) {
        for file in files {
            if let Ok(entry) = file {
                file_name = extract_analysed_item_key(&entry.path());
            }
        }
    }
    file_name
}

fn compute_lines_count_metric(folder: &PathBuf) -> u32{
    let mut lines_count: u32 = 0;
    let mut current_path = PathBuf::new();
    current_path.push(".");
    let file_name = extract_file_name(&folder);
    let file_to_analyse = folder.to_string_lossy().into_owned().to_string()+"/"+&file_name;

    match try_opening_file(file_to_analyse){
        Ok(file) => {
            let file_reader = BufReader::new(file);
            if folder.exists() && !folder_is_empty(folder){
                if folder.to_path_buf() != current_path {
                    for _ in file_reader.lines(){
                        lines_count = lines_count + 1;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    lines_count
}

fn try_opening_file(file_to_open: String) -> Result<File, String>{
    let file = match File::open(file_to_open.clone()){
        Ok(file) => file,
        Err(e) => {
            return Err(format!("Can't open {}: {}", file_to_open, e));
        }
    };
    Ok(file)
}

// build json for each item of the folder content array
fn build_json_item_analysis(item: &AnalysisResult) -> String {
    let json_metrics = build_json_metrics(&item.metrics);
    let item_to_json = match &item.folder_content {
        // item is a folder
        Some(folder_content) => {
            let json_folder_content = build_inner_folder_content(folder_content);
            build_json_folder_analysis(item.item_key.to_string(), &json_metrics, &json_folder_content)
        },
        // item is a file
        _ => build_json_file_analysis(&item)
    };
    item_to_json
}

// build json folder content of the root folder
// -> "folder_content" : [XXX]
fn build_root_folder_content(items: Vec<AnalysisResult>) -> String{
    items.iter().map(|item| build_json_item_analysis(item)).collect()
}

// build content for each folder in the json root folder content (recursive content)
// -> folder_content [ folder2_content : [XXX] ]
fn build_inner_folder_content(folder_contents: &Vec<AnalysisResult>) -> String{
    folder_contents.iter().map(|item| print_analysis(item.clone())).collect::<String>()
}

// build the content that is in folder content array
fn build_json_folder_analysis(folder: String, json_metrics: &String, folder_content: &String) -> String{
    // build analysis result json
    // build metrics
    format!(
        r#"{{
            "{}": {{
                {}
                {}
            }}
        }}"#, folder, json_metrics, folder_content)
}

fn build_json_file_analysis(file: &AnalysisResult) -> String{
    // TODO: handle the unwrap() ?
    let file_metrics = FileMetrics{
        file: file.item_key.clone(),
        metrics: file.metrics
    };

    serde_json::to_string(&json!(
        {
            file_metrics.file:{
            "metrics": file_metrics.metrics
            }
        }
    )).unwrap()
}

fn build_json_metrics(metrics: &Metrics) -> String {
    // build analysis result json
    format!(r#" "metrics":{}"#, json!({"lines_metric": metrics.lines_metric}))
}

// build analysis result in json
fn build_json_result_analysis(analysis: AnalysisResult) -> String{
    // build root item
    let item_key = analysis.item_key;
    let folder_content = analysis.folder_content;
    let mut folder_content_result = String::new();
    let json_metrics = build_json_metrics(&analysis.metrics);

    // build folder content
    if let Some(items) = folder_content {
        folder_content_result = build_root_folder_content(items);
    }
    let folder_content = format!(
        r#","folder_content": [{}]"#, folder_content_result);
    build_json_folder_analysis(item_key, &json_metrics, &folder_content)
}

// print analysis result json
fn print_analysis(analysis: AnalysisResult) -> String{
    let json_result_analysis = build_json_result_analysis(analysis);
    print_formatted_json(&json_result_analysis);
    json_result_analysis
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