use structopt::StructOpt;
use std::path::PathBuf;
use serde_json::{Result, Value};

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

fn extract_file_content(file: PathBuf) -> String{
    // extract all files of a folder
    let mut file_keys = "".to_string();
    if let Ok(entries) = std::fs::read_dir(&file) {
        for entry in entries {
            if let Ok(entry) = entry {
                file_keys = extract_key(&entry.path());
            }
        }
    }

    // create the file content if needed
    let mut file_content = "".to_string();
    let mut path_to_compare = PathBuf::new();
    path_to_compare.push(".");
    if !file.read_dir().unwrap().next().is_none() && file != path_to_compare{
        file_content = format!(
        r#""{}": {{
            "metrics": {{
                "lines_metric": 0
            }}
        }}"#, file_keys);
    }
    file_content
}

fn print_analysis(analysis: AnalysisResult) -> Result<()>{
    let file_key = extract_key(&analysis.file);
    let file_content = extract_file_content(analysis.file);
    let json_output = format!(
    r#"{{
        "{}": {{
            "metrics": {{
                "lines_metric": 0
            }},
            "folder_content": {{{}}}
        }}
    }}"#, file_key, file_content);

    let converted_json_output: Value = serde_json::from_str(&json_output)?;
    print!("{}", serde_json::to_string_pretty(&converted_json_output)?);
    Ok(())
}