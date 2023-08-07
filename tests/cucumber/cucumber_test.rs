mod cucumber_test_auxiliary_functions;
mod project;

use crate::project::Project;
use assert_cmd::Command;
use cucumber::{given, World};
use env_logger::Env;
use futures::FutureExt;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::process::{exit, Output};
use std::time::Duration;
use std::{env, io, thread};

#[derive(Debug, World)]
pub struct SmellsWorld {
    project: Project,
    initial_wd: PathBuf,
    cmd: Command,
    cmd_output: Option<io::Result<Output>>,
}

impl Default for SmellsWorld {
    fn default() -> SmellsWorld {
        SmellsWorld {
            project: Project::new(),
            initial_wd: PathBuf::new(),
            cmd: Command::cargo_bin("smells").expect("Failed to create Command"),
            cmd_output: None,
        }
    }
}

impl SmellsWorld {
    fn teardown(&self) {
        //TODO: suppression of test folders can't be done because files are used by another process (step ?)

        /* let full_project_path = self.initial_wd.join(&self.relative_path_to_project);
        if full_project_path.exists() {
            remove_dir_all(&full_project_path).unwrap();
        } else {
        }*/
        set_current_dir(&self.initial_wd).unwrap();
    }
}

fn main() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let feature_files = [
        "tests/cucumber/features/basic_usages.feature",
        "tests/cucumber/features/social_complexity.feature",
        "tests/cucumber/features/lines_count.feature",
    ];

    let mut error_number = 0;

    for feature in feature_files {
        error_number += run_feature_file(feature);
    }

    if error_number != 0 {
        exit(42);
    }
}

fn run_feature_file(feature_file: &str) -> usize {
    let result = futures::executor::block_on(
        SmellsWorld::cucumber()
            .before(|_feature, _rule, _scenario, world| {
                world.initial_wd = env::current_dir().unwrap();
                let sleep_duration = Duration::from_millis(300);
                let sleep_future = async move {
                    thread::sleep(sleep_duration);
                }
                .boxed();
                sleep_future
            })
            .after(|_feature, _rule, _scenario, _ev, world| {
                world.unwrap().teardown();
                let sleep_duration = Duration::from_millis(300);
                let sleep_future = async move {
                    thread::sleep(sleep_duration);
                }
                .boxed();
                sleep_future
            })
            .fail_on_skipped()
            .run(feature_file),
    );
    result.steps_stats().to_owned().failed
}

/*************************************************************************************************************************/

#[cfg(test)]
mod smells_steps {
    use super::*;
    use crate::cucumber_test_auxiliary_functions::*;
    use cucumber::*;
    use git2::{Signature};
    use log::warn;
    use serde_json::Value::Null;
    use std::env::set_current_dir;
    use std::{assert_eq};

    /***********************************************************************************
     * BASIC USAGE
     **********************************************************************************/

    #[when(regex = "smells is called with \"(.*)\"")]
    fn smells_called(w: &mut SmellsWorld, arguments: String) {
        let argv = arguments.split_whitespace();

        let change_of_working_directory = if w.project.relative_path_to_project != PathBuf::new() {
            set_current_dir(&w.project.relative_path_to_project)
        } else {
            Ok(())
        };

        if arguments.contains(&"./".to_string()) {
            let path = PathBuf::from(&arguments);
            let path_without_point = path.strip_prefix("./").unwrap();
            w.project.project_relative_path_to_analyzed_folder = PathBuf::from(path_without_point);
        }

        if change_of_working_directory.is_ok() {
            w.cmd_output = Some(w.cmd.args(argv).output());
        } else {
            warn!("Change of working directory failed");
        }
    }

    #[then(regex = "exit code is (.+)")]
    fn exit_code_is_a_number(w: &mut SmellsWorld, code_number: i32) {
        if let Some(Ok(output)) = &w.cmd_output {
            assert_eq!(output.status.code(), Some(code_number));
        } else {
            assert!(false)
        }
    }

    #[then(regex = "standard output is (.+)")]
    fn stdout_is_empty(w: &mut SmellsWorld, empty: String) {
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        if empty == "empty" {
            assert!(output.stdout.is_empty(), "stdout contains {:?}", convert_std_to_string(output.stdout));
        } else {
            assert!(!output.stdout.is_empty());
        }
    }

    #[then(regex = "standard output contains \"(.+)\"")]
    fn stdout_contains_message(w: &mut SmellsWorld, message: String) {
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let stdout: String = convert_std_to_string(output.stdout);
        assert!(stdout.contains(&message), "stdout contains {:?}", &stdout);
    }

    //TODO: find a way to handle fr/en
    #[then(regex = "standard error contains \"(.+)\"")]
    fn stderr_contains_message(w: &mut SmellsWorld, message: String) {
        let french_message = String::from("Le fichier spécifié est introuvable.");
        assert!(w.cmd_output.is_some() && w.cmd_output.as_ref().unwrap().is_ok());
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let stderr: String = convert_std_to_string(output.stderr);

        if message == "No such file or directory" {
            assert!(stderr.contains(&message) || stderr.contains(&french_message))
        } else {
            assert!(stderr.contains(&message));
        }
    }

    #[then("standard error is empty")]
    fn stderr_is_empty(w: &mut SmellsWorld) {
        let stderr = convert_std_to_string(
            w.cmd_output
                .as_ref()
                .unwrap()
                .as_ref()
                .cloned()
                .unwrap()
                .stderr,
        );
        assert!(stderr.is_empty(), "stderr contains: {:?}", &stderr);

    }

    /***********************************************************************************
     * FILES
     **********************************************************************************/

    #[given(regex = "(.+) is created")]
    fn step_file_is_created(w: &mut SmellsWorld, file: String) {
        w.project.create_file(&file);
    }

    #[given(expr = "a project")]
    fn smells_existing_project(_w: &mut SmellsWorld) {}

    #[given(regex = "file (.+) is deleted")]
    fn step_delete_file(w: &mut SmellsWorld, file: String) {
        w.project.remove_file(PathBuf::from(&file));
    }

    #[given(expr = "the project is empty")]
    fn step_project_empty(_w: &mut SmellsWorld) {}




    /***********************************************************************************
     * METRICS
     **********************************************************************************/

    #[then(regex = "(.+) (.+) score is (.+)")]
    fn step_get_metric_score(w: &mut SmellsWorld, file: String, metric_key: String, score: String) {
        let analysis_result = get_json_analysis(&w.cmd_output);
        //TODO: find robust solution
        if file.contains(
            &w.project
                .project_relative_path_to_analyzed_folder
                .to_string_lossy()
                .to_string(),
        ) {
            assert_eq!(
                get_metric_score(PathBuf::from(&file), &analysis_result, &metric_key),
                score.parse::<i32>().unwrap()
            );
        } else {
            let filename = get_filename_for_analysis(
                &w.project.project_relative_path_to_analyzed_folder,
                &file,
            );
            assert_eq!(
                get_metric_score(filename, &analysis_result, &metric_key),
                score.parse::<i32>().unwrap()
            );
        }
    }

    #[then(regex = "(.+) has no (.+) score")]
    fn step_no_metric_score(w: &mut SmellsWorld, file: String, metric_key: String) {
        let analysis_result = get_json_analysis(&w.cmd_output);
        let filename = get_filename_for_analysis(&w.project.relative_path_to_project, &file);

        assert_eq!(
            get_metric_score(filename, &analysis_result, &metric_key),
            Null
        );
    }

    /***********************************************************************************
     * LINES COUNT
     **********************************************************************************/

    // Analyse an empty file
    #[given(regex = "(.+) lines are added to (.+)")]
    fn step_add_lines_to_file(w: &mut SmellsWorld, lines_count: String, file: String) {
        w.project
            .write_lines_in_a_file(PathBuf::from(file), lines_count.parse::<u32>().unwrap())
    }

    /***********************************************************************************
     * SOCIAL COMPLEXITY
     **********************************************************************************/

    //	Scenario: Analyse a non-git repository

    #[given(expr = "project is not a git repository")]
    fn step_project_is_not_a_git_repository(_w: &mut SmellsWorld) {}

    #[then(regex = "the warning \"(.+)\" is raised")]
    fn step_warning_is_raised(w: &mut SmellsWorld, warning: String) {
        if let Some(Ok(output)) = &w.cmd_output {
            let stderr_str = String::from_utf8_lossy(&*output.stderr);
            assert!(stderr_str.contains("WARN:") && stderr_str.contains(&warning), "stderr contains : {:?}", &stderr_str);
        } else {
            assert!(false);
        }
    }

    #[then(regex = "no (.+) metric is computed")]
    fn step_metric_is_not_computed(w: &mut SmellsWorld, metric_key: String) {
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let analysis_result = convert_std_to_json(output.stdout);
        let analysed_folder = w.project.relative_path_to_project.clone();
        let analysed_folder_file_name = analysed_folder.file_name().unwrap();

        let metric_field = analysis_result
            .get(analysed_folder_file_name.to_string_lossy().to_string())
            .and_then(|analysis_fields| analysis_fields.get("metrics"))
            .and_then(|metrics| metrics.get(metric_key));
        assert!(metric_field.is_none())
    }

    //	Scenario: Analyse a git repository without any contributors
    #[given(expr = "project is a git repository")]
    fn step_project_is_a_git_repository(w: &mut SmellsWorld) {
        w.project.init_git_repository();
    }

    #[given(expr = "there is no contributor")]
    fn step_no_contributors(_w: &mut SmellsWorld) {}

    #[then(expr = "no warning is raised")]
    fn step_no_warning_is_raised(w: &mut SmellsWorld) {
        if let Some(Ok(output)) = &w.cmd_output {
            let stderr_str = String::from_utf8_lossy(&*output.stderr);
            assert!(!stderr_str.contains("WARN:"), "stderr contains: {:?}", stderr_str);
        } else {
            assert!(false);
        }
    }

    // 	Scenario: Analyse a git repository with contributors

    #[given(regex = "(.+) contributed to (.+)")]
    fn step_contributor_to_file(w: &mut SmellsWorld, contributor: String, file: String) {
        let contributor_signature = Signature::now(&contributor, "mail").unwrap();
        w.project.create_file(&file);
        w.project
            .get_a_contribution_in(&file, &contributor_signature);
    }
}
