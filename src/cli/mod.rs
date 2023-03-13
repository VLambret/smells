use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: std::path::PathBuf,
}

pub fn smells(){
    let args = CmdArgs::from_args();
    print_analysis(args.path);
}   

fn print_analysis(analysed_file: std::path::PathBuf){
    let file_key = match analysed_file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => analysed_file.into_os_string(),
    };

    let json_output = format!(
    r#"[
        {:?}: {{
            "metrics": {{
                "lines_metric": 0,
            }},
            "folder_content": []
        }}
    ]"#, file_key);
    print!("{}", json_output);
}