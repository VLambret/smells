use assert_cmd::cmd::Command;
use predicates::prelude::*;

#[test]
#[ignore]
fn without_argument_smells_analyses_current_folder() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells";

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
fn with_argument_which_is_point_smells_analyses_current_folder() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells .";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    // then
    let expected_stdout = predicate::str::is_empty().not();
    cmd.assert()
        .code(0)
        .stdout(expected_stdout)
        .stderr("");
    Ok(())
}*/

#[test]
#[ignore]
fn folder_to_analyse_can_be_specified_with_first_parameter() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells tests/data/empty_folder";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    // then
    let expected_stdout = 
r#"[
    "empty_folder": {
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
fn smells_can_count_lines_of_a_single_file() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells tests/data/single_file_folder";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    //then
    let expected_stdout = 
r#"[
    "single_file.txt": {
        "metrics": {
            "lines_metric": 0,
        }
    },
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
fn smells_can_count_lines_of_a_single_file_other_case() -> Result<(), Box<dyn std::error::Error>>{
    // given
    let cmd_call = "smells tests/data/single_file_folder_other";

    // when
    let mut cmd = Command::cargo_bin(cmd_call)?;

    //then
    let expected_stdout = 
r#"[
    "single_file.txt": {
        "metrics": {
            "lines_metric": 10,
        }
    },
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

/*#[test]
#[ignore]
fn with_two_arguments_smells_show_an_error() -> Result<(), Box<dyn std::error::Error>>{
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
}*/