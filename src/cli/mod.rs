use structopt::StructOpt;
use std::path::PathBuf;
use serde_json::{Result, Value};
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

struct AnalysisResult {
    file: PathBuf
}

pub fn smells() -> Result<()> {
    let args = CmdArgs::from_args();
    do_analysis(args.path)?;
    Ok(())
}  

fn do_analysis(folder: PathBuf) -> Result<()> {
    let analysis = analyse(folder);
    print_analysis(analysis)?;
    Ok(())
}

fn analyse(folder: PathBuf) -> AnalysisResult {
    AnalysisResult{
        file: folder
    }
}

fn extract_key(file: &PathBuf) -> String{
    let file_os_str = file.as_os_str();
    let file_key = match file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => file_os_str.to_owned(),
    };
    file_key.to_string_lossy().into_owned()
}

fn extract_file_content(file: &PathBuf) -> String{
    let files_name = extract_files_name(&file);
    // create the file content if needed to be displayed in print_analysis
    let mut file_content = "".to_string();
    let mut path_to_compare = PathBuf::new();
    let lines_metric = get_file_line_metrics(&file);

    path_to_compare.push(".");

    if let Ok(mut path) = file.read_dir(){
        if !path.next().is_none() && file.to_path_buf() != path_to_compare{
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

fn is_folder_empty(folder_path: &PathBuf) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(folder_path) {
        return entries.next().is_none();
    }
    false
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
    let files_name = extract_files_name(&file); 
    
    let mut path_to_compare = PathBuf::new();
    path_to_compare.push(".");

    if file.exists() && !is_folder_empty(file){
        let file_to_analyse = file.to_string_lossy().into_owned().to_string()+"/"+&files_name;
        let file_reader = BufReader::new(File::open(file_to_analyse).unwrap());
        if file.to_path_buf() != path_to_compare{ 
            for _ in file_reader.lines(){
                lines_metric = lines_metric + 1;
            }
        }
    }
    lines_metric
}


fn print_analysis(analysis: AnalysisResult) -> Result<()>{
    let file_key = extract_key(&analysis.file);
    let file_content = extract_file_content(&analysis.file);
    let lines_metric = get_file_line_metrics(&analysis.file);

    let json_output = format!(
    r#"{{
        "{}": {{
            "metrics": {{
                "lines_metric": {}
            }},
            "folder_content": {{{}}}
        }}
    }}"#, file_key, lines_metric, file_content);

    let converted_json_output: Value = serde_json::from_str(&json_output)?;
    print!("{}", serde_json::to_string_pretty(&converted_json_output)?);
    Ok(())
}