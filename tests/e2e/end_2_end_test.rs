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
    let cmd_call = "smells";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    // then
    let expected_stdout = r#"[
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

#[test]
#[ignore]
fn should_return_file0_with_0_line_in_current_dir() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";

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
}

/*#[test]
#[ignore]
fn should_return_file1_with_1_line_in_current_dir() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";

    // when
    let mut _cmd = Command::cargo_bin(cmd_call)?;

    //then
    let _expected_stdout = 
    r#"[
        "file1": {
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
    ]
    "#;




    Ok(())
}*/