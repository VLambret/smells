use assert_cmd::cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn without_argument_smells_analyses_current_folder() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    // then
    let expected_stdout = 
    r#"{
        ".": {
        "metrics": {
                "lines_metric": 0
            },
            "folder_content": []
        }
    }"#;
    
    let json_expected_stdout: Value = serde_json::from_str(expected_stdout).unwrap();
    let json_expected_stdout_to_str = serde_json::to_string_pretty(&json_expected_stdout).unwrap();

    cmd.assert()
        .code(0)
        .stdout(json_expected_stdout_to_str)
        .stderr("");
    Ok(())
}

#[test]
fn with_argument_which_is_point_smells_analyses_current_folder() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";
    let args = ".";
    //let args = &[".", ".."];

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;
    cmd.args(&[args]);

    // then
    let expected_stdout = predicate::str::is_empty().not();
    cmd.assert()
        .code(0)
        .stdout(expected_stdout)
        .stderr("");
    Ok(())
}

#[test]
fn folder_to_analyse_can_be_specified_with_first_parameter() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";
    let args = "tests/data/empty_folder";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;
    cmd.args(&[args]);

    // then
    let expected_stdout =
    r#"{
        "empty_folder": {
            "metrics": {
                "lines_metric": 0
            },
            "folder_content": []
        }
    }"#;

    let json_expected_stdout: Value = serde_json::from_str(expected_stdout).unwrap();
    let json_expected_stdout_to_str = serde_json::to_string_pretty(&json_expected_stdout).unwrap();

    cmd.assert()
        .code(0)
        .stdout(json_expected_stdout_to_str)
        .stderr("");
    Ok(())
}

#[test]
fn smells_can_count_lines_of_a_single_file() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";
    let args = "tests/data/single_file_folder";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;
    cmd.args(&[args]);

    //then
    let expected_stdout = 
    r#"{
        "single_file_folder": {
            "metrics": {
                "lines_metric": 0
            },
            "folder_content": [
                {
                    "file0.txt": {
                        "metrics": {
                            "lines_metric": 0
                        }
                    }
                }
            ]
        }
    }"#;

    let json_expected_stdout: Value = serde_json::from_str(expected_stdout).unwrap();
    let json_expected_stdout_to_str = serde_json::to_string_pretty(&json_expected_stdout).unwrap();

    cmd.assert()
    .code(0)
    .stdout(predicates::ord::eq(json_expected_stdout_to_str))
    .stderr("");  

    Ok(())
}

#[test]
fn smells_can_count_lines_of_a_single_file_other_case() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";
    let args = "tests/data/single_file_folder_other";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;
    cmd.args(&[args]);

    //then
    let expected_stdout = 
    r#"{
        "single_file_folder_other": {
            "metrics": {
                "lines_metric": 5
            },
            "folder_content": [
                {
                    "file5.txt": {
                        "metrics": {
                            "lines_metric": 5
                        }
                    }
                }
            ]
        }
    }"#;  

    let json_expected_stdout: Value = serde_json::from_str(expected_stdout).unwrap();
    let json_expected_stdout_to_str = serde_json::to_string_pretty(&json_expected_stdout).unwrap();

    cmd.assert()
    .code(0)
    .stdout(json_expected_stdout_to_str)
    .stderr(""); 

    Ok(())
}

#[test]
#[ignore]
fn smells_can_analyses_folder_with_multiple_files() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";
    let args = "tests/data/folder_with_multiple_files";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;
    cmd.args(&[args]);

    //then
    let expected_stdout =
    r#"{
        "folder_with_multiple_files":{
            "metrics": {
                "lines_metric": 6
            },
            "folder_content":[
                {
                "file1.txt": {
                    "metrics": {
                        "lines_metric": 1
                    }
                }
                },
                {
                "file5.txt": {
                    "metrics": {
                        "lines_metric": 5
                    }
                }
                }
            ]
        }
    }"#;  

    let json_expected_stdout: Value = serde_json::from_str(expected_stdout).unwrap();
    let json_expected_stdout_to_str = serde_json::to_string_pretty(&json_expected_stdout).unwrap();

    cmd.assert()
    .code(0)
    .stdout(json_expected_stdout_to_str)
    .stderr("");
    Ok(())
}

#[test]
#[ignore]
fn smells_can_analyses_folder_with_a_folder_and_a_file() -> Result<(), Box<dyn std::error::Error>>{
        // given
        let cmd_call = "smells tests/data/folder_with_multiple_files";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
    
        //then
        let expected_stdout =
r#"{
    "folder_with_folder_and_file":{
        "metrics": {
            "lines_metric": 11,
    },
    "folder_content":[
        "file1.txt": {
            "metrics": {
                "lines_metric": 1,
            }
        },
        "folder": {
            "metrics": {
                "lines_metric": 10,
            }
            "folder_content":[
                "file10.txt": {
                    "metrics": {
                        "lines_metric": 10,
                    }
                }
            ]
        }

    ]
    }"#;
    cmd.assert()
    .code(0)
    .stdout(expected_stdout)
    .stderr("");

    Ok(())
}

#[test]
#[ignore]
fn smells_must_not_access_to_a_file_with_no_permission() -> Result<(), Box<dyn std::error::Error>>{
       // given
       let cmd_call = "smells tests/data/folder_with_no_permission";

       // when
       let mut cmd = Command::cargo_bin(cmd_call)?;

       //then
       let excpected_stderr = "Error! Permission denied!";

       cmd.assert()
       .code(0)
       .stdout("")
       .stderr(excpected_stderr);
    Ok(())
}

#[test]
#[ignore]
fn with_two_arguments_smells_shows_an_error() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells . another_argument";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    //then
    let expected_stderr = "Error! Argument number error!";

       cmd.assert()
       .code(0)
       .stdout("")
       .stderr(expected_stderr);

    Ok(())
}