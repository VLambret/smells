use structopt::StructOpt;
use crate::analysis::public_interface::{CmdArgs, do_analysis};
use crate::formatters::json::convert_analysis_to_formatted_json;

mod formatters;
mod metrics;
mod analysis;

fn main() {
    let analysed_folder = CmdArgs::from_args().path;
    let analysis = do_analysis(analysed_folder);
    let json_formatted_output = convert_analysis_to_formatted_json(analysis);
    print!("{}", json_formatted_output);
}
