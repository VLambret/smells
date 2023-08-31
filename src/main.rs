use env_logger::Env;
use smells::analysis_module::public_analysis::do_analysis;
use smells::formatters::json::convert_analysis_to_formatted_json;
use smells::viewers::cli::print_formatted_json_output;
use std::path::PathBuf;
use log::info;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CmdArgs {
    #[structopt(parse(try_from_str = get_folder_to_analyse))]
    pub folder_to_analyse: PathBuf,

    /// you can specify multiple extensions by separating them with commas like <ext1,ext2>
    #[structopt(short = "f", long = "filter")]
    pub extension_of_files_to_analyse: Option<String>,
}

fn get_folder_to_analyse(input: &str) -> Result<PathBuf, String> {
    match PathBuf::from(input).canonicalize() {
        Ok(folder) => Ok(folder),
        Err(error) => Err(error.to_string()),
    }
}

fn main() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let folder_to_analyse = CmdArgs::from_args().folder_to_analyse;
    info!("Starting analysis ...");
    let analysis = do_analysis(folder_to_analyse);
    info!("All files have been analysed. Starting JSON conversion of analysis result ...");
    let formatted_json_output = convert_analysis_to_formatted_json(analysis);
    info!("JSON generated !");
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
