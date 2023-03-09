pub fn smells(){

    let _json_file_empty_dir =
r#"[
        ".": {
            "metrics": {
                "lines_metric": 0,
            },
            "folder_content": []
        }
]"#;

    let _json_file_file0 =
r#"[
    "file0": {
        "metrics": {
            "lines_metric": 0,
        }
    },
    "folder1": {
        "metrics": {
            "lines_metric": 0,
        },
        "folder_content": []
    }
]"#;

    let _json_file_file1 =
r#"[
    "file0": {
        "metrics": {
            "lines_metric": 1,
        }
    },
    "folder1": {
        "metrics": {
            "lines_metric": 0,
        },
        "folder_content": []
    }
]"#;

    let json_file_file10 =
r#"[
    "file0": {
        "metrics": {
            "lines_metric": 10,
        }
    },
    "folder1": {
        "metrics": {
            "lines_metric": 0,
        },
        "folder_content": []
    }
]"#;


    println!("{}", json_file_file10);
}