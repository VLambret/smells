use serde_json::{json, Value};
use crate::analysis::models::{Analysis, FileAnalysis, FolderAnalysis};

// print analysis result json
pub fn convert_analysis_to_formatted_json(analysis: FolderAnalysis) -> String{
    let json_result_analysis = build_json_folder_analysis(&analysis);
    format_json_output(&serde_json::to_string(&json_result_analysis).unwrap())
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

fn format_json_output(json_output: &String) -> String{
    // TODO: remonter le from_str
    match serde_json::from_str::<Value>(json_output){
        Ok(converted_json_output) => {
            match serde_json::to_string_pretty(&converted_json_output){
                Ok(pretty_json) => {
                    return pretty_json;
                }
                Err(..) => {
                    // if formatting fails we print the original version
                    return json_output.to_owned();
                }
            }
        }
        Err(e) => {
            eprintln!("Error for serializing JSON: {}", e);
            return e.to_string();
        }
    }
}