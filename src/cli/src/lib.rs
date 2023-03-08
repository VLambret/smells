pub fn cli(){
    //let file = std::fs::read_dir(".");

    let json_file =
r#"[
        ".": {
            "metrics": {
                "lines_metric": 0,
            },
            "folder_content": []
        }
]"#;
    println!("{}", json_file);
}