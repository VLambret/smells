use crate::smells_steps::SmellsWorld;
use cucumber::World;

fn main() {
    /*    futures::executor::block_on(SmellsWorld::run(
        "tests/cucumber/features/basic_usages.feature",
    ));*/
    futures::executor::block_on(SmellsWorld::run(
        "tests/cucumber/features/social_complexity.feature",
    ));
}

/*************************************************************************************************************************/

#[cfg(test)]
mod smells_steps {
    use assert_cmd::Command;
    use cucumber::*;
    use predicates::boolean::PredicateBooleanExt;
    use predicates::prelude::predicate;
    use serde_json::Value;
    use std::fs::{create_dir, create_dir_all, remove_dir, File};
    use std::io::{stdout, Write};
    use std::path::PathBuf;

    #[derive(Debug, World)]
    pub struct SmellsWorld {
        analysed_folder: Vec<String>,
        cmd: Command,
    }

    impl Default for SmellsWorld {
        fn default() -> Self {
            SmellsWorld {
                analysed_folder: vec![],
                cmd: Command::cargo_bin("smells").expect("Failed to create Command"),
            }
        }
    }

    fn convert_stdout_to_json(cmd: &mut Command) -> Value {
        let actual_stdout = cmd.output().unwrap().stdout;
        let actual_stdout_str = String::from_utf8(actual_stdout).unwrap();
        convert_string_to_json(&actual_stdout_str)
    }

    fn convert_string_to_json(expected_stdout: &str) -> Value {
        match serde_json::from_str(expected_stdout) {
            Ok(json) => json,
            Err(err) => panic!("Failed to parse JSON: {}", err),
        }
    }

    /***********************************************************************************
     * BASIC USAGE
     **********************************************************************************/

    #[given(expr = "no program argument is provided")]
    fn no_argument_is_provided(w: &mut SmellsWorld) {
        w.analysed_folder = vec![];
    }

    #[given(regex = "arguments are \"(.+)\"")]
    fn arguments_exist(w: &mut SmellsWorld, file: String) {
        let existing_folder = PathBuf::from("tests").join("data").join("existing_folder");
        if !existing_folder.exists() {
            create_dir_all(existing_folder).unwrap();
        }
        let split_file_argument = file.split_whitespace();
        w.analysed_folder = split_file_argument.map(String::from).collect();
    }

    #[when(expr = "smells is called")]
    fn smells_called(w: &mut SmellsWorld) {
        w.cmd.args(&w.analysed_folder);
    }

    #[then(regex = "exit code is (.+)")]
    fn exit_code_is_a_number(w: &mut SmellsWorld, code_number: i32) {
        w.cmd.assert().code(code_number);
    }

    #[then(regex = "standard output is \"(.+)\"")]
    fn stdout_is_empty(w: &mut SmellsWorld, empty: String) {
        if empty == "empty" {
            w.cmd.assert().stdout(predicate::str::is_empty());
        } else {
            w.cmd.assert().stdout(predicate::str::is_empty().not());
        }
    }

    #[then(regex = "standard output contains \"(.+)\"")]
    fn stdout_contains_message(w: &mut SmellsWorld, message: String) {
        w.cmd.assert().stdout(predicate::str::contains(message));
    }

    //TODO: find a way to handle fr/en
    #[then(regex = "standard error contains \"(.+)\"")]
    fn stderr_contains_message(w: &mut SmellsWorld, message: String) {
        let french_message = String::from("Le fichier spécifié est introuvable.");
        if message == "No such file or directory" {
            w.cmd.assert().stderr(
                predicate::str::contains(message).or(predicate::str::contains(french_message)),
            );
        } else {
            w.cmd.assert().stderr(predicate::str::contains(message));
        }
    }

    #[then("standard error is empty")]
    fn stderr_is_empty(w: &mut SmellsWorld) {
        w.cmd.assert().stderr(predicate::str::is_empty());
    }

    /***********************************************************************************
     * SOCIAL COMPLEXITY
     **********************************************************************************/

    #[given(expr = "analysed folder is not a git repository")]
    fn step_analysed_folder_is_not_a_git_repository(w: &mut SmellsWorld) {
        let analyzed_folder = PathBuf::from("tests")
            .join("data")
            .join("non_git_repository");
        if !analyzed_folder.exists() {
            create_dir(&analyzed_folder).unwrap();
        };
        let mut file = File::create(PathBuf::from(&analyzed_folder).join("file5.txt")).unwrap();
        for _n in 0..4 {
            file.write_all(b"Line").unwrap()
        }
        w.analysed_folder = vec![analyzed_folder.to_string_lossy().to_string()];
    }

    #[then(regex = "the warning \"(.+)\" is raised")]
    fn step_warning_is_raised(w: &mut SmellsWorld, warning: String) {
        w.cmd.assert().stderr(predicate::str::contains("WARN:"));
        w.cmd.assert().stderr(predicate::str::contains(warning));
    }

    #[then(regex = "no social complexity metric is computed")]
    fn step_social_complexity_metric_is_not_computed(w: &mut SmellsWorld) {
        let analysis_result = convert_stdout_to_json(&mut w.cmd);
        /*if let Some(metrics) = analysis_result[w.analysed_folder[0].clone()]["metrics"].as_object() {
            assert!(!metrics.contains_key("social_complexity"));
        }*/
        let analysed_folder = PathBuf::from(w.analysed_folder[0].clone());
        let analysed_folder_file_name = analysed_folder.file_name().unwrap();
        if let Some(analysis_fields) =
            analysis_result.get(analysed_folder_file_name.to_string_lossy().to_string())
        {
            if let Some(metrics) = analysis_fields.get("metrics") {
                if let Some(social_complexity) = metrics.get("social_complexity") {
                    assert!(false)
                } else {
                    assert!(true)
                }
            } else {
                assert!(false)
            }
        } else {
            assert!(false)
        }
    }

    /*    C:\Users\Lucas\git\smells\tests\data\non_git_repository
     */

    /***********************************************************************************
     * TEARDOWN
     **********************************************************************************/

    /*
    fn teardown(w: &mut SmellsWorld) {
            w.analysed_folder.iter().for_each(|folder| {
                let path = PathBuf::from(folder);
                remove_dir(path).unwrap();
            });
    }*/
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
