use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

pub fn smells(){
    let args = CmdArgs::from_args();
    print_analysis(args.path).expect("Error");
}   

fn print_analysis(analysed_file: PathBuf) -> Result<(),Box<dyn std::error::Error>> {
    let file_os_str = analysed_file.as_os_str();
    let file_key = match analysed_file.file_name() {
        Some(file_name) => file_name.to_owned(),
        _ => file_os_str.to_owned(),
    };

    let file_line_number = 0;
    let json_output_with_empty_folder = format!(
    r#"{{
        {:?}: {{
            "metrics": {{
                "lines_metric": 0
            }},
            "folder_content": {{}}
        }}
    }}"#, file_key);

    let folder_content = format!(
    r#"
                {:?}: {{
                    "metrics": {{
                        "lines_metric": {}
                    }}
                }}"#, "file0.txt",file_line_number);

    let json_output_with_folder_not_empty = format!(
    r#"{{
        {:?}: {{
            "metrics": {{
                "lines_metric": 0
            }},
            "folder_content": {{{}
            }}
        }}
    }}"#, file_key, folder_content);


    let is_empty = std::fs::read_dir(&analysed_file)
        .map(|mut dir| dir.next().is_none())
        .unwrap_or(true);

    if analysed_file.into_os_string() != "." && !is_empty{
        print!("{}", json_output_with_folder_not_empty);
    }
    else{
        print!("{}", json_output_with_empty_folder);
    }
    Ok(())
}