use structopt::StructOpt;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::collections::HashMap;

#[derive(Debug, StructOpt)]
pub struct CmdArgs{
    #[structopt(default_value=".")]
    path: PathBuf,
}

#[derive(Deserialize, Serialize)]
enum FileType{
    File(FileEntry),
    Folder(FolderEntry),
}

#[derive(Deserialize, Serialize)]
struct Metrics{
    lines_metrics: u32
}

#[derive(Deserialize, Serialize)]
struct FileEntry{
    metrics: Metrics
}

#[derive(Deserialize, Serialize)]
struct FolderEntry{
    metrics: Metrics,
    folder_content: HashMap<String, FileType>,
}

pub fn smells(){
    let args = CmdArgs::from_args();
    //print_analysis(args.path).expect("Error");

    let file1_metrics = Metrics{
        lines_metrics: 0,
    };

    let file2_metrics = Metrics{
        lines_metrics: 2,
    };

    let file1_entry = FileEntry{
        metrics: file1_metrics,
    };

    let file2_entry = FileEntry{
        metrics: file2_metrics,
    };

    let folder1_metrics = Metrics{
        lines_metrics: 0,
    };

    let mut folder1_entry = FolderEntry{
        metrics: folder1_metrics,
        folder_content: HashMap::new(),
    };

    let mut json_data = HashMap::new();
    json_data.insert("file1".to_string(), FileType::File(file1_entry));
    folder1_entry.folder_content.insert("file2".to_string(), FileType::File(file2_entry));
    json_data.insert("folder1".to_string(), FileType::Folder(folder1_entry));

    let mut file = std::fs::File::options()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open("json_output_test_file.json").unwrap();

    let output = serde_json::to_string(&json_data).unwrap();
    file.write(output.as_bytes()).unwrap();
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