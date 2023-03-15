use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

pub struct AnalysisResult {
    file: PathBuf
}

pub fn smells(){
    let args = CmdArgs::from_args();
    do_analysis(args.path);
}  

fn do_analysis(folder: PathBuf){
    let analysis = analyse(folder);
    print_analysis(analysis);
}

fn analyse(folder: PathBuf) -> AnalysisResult {
    AnalysisResult{
        file: folder
    }
}

fn extract_key(file: PathBuf) -> String{
    let file_os_str = file.as_os_str();
    let file_key = match file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => file_os_str.to_owned(),
    };
    file_key.to_string_lossy().into_owned()
}

fn print_analysis(analysis: AnalysisResult) {
    let file_key = extract_key(analysis.file);

    let json_output_with_empty_folder = format!(
        r#"{{
        {:?}: {{
            "metrics": {{
                "lines_metric": 0
            }},
            "folder_content": {{}}
        }}
    }}"#, file_key);  
    print!("{}", json_output_with_empty_folder)
}