use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: String,
}

pub fn smells(){
    let args = CmdArgs::from_args();
    let json_file_empty_dir = format!(
r#"[
        "{}": {{
            "metrics": {{
                "lines_metric": 0,
            }},
            "folder_content": []
        }}
]"#, args.path);
    println!("{}", json_file_empty_dir);
}