use crate::cucumber_test_auxiliary_functions::{
    add_file_to_staging_area, commit_changes_to_repo, create_git_test_repository,
    create_test_commit,
};
use git2::{Repository, Signature};
use std::fs;
use std::fs::{create_dir, create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Project {
    pub relative_path_to_project: PathBuf,
    pub project_relative_path_to_analyzed_folder: PathBuf,
}

impl Project {
    pub(crate) fn new() -> Project {
        let relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("generated_project");

        let project = Project {
            relative_path_to_project: relative_path_to_project.clone(),
            project_relative_path_to_analyzed_folder: relative_path_to_project,
        };
        project.destroy();
        create_dir(&project.relative_path_to_project).unwrap();
        project
    }

    pub fn destroy(&self) {
        if self.relative_path_to_project.exists() {
            remove_dir_all(&self.relative_path_to_project).unwrap();
        }
    }

    pub(crate) fn init_git_repository(&self) {
        create_git_test_repository();
    }

    pub(crate) fn create_file(&self, filename: &String) {
        let file_in_project = self.relative_path_to_project.join(filename);
        if let Some(parent_dir) = file_in_project.parent() {
            create_dir_all(parent_dir).expect("Failed to create parent directory")
        }
        if !file_in_project.exists() {
            File::create(file_in_project).unwrap();
        }
    }

    pub fn write_lines_in_a_file(&self, file: PathBuf, lines_count: u32) {
        let file_in_project = self.relative_path_to_project.join(file);
        for _ in 0..lines_count {
            let mut file_to_modify = File::options()
                .create(true)
                .append(true)
                .open(&file_in_project)
                .unwrap();
            writeln!(&mut file_to_modify, "line").unwrap();
        }
    }

    pub(crate) fn get_a_contribution_in(&self, filename: &String, author: &Signature) {
        let repo = Repository::open(&self.relative_path_to_project).unwrap();
        let file_in_project = self.relative_path_to_project.join(filename);
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(file_in_project)
            .unwrap();
        writeln!(&mut file, "a").unwrap();

        add_file_to_staging_area(filename, &repo);
        commit_changes_to_repo(&repo, author);
    }
}
