use std::fs::{create_dir, File};
use std::io::Write;
use std::path::PathBuf;
use crate::cucumber_test_annex_functions::create_git_test_repository;

#[derive(Debug)]
pub struct Project {
    pub relative_path_to_project: PathBuf
}

impl Project {
    pub(crate) fn init_git_repository(&self) {
        create_git_test_repository();
    }
}

impl Project {
    pub(crate) fn new() -> Project {
        let relative_path_to_project = PathBuf::from("tests")
            .join("data")
            .join("non_git_repository");
        if !relative_path_to_project.exists() {
            create_dir(&relative_path_to_project).unwrap();
        };
        let mut file =
            File::create(PathBuf::from(&relative_path_to_project).join("file5.txt")).unwrap();
        for _n in 0..4 {
            file.write_all(b"Line\n").unwrap()
        };
        Project { relative_path_to_project }
    }
}