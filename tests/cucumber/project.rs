use crate::cucumber_test_auxiliary_functions::create_git_test_repository;
use std::fs::{create_dir, remove_dir_all, File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use git2::Signature;

#[derive(Debug)]
pub struct Project {
    pub relative_path_to_project: PathBuf,
}

impl Project {
    pub(crate) fn get_contribution_to(&self, filename: String) {
        let file_in_project = self.relative_path_to_project.join(filename);
        let mut file = File::options()
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
        File::create(file_in_project).unwrap();
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
