use crate::basic_usage_test::SmellsBasicWorld;
use cucumber::World;

fn main() {
    futures::executor::block_on(SmellsBasicWorld::run(
        "tests/cucumber/features/basic_usages.feature",
    ));
}

/*************************************************************************************************************************/

#[cfg(test)]
mod basic_usage_test {
    use assert_cmd::Command;
    use cucumber::{given, then, when, World};
    use predicates::boolean::PredicateBooleanExt;
    use predicates::prelude::predicate;
    use std::path::PathBuf;

    #[derive(Debug, World)]
    pub struct SmellsBasicWorld {
        files: Vec<String>,
        cmd: Command,
    }

    impl Default for SmellsBasicWorld {
        fn default() -> Self {
            SmellsBasicWorld {
                files: vec![],
                cmd: Command::cargo_bin("smells").expect("Failed to create Command"),
            }
        }
    }

    #[given(expr = "no program argument is provided")]
    fn no_argument_is_provided(w: &mut SmellsBasicWorld) {
        w.files = vec![];
    }

    #[given(regex = "arguments are \"(.+)\"")]
    fn arguments_exist(w: &mut SmellsBasicWorld, file: String) {
        if file == "existing_folder" {
            w.files = vec![PathBuf::from("tests")
                .join("data")
                .join("single_file_folder")
                .join("file0.txt")
                .to_string_lossy()
                .to_string()];
        } else {
            let split_file_argument = file.split_whitespace();
            w.files = split_file_argument.map(String::from).collect();
        }
    }

    #[when(expr = "smells is called")]
    fn smells_called(w: &mut SmellsBasicWorld) {
        w.cmd.args(&w.files);
    }

    #[then(regex = r"exit code is (.+)")]
    fn exit_code_is_a_number(w: &mut SmellsBasicWorld, code_number: i32) {
        w.cmd.assert().code(code_number);
    }

    #[then(regex = "standard output is \"(.+)\"")]
    fn stdout_is_empty(w: &mut SmellsBasicWorld, empty: String) {
        if empty == "empty" {
            w.cmd.assert().stdout(predicate::str::is_empty());
        } else {
            w.cmd.assert().stdout(predicate::str::is_empty().not());
        }
    }

    //TODO: find a way to handle fr/en
    #[then(regex = "standard error contains \"(.+)\"")]
    fn stderr_contains_usage(w: &mut SmellsBasicWorld, message: String) {
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
    fn stderr_is_empty(w: &mut SmellsBasicWorld) {
        w.cmd.assert().stderr(predicate::str::is_empty());
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
