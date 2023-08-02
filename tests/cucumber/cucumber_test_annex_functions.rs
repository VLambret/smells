use git2::{Commit, Repository, Signature, Tree};
use serde_json::Value;
use std::env::current_dir;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{Result, Write};
use std::path::PathBuf;

pub fn convert_std_to_json(cmd_output_std: Vec<u8>) -> Value {
    let stdout_str = String::from_utf8(cmd_output_std.to_owned()).unwrap();
    convert_string_to_json(&stdout_str)
}

pub fn convert_std_to_string(cmd_output_std: Vec<u8>) -> String {
    String::from_utf8(cmd_output_std).unwrap()
}

pub fn convert_string_to_json(expected_stdout: &str) -> Value {
    match serde_json::from_str(expected_stdout) {
        Ok(json) => json,
        Err(err) => panic!("Failed to parse JSON: {}", err),
    }
}

pub fn create_git_test_repository() -> Repository {
    let repo = current_dir().unwrap().join(
        PathBuf::from("tests")
            .join("data")
            .join("generated_project"),
    );
    if repo.exists() {
        remove_dir_all(&repo).unwrap();
    }
    create_dir_all(&repo).unwrap();
    Repository::init(repo).unwrap()
}

pub fn get_social_complexity_score(file_path: PathBuf, analysis: &Value) -> Value {
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

pub fn update_file(repo: &Repository, file: &String) {
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

pub fn add_file_to_the_staging_area(repo: &Repository, file: String) {
    let mut index = repo.index().unwrap(); // index = staging_area
    index.add_path(&PathBuf::from(file)).unwrap();
    index.write().unwrap();
}

pub fn commit_changes_to_repo(repo: &Repository, author: &Signature) {
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

pub fn create_test_commit(repo: &Repository, author: &Signature, tree: &Tree, parents: &[&Commit]) {
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
