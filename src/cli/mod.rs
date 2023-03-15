use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

pub struct Analysis {
    analysed_file: PathBuf
}

pub fn smells(){
    let args = CmdArgs::from_args();
    do_analysis(args.path).expect("Error");
}   

fn do_analysis(analysed_file: PathBuf) -> Result<(),Box<dyn std::error::Error>> {

    let file_line_number = 0;

    let analysis = Analysis{
        analysed_file
    };
    print_analysis(analysis);   
    Ok(())
}

fn print_analysis(analysis: Analysis) {
    let file_os_str = analysis.analysed_file.as_os_str();
    let file_key = match analysis.analysed_file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => file_os_str.to_owned(),
    };

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