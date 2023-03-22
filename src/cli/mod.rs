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
    file: String,
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

fn do_analysis(file: PathBuf) -> Result<()> {
    print_analysis(analyse(file))?;
    Ok(())
}

fn analyse(folder: PathBuf) -> AnalysisResult{
    let metrics_content = Metrics{
        lines_count: get_file_line_metrics(&folder)
    };

    let file_content1 = AnalysisResult{
        file: extract_files_name(&folder),
        metrics: metrics_content,
        folder_content: None
    };

    let metrics = Metrics {
        lines_count: get_file_line_metrics(&folder)
    };

    let mut vec_content = Vec::new();

    AnalysisResult{
        file: extract_key(&folder),
        metrics,
        folder_content: if file_is_empty(&folder) || file_is_current_folder(&folder){
            None
        } else {
            vec_content.push(file_content1);
            Some(vec_content)
        },
    }
}

fn file_is_current_folder(file: &PathBuf) -> bool{
    if let Ok(_path) = file.read_dir(){
        let mut path_to_compare = PathBuf::new();
        path_to_compare.push(".");
        if file.to_path_buf() == path_to_compare{
            return true;
        }
    }
    false
}

fn file_is_empty(folder_path: &PathBuf) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(folder_path) {
        return entries.next().is_none();
    }
    false
}

fn extract_key(file: &PathBuf) -> String{
    // extract the file name of the target given by program parameter
    let file_os_str = file.as_os_str();
    let file_key = match file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => file_os_str.to_owned(),
    };
    file_key.to_string_lossy().into_owned()
}

fn extract_files_name(file: &PathBuf) -> String{
    // extract all files name of a folder
    let mut files_name = "".to_string();
    if let Ok(entries) = std::fs::read_dir(&file) {
        for entry in entries {
            if let Ok(entry) = entry {
                files_name = extract_key(&entry.path());
            }
        }
    }
    files_name
}

fn get_file_line_metrics(file: &PathBuf) -> u32{
    let mut lines_metric = 0;
    let mut path_to_compare = PathBuf::new();
    path_to_compare.push(".");
    let files_name = extract_files_name(&file);
    let files_to_analyse = file.to_string_lossy().into_owned().to_string()+"/"+&files_name;
    let file_reader = BufReader::new(File::open(files_to_analyse).unwrap());
    
    if file.exists() && !file_is_empty(file){
        if file.to_path_buf() != path_to_compare{ 
            for _ in file_reader.lines(){
                lines_metric = lines_metric + 1;
            }
        }
    }
    lines_metric
}


fn build_json_folder_analysis(file: String, json_metrics: &String, folder_content: &String) -> String{
    // build analysis result json
    // build metrics
    format!(
        r#"{{
            "{}": {{
                {}
                {}
            }}
        }}"#, file, json_metrics, folder_content)
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

fn extract_inner_content(inner_contents: &Vec<AnalysisResult>) -> Result<String> {
    let mut file_content_string = String::new();
    for inner_content in inner_contents{
        let inner_result = print_analysis(inner_content.clone())?;
        file_content_string.push_str(&inner_result);
    }
    Ok(file_content_string.to_string())
}

// build folder content array elements
fn extract_folder_content(contents: Vec<AnalysisResult>) -> Result<String>{
    let mut elements: Vec<String> = Vec::new();
    for content in contents.iter(){
        let json_metrics = build_json_metrics(&content.metrics);

        let element = match &content.folder_content {
            Some(folder_content) => {
                let json_folder_content = extract_inner_content(folder_content)?;
                build_json_folder_analysis(content.file.to_string(), &json_metrics, &json_folder_content)
            },
            _ => build_json_file_analysis(content.file.to_string(), &json_metrics)
        };
        elements.push(element);
    }

    Ok(build_array_content(&mut elements))
}

fn build_array_content(elements: &mut Vec<String>) -> String {
    let mut result = String::new();
    let mut skip_first_comma = true;
    for element in elements {
        if !skip_first_comma {
            result.push_str(", ");
        }
        result.push_str(element.as_str());
        skip_first_comma = false;
    }
    result
}

// build analysis result json AND print it
fn print_analysis(analysis: AnalysisResult) -> Result<String>{
    // build analysis result
    // build root item
    let file_key = analysis.file;
    let file_content = analysis.folder_content;
    let mut converted_file_content = "".to_string();
    let json_metrics = build_json_metrics(&analysis.metrics);

    // build folder content
    if let Some(contents) = file_content {
        converted_file_content = extract_folder_content(contents)?;
    }
    let folder_content = format!(
    r#","folder_content": [{}]"#, converted_file_content);
        
    let json_output = build_json_folder_analysis(file_key, &json_metrics, &folder_content);
    // print analysis result
    print_formatted_json(&json_output)?;
    Ok(json_output)
}

fn print_formatted_json(json_output: &String) -> Result<()>{
    let converted_json_output: Value = serde_json::from_str(json_output)?;
    print!("{}", serde_json::to_string_pretty(&converted_json_output)?);
    Ok(())
}