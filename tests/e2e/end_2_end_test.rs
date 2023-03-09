use assert_cmd::cmd::Command;

/*macro_rules! concatenate {
    ($suffix:literal $($s:literal)*) => {
        concat!($suffix $(, '\n', $s)*)
    };
}*/

#[test]
#[ignore]
fn should_return_empty_json_if_in_current_empty_dir_with_no_argument() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "cli";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    // then
    let expected_stdout = 
r#"[
        ".": {
            "metrics": {
                "lines_metric": 0,
            },
            "folder_content": []
        }
]
"#;
    cmd.assert()
        .code(0)
        .stdout(expected_stdout)
        .stderr("");

    Ok(())
}

/*#[test]
#[ignore]
fn should_return_file0_with_0_line_in_current_directory() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "cli";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    //then
    let expected_stdout = 
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
]
"#;  

    cmd.assert()
    .code(0)
    .stdout(expected_stdout)
    .stderr("");  


    Ok(())
}*/