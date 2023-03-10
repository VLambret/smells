use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: std::path::PathBuf,
}

pub fn smells(){
    let args = CmdArgs::from_args();
    let json_output = format!(
r#"[
        "{}": {{
            "metrics": {{
                "lines_metric": 0,
            }},
            "folder_content": []
        }}
]"#, args.path.display());
    println!("{}", json_output);
}