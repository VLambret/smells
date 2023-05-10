use crate::analysis::Analysis;
use serde_json::{json, Value};

// print analysis result json
pub fn convert_analysis_to_formatted_json(analysis: Analysis) -> String {
    if analysis.content.is_some(){
        let json_result_analysis = build_json_folder_analysis(&analysis);
        format_json_output(&serde_json::to_string(&json_result_analysis).unwrap())
    } else {
        let json_result_analysis = build_json_file_analysis(&analysis);
        format_json_output(&serde_json::to_string(&json_result_analysis).unwrap())
    }
}

fn build_json_folder_analysis(folder: &Analysis) -> Value {
    let mut folder_content_json = Vec::new();
    if let Some(content) = &folder.content {
        for (filename, analysis) in content {
            let json_item =  build_json_folder_analysis(analysis);
            folder_content_json.push(json_item);
        }
    }
    json!(
        {
            folder.id.to_owned():{
            "metrics": folder.metrics,
            "folder_content": folder_content_json
             }
        }
    )
}

fn build_json_file_analysis(file: &Analysis) -> Value {
    json!(
        {
            &file.id :{
            "metrics": file.metrics
            }
        }
    )
}

fn format_json_output(json_output: &String) -> String {
    // TODO: remonter le from_str
    return match serde_json::from_str::<Value>(json_output) {
        Ok(converted_json_output) => {
            match serde_json::to_string_pretty(&converted_json_output) {
                Ok(pretty_json) => pretty_json,
                Err(..) => {
                    // if formatting fails we print the original version
                    json_output.to_owned()
                }
            }
        }
        Err(e) => {
            eprintln!("Error for serializing JSON: {}", e);
            e.to_string()
        }
    };
}
