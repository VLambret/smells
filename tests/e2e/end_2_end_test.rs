#[cfg(test)]
mod end_2_end_test{
    use assert_cmd::cmd::Command;
    use predicates::prelude::*;
    use serde_json::Value;
    use std::path::Path;
    //use super::EndToEndTest;

    fn string_to_json(expected_stdout: &str) -> Value {
        let expected_stdout_json: Value = serde_json::from_str(expected_stdout).unwrap();
        expected_stdout_json
    }

    fn stdout_to_json(cmd: &mut Command) -> Value {
        let actual_stdout = cmd.output().unwrap().stdout;
        let actual_stdout_str = String::from_utf8(actual_stdout).unwrap();
        let actual_stdout_json: Value = string_to_json(&actual_stdout_str);
        actual_stdout_json
    }

    #[test]
    #[ignore]
    fn without_argument_smells_analyses_current_folder() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.current_dir("tests/data/empty_folder");
        // then
        let expected_stdout = predicate::str::is_empty().not();
        cmd.assert().code(0).stdout(expected_stdout).stderr("");
        Ok(())
    }

    // TODO: For an empty folder, we cannot return a 0 metric because if we do that we cannot differentiate between an empty folder and a real 0 metric (like an empty file.
//       The correct solution is to return an error saying that the folder is empty, but we will define that when we handle errors.
    #[test]
    #[ignore]
    fn folder_to_analyse_can_be_specified_with_first_parameter(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = Path::new("tests")
            .join("data")
            .join("empty_folder")
            .display()
            .to_string();

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);
        let expected_stdout = r#"{
        "empty_folder": {
            "metrics": {
                "lines_count": 0,
                "social_complexity": 0
            },
            "folder_content": []
        }
    }"#;

        // then
        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        Ok(())
    }

    fn smells_can_count_lines_of_a_single_file() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = "tests/data/single_file_folder";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);

        // Then
        let expected_stdout = r#"{
        "single_file_folder": {
            "metrics": {
                "lines_count": 0,
                "social_complexity": 0
            },
            "folder_content": [
                {
                    "file0.txt": {
                        "metrics": {
                            "lines_count": 0,
                            "social_complexity": 0
                        }
                    }
                }
            ]
        }
    }"#;

        /*    cmd.assert()
        .code(0)
        .stdout(predicates::ord::eq(json_expected_stdout_to_str))
        .stderr("");*/

        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        println!("execution of single_file e2e test");
        Ok(())
    }

// inventory::submit!(EndToEndTest {
//     name: "basic",
//     test_fn: smells_can_count_lines_of_a_single_file
// });

    #[test]
    fn smells_can_count_lines_of_a_single_file_other_case() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = "tests/data/single_file_folder_other";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);

        //then
        let expected_stdout = r#"{
        "single_file_folder_other": {
            "metrics": {
                "lines_count": 5,
                "social_complexity": 0
            },
            "folder_content": [
                {
                    "file5.txt": {
                        "metrics": {
                            "lines_count": 5,
                            "social_complexity": 0
                        }
                    }
                }
            ]
        }
    }"#;

        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        Ok(())
    }

    #[test]
    fn smells_can_analyses_folder_with_multiple_files() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = "tests/data/folder_with_multiple_files";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);

        //then
        let expected_stdout = r#"{
        "folder_with_multiple_files":{
            "metrics": {
                "lines_count": 6,
                "social_complexity": 0
            },
            "folder_content":[
                {
                "file1.txt": {
                    "metrics": {
                        "lines_count": 1,
                        "social_complexity": 0
                    }
                }
                },
                {
                "file5.txt": {
                    "metrics": {
                        "lines_count": 5,
                        "social_complexity": 0
                    }
                }
                }
            ]
        }
    }"#;

        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        Ok(())
    }

    #[test]
    fn smells_can_analyses_folder_with_one_empty_folder() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = "tests/data/folder_with_one_empty_folder";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);

        //then
        let expected_stdout = r#"{
        "folder_with_one_empty_folder":{
            "metrics": {
                "lines_count": 0,
                "social_complexity": 0
            },
            "folder_content":[
                {
                "empty_folder":{
                    "metrics": {
                        "lines_count": 0,
                        "social_complexity": 0
                    },
                    "folder_content":[]
                }
           }
           ]
        }
    }"#;

        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        Ok(())
    }

    #[test]
    fn smells_can_analyses_folder_with_a_folder_and_a_file() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells";
        let args = "tests/data/folder_with_folder_and_file";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;
        cmd.args(&[args]);

        //then
        let expected_stdout = r#"{
        "folder_with_folder_and_file": {
            "metrics": {
                "lines_count": 11,
                "social_complexity": 0
            },
        "folder_content": [
            {
                "file1.txt": {
                    "metrics": {
                        "lines_count": 1,
                        "social_complexity": 0
                    }
                }
            },
            {
                "folder": {
                    "metrics": {
                        "lines_count": 10,
                        "social_complexity": 0
                    },
                "folder_content": [
                {
                    "file10.txt": {
                        "metrics": {
                            "lines_count": 10,
                            "social_complexity": 0
                        }
                    }
                }]
                }
            }]
        }
    }"#;

        let expected_stdout_json = string_to_json(expected_stdout);
        let actual_stdout_json = stdout_to_json(&mut cmd);
        assert_eq!(actual_stdout_json, expected_stdout_json);
        Ok(())
    }

    #[test]
    #[ignore]
    fn smells_must_not_access_to_a_file_with_no_permission() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells tests/data/folder_with_no_permission";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;

        //then
        let expected_stderr = "Error! Permission denied!";

        cmd.assert().code(0).stdout("").stderr(expected_stderr);
        Ok(())
    }

    #[test]
    #[ignore]
    fn with_two_arguments_smells_shows_an_error() -> Result<(), Box<dyn std::error::Error>> {
        // given
        let cmd_call = "smells . another_argument";

        // when
        let mut cmd = Command::cargo_bin(cmd_call)?;

        //then
        let expected_stderr = "Error! Argument number error!";

        cmd.assert().code(0).stdout("").stderr(expected_stderr);

        Ok(())
    }
}