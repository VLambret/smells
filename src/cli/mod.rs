use structopt::StructOpt;
use std::ffi::OsStr;

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
        Some(file_name) => file_name.to_string_lossy().into_owned(),
        _ => analysed_file
            .to_str()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| OsStr::new("foo").to_string_lossy().into_owned()),
    };

    let json_output = format!(
    r#"[
        "{}": {{
            "metrics": {{
                "lines_metric": 0,
            }},
            "folder_content": []
        }}
    ]"#, file_key);
    println!("{}", json_output);
}