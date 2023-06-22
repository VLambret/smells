use crate::analysis_module::analysis::TopAnalysis;
use crate::metrics::metric::{AnalysisError, MetricScoreType};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn convert_analysis_to_formatted_json(analysis: TopAnalysis) -> String {
    format_json_output(
        &serde_json::to_string(&convert_analysis_to_json(&analysis))
            .unwrap_or(String::from("Error during analysis conversion to json")),
    )
}

pub fn convert_analysis_to_json(analysis: &TopAnalysis) -> Value {
    if analysis.folder_content.is_some() {
        build_json_folder_analysis(analysis)
    } else {
        build_json_file_analysis(analysis)
    }
}

enum MetricScoreOrError {
    Score(MetricScoreType),
    Error(AnalysisError),
}

impl Serialize for MetricScoreOrError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Score(MetricScoreType::Score(value)) => serializer.serialize_u64(*value),
            Self::Error(error) => serializer.serialize_str(error),
        }
    }
}

fn build_json_folder_analysis(folder: &TopAnalysis) -> Value {
    let folder_content_json = {
        if let Some(content) = &folder.folder_content {
            content.values().map(convert_analysis_to_json).collect()
        } else {
            vec![]
        }
    };

    json!(
        {
            folder.file_name.to_owned():{
            "metrics": build_analysis_metrics_for_json(folder),
            "folder_content_analyses": folder_content_json
             }
        }
    )
}

fn build_json_file_analysis(file: &TopAnalysis) -> Value {
    json!(
        {
            &file.file_name :{
            "metrics": build_analysis_metrics_for_json(file)
            }
        }
    )
}

fn build_analysis_metrics_for_json(folder: &TopAnalysis) -> BTreeMap<&str, MetricScoreOrError> {
    folder
        .metrics
        .clone()
        .into_iter()
        .map(|metric| {
            let (metric_key, metric_score_result) = metric;
            if let Ok(metric_score) = metric_score_result {
                (metric_key, MetricScoreOrError::Score(metric_score))
            } else {
                (
                    metric_key,
                    MetricScoreOrError::Error(String::from("Analysis error")),
                )
            }
        })
        .collect()
}

fn format_json_output(json_output: &String) -> String {
    // TODO: remonter le from_str
    match serde_json::from_str::<Value>(json_output) {
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
    }
}
