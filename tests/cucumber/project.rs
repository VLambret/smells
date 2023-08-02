use std::fs::{create_dir, File, remove_dir_all};
use std::io::Write;
use std::path::PathBuf;
use crate::cucumber_test_annex_functions::create_git_test_repository;

#[derive(Debug)]
pub struct Project {
    pub relative_path_to_project: PathBuf
}

impl Project {
    pub(crate) fn create_file(&self, filename: String) {
        File::create(filename);
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

        let project = Project { relative_path_to_project };
        project.destroy();
        create_dir(&project.relative_path_to_project).unwrap();
        project
    }
}