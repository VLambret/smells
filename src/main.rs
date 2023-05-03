//use std::path::PathBuf;

use crate::analysis::do_analysis;
use crate::formatters::json::convert_analysis_to_formatted_json;
use crate::viewers::cli::print_formatted_json_output;
use std::path::PathBuf;
use structopt::StructOpt;

mod analysis;
mod data_sources;
mod formatters;
mod metrics;
mod viewers;

#[derive(Debug, StructOpt)]
pub struct CmdArgs {
    #[structopt(default_value = ".")]
    pub path: PathBuf,
}

fn main() {
    let analysed_folder = CmdArgs::from_args().path;
    //let analysed_folder2: PathBuf = PathBuf::from("tests").join("data").join("empty_folder");
    let analysis = do_analysis(analysed_folder);
    let formatted_json_output = convert_analysis_to_formatted_json(analysis);
    print_formatted_json_output(formatted_json_output);
}

#[cfg(test)]
mod main_tests {
    use std::path::PathBuf;
    //use super::*;
    #[test]
    fn test_path_portability() {
        assert_eq!(
            PathBuf::from("dir1\\dir2").display().to_string(),
            "dir1\\dir2".to_string()
        );
    }
}
