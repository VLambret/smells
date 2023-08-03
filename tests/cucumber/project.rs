use crate::cucumber_test_auxiliary_functions::create_git_test_repository;
use git2::{Repository, Signature};
use std::fs::{create_dir, create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Project {
    pub relative_path_to_project: PathBuf,
}

impl Project {
    pub(crate) fn add_file_to_staging_area(&self, filename: &String, repo: &Repository) {
        let mut index = repo.index().unwrap();
        index.add_path(&PathBuf::from(filename)).unwrap();
        index.write().unwrap();
    }
}

impl Project {
    pub(crate) fn get_contribution_in(&self, filename: String) {
        let file_in_project = self.relative_path_to_project.join(filename);
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(file_in_project)
            .unwrap();
        writeln!(&mut file, "a").unwrap();
    }
}

impl Project {
    pub(crate) fn create_file(&self, filename: String) {
        let file_in_project = self.relative_path_to_project.join(filename);
        dbg!(&file_in_project);
        if let Some(parent_dir) = file_in_project.parent() {
            create_dir_all(parent_dir).expect("Failed to create parent directory")
        }
        if !file_in_project.exists() {
            File::create(file_in_project).unwrap();
        }
    }
}

impl Project {
    pub(crate) fn init_git_repository(&self) {
        create_git_test_repository();
    }

    pub fn destroy(&self) {
        if self.relative_path_to_project.exists() {
            remove_dir_all(&self.relative_path_to_project).unwrap();
        }
    }
}

impl Project {
    pub(crate) fn new() -> Project {
        let relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("generated_project");

        let project = Project {
            relative_path_to_project,
        };
        project.destroy();
        create_dir(&project.relative_path_to_project).unwrap();
        project
    }
}
