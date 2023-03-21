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
    file_content: Option<Vec<AnalysisResult>>
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

    let file_content = AnalysisResult{
        file: extract_files_name(&folder),
        metrics: metrics_content,
        file_content: None
    };

    let metrics = Metrics {
        lines_count: get_file_line_metrics(&folder)
    };

    let mut vec_content = Vec::new();

    AnalysisResult{
        file: extract_key(&folder),
        metrics,
        file_content: if file_is_empty(&folder) || file_is_current_folder(folder){
            None
        } else {
            vec_content.push(file_content);
            Some(vec_content)
        },
    }
}

fn file_is_current_folder(file: PathBuf) -> bool{
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
    
    if file.exists() && !file_is_empty(file){
        let mut path_to_compare = PathBuf::new();
        path_to_compare.push(".");
        let files_name = extract_files_name(&file); 
        let files_to_analyse = file.to_string_lossy().into_owned().to_string()+"/"+&files_name;
        let file_reader = BufReader::new(File::open(files_to_analyse).unwrap());

        if file.to_path_buf() != path_to_compare{ 
            for _ in file_reader.lines(){
                lines_metric = lines_metric + 1;
            }
        }
    }
    lines_metric
}

fn format_analysis_to_json(file: String, line_count_metric: u32, folder_content: String) -> String{
    format!(
        r#"{{
            "{}": {{
                "metrics": {{
                    "lines_metric": {}
                }}
                {}
            }}
        }}"#, file, line_count_metric, folder_content)
}

fn print_analysis(analysis: AnalysisResult) -> Result<String>{
    let file_key = analysis.file;
    let lines_metric = analysis.metrics.lines_count;
    let file_content = analysis.file_content;
    let mut converted_file_content = "".to_string();

    if let Some(contents) = file_content {
        for content in contents{
            let mut file_content_string = String::new();
            if let Some(inner_contents) = &content.file_content {
                for inner_content in inner_contents.iter(){
                    let inner_result = print_analysis(inner_content.clone())?;
                file_content_string.push_str(&inner_result);
                }
            }
            converted_file_content = format_analysis_to_json(content.file.to_string(), content.metrics.lines_count, file_content_string);
        }
    }
    let folder_content = format!(
    r#","folder_content": [{}]"#, converted_file_content);
        
    let json_output = format_analysis_to_json(file_key, lines_metric, folder_content);
    print_formatted_json(json_output.clone())?;
    Ok(json_output)
}

fn print_formatted_json(json_output: String) -> Result<()>{
    let converted_json_output: Value = serde_json::from_str(&json_output)?;
    print!("{}", serde_json::to_string_pretty(&converted_json_output)?);
    Ok(())
}