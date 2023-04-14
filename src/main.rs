use structopt::StructOpt;
use crate::analysis::public_interface::{CmdArgs, do_analysis};
use crate::formatters::json::convert_analysis_to_formatted_json;
use crate::viewers::cli::print_formatted_json_output;

mod formatters;
mod metrics;
mod analysis;
mod viewers;
mod data_sources;

fn main() {
    let analysed_folder = CmdArgs::from_args().path;
    let analysis = do_analysis(analysed_folder);
    let formatted_json_output = convert_analysis_to_formatted_json(analysis);
    print_formatted_json_output(formatted_json_output);
}
