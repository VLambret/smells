use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use crate::cli::cli_lib::Analysis;

pub fn summary_lines_metric(folder_contents: &Vec<Analysis>) -> usize {
    folder_contents
        .iter()
        .filter_map(|content| {
            if let Analysis::FileAnalysis(file) = content {
                Some(file.metrics.lines_metric)
            } else if let Analysis::FolderAnalysis(folder) = content{
                Some(folder.metrics.lines_metric)
            }else{
                None
            }
        })
        .sum()
}

pub fn compute_lines_count_metric(file_path: &PathBuf) -> usize {
    // TODO: handle the except
    let file = File::open(file_path).expect("failed to open file");
    let reader = BufReader::new(file);
    reader.lines().count()
}