use structopt::StructOpt;
use std::path::PathBuf;
use serde_json::{Result, Value};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Metrics{
    lines_count: u32
}

pub fn smells() -> Result<()> {
    let args = CmdArgs::from_args();
    do_analysis(args.path)?;
    Ok(())
}  

fn do_analysis(item: PathBuf) -> Result<()> {
    print_analysis(analyse(item))?;
    Ok(())
}

fn analyse(item: PathBuf) -> AnalysisResult{
    let metrics_content = Metrics{
        lines_count: compute_lines_count_metric(&item)
    };

    let file_content1 = AnalysisResult{
        item_key: extract_file_name_of_analysed_folder(&item),
        metrics: metrics_content,
        folder_content: None
    };

    let metrics = Metrics {
        lines_count: compute_lines_count_metric(&item)
    };

    let mut folder_contents = Vec::new();

    AnalysisResult{
        item_key: extract_analysed_item_key(&item),
        metrics,
        folder_content: if folder_is_empty(&item) || analysed_item_is_in_current_folder(&item){
            None
        } else {
            folder_contents.push(file_content1);
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

fn extract_file_name_of_analysed_folder(folder: &PathBuf) -> String{
    let mut file_name = String::new();
    if let Ok(files) = std::fs::read_dir(&folder) {
        for file in files {
            if let Ok(entry) = file {
                file_name = extract_analysed_item_key(&entry.path());
            }
        }
    }
    file_name
}

fn compute_lines_count_metric(folder: &PathBuf) -> u32{
    // TODO: handle the unwrap()
    let mut lines_count = 0;
    let mut current_path = PathBuf::new();
    current_path.push(".");
    let file_name = extract_file_name_of_analysed_folder(&folder);
    let file_to_analyse = folder.to_string_lossy().into_owned().to_string()+"/"+&file_name;
    let file_reader = BufReader::new(File::open(file_to_analyse).unwrap());
    
    if folder.exists() && !folder_is_empty(folder){
        if folder.to_path_buf() != current_path {
            for _ in file_reader.lines(){
                lines_count = lines_count + 1;
            }
        }
    }
    lines_count
}

// build json for each item of the folder content array
fn build_json_item_analysis(item: &AnalysisResult) -> String {
    let json_metrics = build_json_metrics(&item.metrics);
    // TODO: remove the unwrap to handle the error correctly :-)
    let item_to_json = match &item.folder_content {
        // item is a folder
        Some(folder_content) => {
            let json_folder_content = build_inner_folder_content(folder_content).unwrap();
            build_json_folder_analysis(item.item_key.to_string(), &json_metrics, &json_folder_content)
        },
        // item is a file
        _ => build_json_file_analysis(item.item_key.to_string(), &json_metrics)
    };
    item_to_json
}

// build folder content array of the root folder
// -> "folder_content" : [XXX]
fn build_root_folder_content_array(items: Vec<AnalysisResult>) -> Result<String>{
    let mut items_result: Vec<String> = Vec::new();
    for item in items.iter(){
        items_result.push(build_json_item_analysis(&item));
    }
    Ok(build_folder_content(&mut items_result))
}

// build content for each folder in the root folder content array (recursive content)
// -> folder_content [ folder2_content : [XXX] ]
fn build_inner_folder_content(folder_contents: &Vec<AnalysisResult>) -> Result<String>{
    let mut folder_content = String::new();
    for item in folder_contents {
        folder_content.push_str(&print_analysis(item.clone())?);
    }
    Ok(folder_content.to_string())
}

// build the content that is in folder content array
/* Result is (if only one file):
{
    "file0.txt": {
        "metrics": {
            "lines_metric": 0
        }
    }
}
*/
fn build_folder_content(items: &mut Vec<String>) -> String {
    let mut result = String::new();
    let mut skip_first_comma = true;
    for item in items {
        if !skip_first_comma {
            result.push_str(", ");
        }
        result.push_str(item.as_str());
        skip_first_comma = false;
    }
    result
}

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

fn build_json_file_analysis(file: String, json_metrics: &String) -> String{
    format!(
        r#"{{
            "{}": {{
                {}
            }}
        }}"#, file, json_metrics)
}

fn build_json_metrics(metrics: &Metrics) -> String{
    // build analysis result json
    // build metrics
    format!(
        r#"
        "metrics": {{
            "lines_metric": {}
        }}
        "#, metrics.lines_count)
}

// build analysis result in json
fn build_json_result_analysis(analysis: AnalysisResult) -> String{
    // build analysis result
    // build root item
    let item_key = analysis.item_key;
    let folder_content = analysis.folder_content;
    let mut converted_file_content = String::new();
    let json_metrics = build_json_metrics(&analysis.metrics);

    // build folder content
    // TODO: handle unwrap()
    if let Some(items) = folder_content {
        converted_file_content = build_root_folder_content_array(items).unwrap();
    }
    let folder_content = format!(
        r#","folder_content": [{}]"#, converted_file_content);
    build_json_folder_analysis(item_key, &json_metrics, &folder_content)
}

// print analysis result json
fn print_analysis(analysis: AnalysisResult) -> Result<String>{
    let json_result_analysis = build_json_result_analysis(analysis);
    print_formatted_json(&json_result_analysis);
    Ok(json_result_analysis)
}

fn print_formatted_json(json_output: &String){
    match serde_json::from_str::<Value>(json_output){
        Ok(converted_json_output) => {
            match serde_json::to_string_pretty(&converted_json_output){
                Ok(pretty_json) => {
                    print!("{}", pretty_json);
                }
                Err(e) => {
                    println!("Error for printing JSON: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error for serializing JSON: {}", e);
        }
    }
}