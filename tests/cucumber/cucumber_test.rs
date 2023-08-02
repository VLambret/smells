mod cucumber_test_annex_functions;
mod project;

use assert_cmd::Command;
use cucumber::{given, World};
use env_logger::Env;
use futures::FutureExt;
use log::info;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::process::{exit, Output};
use std::time::Duration;
use std::{env, io, thread};
use crate::project::Project;

#[derive(Debug, World)]
pub struct SmellsWorld {
    project: Project,
    initial_wd: PathBuf,
    relative_path_to_project: PathBuf,
    cmd: Command,
    cmd_output: Option<io::Result<Output>>,
}

impl Default for SmellsWorld {
    fn default() -> SmellsWorld {
        SmellsWorld {
            project: Project::new(),
            initial_wd: PathBuf::new(),
            relative_path_to_project: PathBuf::new(),
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
       "tests/cucumber/features/social_complexity.feature"
   ];

    let mut error_number = 0;

    for feature in feature_files {
        error_number += run_feature_file(feature);
    }

    if (error_number != 0)
    {
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
    result.steps_stats().failed
}

/*************************************************************************************************************************/

#[cfg(test)]
mod smells_steps {
    use super::*;
    use crate::cucumber_test_annex_functions::*;
    use cucumber::*;
    use git2::{Repository, Signature, Tree};
    use log::warn;
    use std::env::{current_dir, set_current_dir};
    use std::fs::{create_dir, create_dir_all, remove_dir_all, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::{assert_eq, panic, vec};

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

        if change_of_working_directory.is_ok() {
            w.cmd_output = Some(w.cmd.args(argv).output());
        } else {
            warn!("Change of working directory failed");
        }
    }

    #[then(regex = "exit code is (.+)")]
    fn exit_code_is_a_number(w: &mut SmellsWorld, code_number: i32) {
        assert!(&w.cmd_output.is_some());
        if let Some(Ok(output)) = &w.cmd_output {
            assert_eq!(output.status.code(), Some(code_number));
        }
    }

    #[then(regex = "standard output is (.+)")]
    fn stdout_is_empty(w: &mut SmellsWorld, empty: String) {
        assert!(w.cmd_output.is_some() && w.cmd_output.as_ref().unwrap().is_ok());
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        if empty == "empty" {
            assert!(output.stdout.is_empty());
        } else {
            assert!(!output.stdout.is_empty());
        }
    }

    #[then(regex = "standard output contains \"(.+)\"")]
    fn stdout_contains_message(w: &mut SmellsWorld, message: String) {
        assert!(w.cmd_output.is_some() && w.cmd_output.as_ref().unwrap().is_ok());
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let stdout: String = convert_std_to_string(output.stdout);
        assert!(stdout.contains(&message));
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
        assert!(stderr.is_empty());
    }

    /***********************************************************************************
     * SOCIAL COMPLEXITY
     **********************************************************************************/

    //	Scenario: Analyse a non-git repository

    #[given(expr = "project is not a git repository")]
    fn step_project_is_not_a_git_repository(w: &mut SmellsWorld) {
    }

    #[then(regex = "the warning \"(.+)\" is raised")]
    fn step_warning_is_raised(w: &mut SmellsWorld, warning: String) {
        if let Some(Ok(output)) = &w.cmd_output {
            let stderr_str = String::from_utf8_lossy(&*output.stderr);
            assert!(stderr_str.contains("WARN:"));
            assert!(stderr_str.contains(&warning));
        } else {
            assert!(false);
        }
    }

    #[then(regex = "no social complexity metric is computed")]
    fn step_social_complexity_metric_is_not_computed(w: &mut SmellsWorld) {
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let analysis_result = convert_std_to_json(output.stdout);
        let analysed_folder = w.project.relative_path_to_project.clone();
        let analysed_folder_file_name = analysed_folder.file_name().unwrap();

        let social_complexity_field = analysis_result
            .get(analysed_folder_file_name.to_string_lossy().to_string())
            .and_then(|analysis_fields| analysis_fields.get("metrics"))
            .and_then(|metrics| metrics.get("social_complexity"));
        assert!(social_complexity_field.is_none())
    }

    //	Scenario: Analyse a git repository without any contributors
    #[given(expr = "project is a git repository")]
    fn step_project_is_a_git_repository(w: &mut SmellsWorld) {
        w.project.init_git_repository();

        w.project.relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("git_repository_social_complexity");
    }

    #[given(expr = "there is no contributor")]
    fn step_no_contributors(w: &mut SmellsWorld) {}

    #[then(expr = "no warning is raised")]
    fn step_no_warning_is_raised(w: &mut SmellsWorld) {
        if let Some(Ok(output)) = &w.cmd_output {
            let stderr_str = String::from_utf8_lossy(&*output.stderr);
            assert!(!stderr_str.contains("WARN:"));
        } else {
            assert!(false);
        }
    }

    // 	Scenario: Analyse a git repository with contributors

    #[given(regex = "(.+) contributed to (.+)")]
    fn step_contributor_to_file(w: &mut SmellsWorld, contributor: String, file: String) {
        let repo = Repository::open(PathBuf::from(&w.initial_wd).join(&w.project.relative_path_to_project))
            .unwrap();
        let contributor_signature = Signature::now(&contributor, "mail").unwrap();
        update_file(&repo, &file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo, &contributor_signature);
    }

    #[then(regex = "(.+) social complexity score is (.+)")]
    fn step_social_complexity_score(w: &mut SmellsWorld, file: String, score: String) {
        let output = w.cmd_output.as_ref().unwrap().as_ref().cloned().unwrap();
        let analysis_result = convert_std_to_json(output.stdout);
        let analysed_folder = w.project.relative_path_to_project.clone();
        let analysed_folder_file_name = PathBuf::from(analysed_folder.file_name().unwrap());
        let file_full_path = analysed_folder_file_name.join(file);

        assert_eq!(
            get_social_complexity_score(file_full_path, &analysis_result),
            score.parse::<i32>().unwrap()
        );
    }
}

#[cfg(test)]
mod end_2_end_test {
    use assert_cmd::cmd::Command;
    use cucumber::gherkin::Step;
    use cucumber::{given, then, when, World};
    use serde_json::Value;

    #[derive(Debug, World)]
    pub struct SmellsWorld {
        file: String,
        cmd: Command,
    }

    impl Default for SmellsWorld {
        fn default() -> Self {
            SmellsWorld {
                file: String::default(),
                cmd: Command::cargo_bin("smells").expect("Failed to create Command"),
            }
        }
    }

    fn convert_string_to_json(expected_stdout: &str) -> Value {
        match serde_json::from_str(expected_stdout) {
            Ok(json) => json,
            Err(err) => panic!("Failed to parse JSON: {}", err),
        }
    }

    fn convert_stdout_to_json(cmd: &mut Command) -> Value {
        let actual_stdout = cmd.output().unwrap().stdout;
        let actual_stdout_str = String::from_utf8(actual_stdout).unwrap();
        convert_string_to_json(&actual_stdout_str)
    }

    #[given(regex = r"a path (.+)")]
    fn a_folder_with_an_empty_file(smells: &mut SmellsWorld, path: String) {
        smells.file = path;
    }

    #[when("I run the analysis of the folder")]
    fn run_analysis(smells: &mut SmellsWorld) {
        smells.cmd.args([&smells.file]);
    }

    #[then("smells will show the json result of the analysis")]
    fn test_result(smells: &mut SmellsWorld, step: &Step) {
        let expected_stdout_json = convert_string_to_json(&step.docstring.clone().unwrap());
        let actual_stdout_json = convert_stdout_to_json(&mut smells.cmd);
        assert_eq!(expected_stdout_json, actual_stdout_json);
    }
}

/*************************************************************************************************************************/

/*#[cfg(test)]
mod analysis_unit_test {
    use cucumber::gherkin::Step;
    use cucumber::{given, then, when, World};
    use serde_json::Value;
    use smells::analysis_module::analysis::TopAnalysis;
    use smells::data_sources::file_explorer::{FakeFileExplorer, FileExplorer, IFileExplorer};
    use smells::metrics::metric::IMetric;
    use std::fmt::Debug;
    use std::path::{Path, PathBuf};

    #[derive(Debug, World)]
    pub struct AnalysisWorld {
        root: PathBuf,
        file_explorer: Box<dyn IFileExplorer>,
        metrics: Vec<Box<dyn IMetric>>,
        actual_analysis: TopAnalysis,
    }

    impl Default for AnalysisWorld {
        fn default() -> Self {
            AnalysisWorld {
                root: PathBuf::from("root"),
                file_explorer: Box::new(FileExplorer::new(Path::new("root"))),
                metrics: vec![],
                actual_analysis: TopAnalysis {
                    file_name: Default::default(),
                    metrics: Default::default(),
                    folder_content: None,
                },
            }
        }
    }

    // param "without metrics" => vec![]
    // "with metrics" => vec![lc, sc]
    /*#[given("an empty folder without metrics")]
    fn analysis_ut_empty_root(analysis: &mut AnalysisWorld) {
        analysis.file_explorer = Box::new(FakeFileExplorer::new(vec![]));
    }

    #[given("a two files folder without metrics")]
    fn analysis_ut_two_files_folder(analysis: &mut AnalysisWorld) {
        let files_to_analyse = vec![
            PathBuf::from(&analysis.root).join("file1"),
            PathBuf::from(&analysis.root).join("file2"),
        ];
        analysis.file_explorer = Box::new(FakeFileExplorer::new(files_to_analyse));
    }

    // do_internal_analysis should be private
    #[when("I do the internal analysis")]
    fn run_the_analysis(analysis: &mut AnalysisWorld) {
        analysis.actual_analysis =
            do_internal_analysis(&*analysis.file_explorer, &analysis.metrics);
    }

    #[then("analysis module will build this analysis")]
    fn check_result(analysis: &mut AnalysisWorld, step: &Step) {
        let actual_result_analysis: Value =
            serde_json::to_value(&analysis.actual_analysis).unwrap();
        let expected_result_analysis: Value =
            serde_json::from_str(&step.docstring.clone().unwrap()).unwrap();
        assert_eq!(expected_result_analysis, actual_result_analysis);
    }*/
}*/
