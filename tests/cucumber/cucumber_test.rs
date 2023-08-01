use assert_cmd::Command;
use cucumber::{given, World};
use env_logger::Env;
use futures::FutureExt;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, thread};

#[derive(Debug, World)]
pub struct SmellsWorld {
    initial_wd: PathBuf,
    relative_path_to_project: PathBuf,
    cmd: Command,
}

impl Default for SmellsWorld {
    fn default() -> SmellsWorld {
        SmellsWorld {
            initial_wd: PathBuf::new(),
            relative_path_to_project: PathBuf::new(),
            cmd: Command::cargo_bin("smells").expect("Failed to create Command"),
        }
    }
}

fn main() {
    /*    futures::executor::block_on(SmellsWorld::run(
        "tests/cucumber/features/basic_usages.feature",
    ));*/
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    /*   futures::executor::block_on(SmellsWorld::run(
        "tests/cucumber/features/social_complexity.feature",
    ));*/

    //TODO:teardown

    futures::executor::block_on(
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
                set_current_dir(&world.unwrap().initial_wd).unwrap();
                let sleep_duration = Duration::from_millis(300);
                let sleep_future = async move {
                    thread::sleep(sleep_duration);
                }
                .boxed();
                sleep_future
            })
            .run_and_exit("tests/cucumber/features/social_complexity.feature"),
    );
}

/*************************************************************************************************************************/

#[cfg(test)]
mod smells_steps {
    use super::*;
    use cucumber::*;
    use git2::{Commit, Repository, Signature, Tree};
    use log::warn;
    use predicates::boolean::PredicateBooleanExt;
    use predicates::prelude::predicate;
    use serde_json::Value;
    use std::env::{current_dir, set_current_dir};
    use std::fs::{create_dir, create_dir_all, remove_dir_all, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::{assert_eq, env, panic, vec};

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

    #[when(regex = "smells is called with \"(.*)\"")]
    fn smells_called(w: &mut SmellsWorld, arguments: String) {
        let argv = arguments.split_whitespace();
        let change_of_working_directory = set_current_dir(&w.relative_path_to_project);
        if change_of_working_directory.is_ok() {
            w.cmd.args(argv);
        } else {
            warn!("Change of working directory failed");
        }
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

    //	Scenario: Analyse a non-git repository

    #[given(expr = "project is not a git repository")]
    fn step_project_is_not_a_git_repository(w: &mut SmellsWorld) {
        w.relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("non_git_repository");
        if !w.relative_path_to_project.exists() {
            create_dir(&w.relative_path_to_project).unwrap();
        };
        let mut file =
            File::create(PathBuf::from(&w.relative_path_to_project).join("file5.txt")).unwrap();
        for _n in 0..4 {
            file.write_all(b"Line\n").unwrap()
        }
    }

    #[then(regex = "the warning \"(.+)\" is raised")]
    fn step_warning_is_raised(w: &mut SmellsWorld, warning: String) {
        w.cmd.assert().stderr(predicate::str::contains("WARN:"));
        w.cmd.assert().stderr(predicate::str::contains(warning));
    }

    #[then(regex = "no social complexity metric is computed")]
    fn step_social_complexity_metric_is_not_computed(w: &mut SmellsWorld) {
        let analysis_result = convert_stdout_to_json(&mut w.cmd);
        let analysed_folder = w.relative_path_to_project.clone();
        let analysed_folder_file_name = analysed_folder.file_name().unwrap();

        let social_complexity_field = analysis_result
            .get(analysed_folder_file_name.to_string_lossy().to_string())
            .and_then(|analysis_fields| analysis_fields.get("metrics"))
            .and_then(|metrics| metrics.get("social_complexity"));
        assert!(social_complexity_field.is_none())
    }

    //	Scenario: Analyse a git repository without any contributors

    fn create_git_test_repository() -> Repository {
        let repo = current_dir().unwrap().join(
            PathBuf::from("tests")
                .join("data")
                .join("git_repository_social_complexity"),
        );
        if repo.exists() {
            remove_dir_all(&repo).unwrap();
        }
        create_dir_all(&repo).unwrap();
        Repository::init(repo).unwrap()
    }

    #[given(expr = "project is a git repository")]
    fn step_project_is_a_git_repository(w: &mut SmellsWorld) {
        create_git_test_repository()
            .path()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();

        w.relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("git_repository_social_complexity");
    }

    #[given(expr = "there is no contributor")]
    fn step_no_contributors(w: &mut SmellsWorld) {}

    #[then(expr = "no warning is raised")]
    fn step_no_warning_is_raised(w: &mut SmellsWorld) {
        w.cmd
            .assert()
            .stderr(predicate::str::contains("WARN:").not());
    }

    // 	Scenario: Analyse a git repository with contributors

    #[given(regex = "(.+) contributed to (.+)")]
    fn step_contributor_to_file(w: &mut SmellsWorld, contributor: String, file: String) {
        let repo = Repository::open(PathBuf::from(&w.initial_wd).join(&w.relative_path_to_project))
            .unwrap();
        let contributor_signature = Signature::now(&contributor, "mail").unwrap();
        update_file(&repo, &file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo, &contributor_signature);
    }

    #[then(regex = "(.+) social complexity score is (.+)")]
    fn step_social_complexity_score(w: &mut SmellsWorld, file: String, score: String) {
        let analysis_result = convert_stdout_to_json(&mut w.cmd);
        let analysed_folder = w.relative_path_to_project.clone();
        let analysed_folder_file_name = PathBuf::from(analysed_folder.file_name().unwrap());
        let file_full_path = analysed_folder_file_name.join(file);

        assert_eq!(
            get_social_complexity_score(file_full_path, &analysis_result),
            score.parse::<i32>().unwrap()
        );
    }

    fn get_social_complexity_score(file_path: PathBuf, analysis: &Value) -> Value {
        let file_components: Vec<String> = file_path
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect();

        match file_components.as_slice() {
            [file_name] => {
                if let Some(file_fields) = analysis.get(file_name) {
                    if let Some(metrics) = file_fields.get("metrics") {
                        if let Some(score) = metrics.get("social_complexity") {
                            return score.clone();
                        }
                    }
                }
            }
            [first_dir, other_dirs @ ..] => {
                let mut results = serde_json::Map::new();

                if let Value::Object(obj) = analysis {
                    if let Some(current_level_folder_content) = obj
                        .get(first_dir)
                        .and_then(|fields| fields.get("folder_content_analyses"))
                    {
                        if let Value::Array(arr) = current_level_folder_content {
                            for item in arr {
                                if let Value::Object(obj) = item {
                                    results.extend(obj.clone());
                                }
                            }
                        }
                    }
                }
                let other_dirs_pathbuf = other_dirs.iter().collect::<PathBuf>();
                return get_social_complexity_score(other_dirs_pathbuf, &Value::Object(results));
            }
            _ => {}
        }
        Value::Null
    }

    fn update_file(repo: &Repository, file: &String) {
        let file_path = repo.path().parent().unwrap().join(file);

        if let Some(parent_dir) = file_path.parent() {
            create_dir_all(parent_dir).expect("Failed to create parent directory")
        }

        let mut file = File::options()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();
        writeln!(&mut file, "a").unwrap();
    }

    fn add_file_to_the_staging_area(repo: &Repository, file: String) {
        let mut index = repo.index().unwrap(); // index = staging_area
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }

    fn commit_changes_to_repo(repo: &Repository, author: &Signature) {
        match repo.head() {
            Ok(head) => {
                let parent = repo.find_commit(head.target().unwrap()).unwrap();
                let tree = repo
                    .find_tree(repo.index().unwrap().write_tree().unwrap())
                    .unwrap();
                let parents = &[&parent];
                create_test_commit(repo, author, &tree, parents);
            }
            Err(_) => {
                let tree_id = {
                    let mut index = repo.index().unwrap();
                    index.write_tree().unwrap()
                };
                let tree = repo.find_tree(tree_id).unwrap();
                let parents = &[];
                create_test_commit(repo, author, &tree, parents);
            }
        }
    }
    fn create_test_commit(repo: &Repository, author: &Signature, tree: &Tree, parents: &[&Commit]) {
        repo.commit(
            Some("HEAD"),
            author,
            author,
            "Commit message",
            tree,
            parents,
        )
        .unwrap();
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
