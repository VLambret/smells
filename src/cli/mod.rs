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
    let json_output = format!(
r#"[
        "{}": {{
            "metrics": {{
                "lines_metric": 0,
            }},
            "folder_content": []
        }}
]"#, analysed_file.display());
    println!("{}", json_output);       
}