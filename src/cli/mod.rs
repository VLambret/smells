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

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResult{
    file: String,
    metrics: Metrics,
    file_content: Option<Box<AnalysisResult>>
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics{
    lines_count: u32
}

pub fn smells() -> Result<()> {
    let args = CmdArgs::from_args();
    do_analysis(args.path)?;
    Ok(())
}  

fn do_analysis(file: PathBuf) -> Result<()> {
    let analysis = analyse(file);
    print_analysis(analysis)?;
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


    AnalysisResult{
        file: extract_key(&folder),
        metrics,
        //file_content: Some(Box::new(file_content))
        file_content: if !file_is_empty_or_current_folder(folder) {
            Some(Box::new(file_content))
        } else {
            None
        },
    }
}

fn file_is_empty_or_current_folder(file: PathBuf) -> bool{
    if let Ok(mut path) = file.read_dir(){
        let mut path_to_compare = PathBuf::new();
        path_to_compare.push(".");
        if path.next().is_none() || file.to_path_buf() == path_to_compare{
            return true;
        }
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

fn extract_file_content(file: &PathBuf) -> String{
    // create the file content if needed to be displayed in print_analysis
    let mut file_content = "".to_string();

    if let Ok(mut path) = file.read_dir(){
        let mut path_to_compare = PathBuf::new();
        path_to_compare.push(".");
        
        if !path.next().is_none() && file.to_path_buf() != path_to_compare{
            let files_name = extract_files_name(&file);
            let lines_metric = get_file_line_metrics(&file);
            file_content = format!(
            r#""{}": {{
                "metrics": {{
                    "lines_metric": {}
                }}
            }}"#, files_name, lines_metric);
        }
    }
    file_content
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
    
    if file.exists() && !is_folder_empty(file){
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

fn is_folder_empty(folder_path: &PathBuf) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(folder_path) {
        return entries.next().is_none();
    }
    false
}

fn print_analysis(analysis: AnalysisResult) -> Result<()>{
    let file_key = analysis.file;
    let lines_metric = analysis.metrics.lines_count;
    let file_content = analysis.file_content;

    let (file, lines_metric_content) = file_content
        .as_ref()
        .map(|content| (content.file.as_str(), content.metrics.lines_count))
        .unwrap_or(("", 0));

    let mut converted_file_content = "".to_string();

    if !file_content.is_none() {
        converted_file_content = format!(
        r#"{{
            "{}": {{
                "metrics": {{
                    "lines_metric": {}
                }},
                "folder_content": [{}]
            }}
        }}"#, file, lines_metric_content, "");
    }
        
    
    let json_output = format!(
    r#"{{
        "{}": {{
            "metrics": {{
                "lines_metric": {}
            }},
            "folder_content": [{}]
        }}
    }}"#, file_key, lines_metric, converted_file_content);

    let converted_json_output: Value = serde_json::from_str(&json_output)?;
    print!("{}", serde_json::to_string_pretty(&converted_json_output)?);
    
    Ok(())
}