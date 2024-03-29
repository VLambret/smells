use glob::{glob_with, MatchOptions};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub trait IFileExplorer: Debug {
    fn discover(&self) -> Vec<PathBuf>;
    fn get_root(&self) -> PathBuf;
}
#[derive(Debug, Clone)]
pub struct FileExplorer {
    root: PathBuf,
}

// TODO: (Vec<PathBuf>, Vec<std::io::Error>)
impl IFileExplorer for FileExplorer {
    fn discover(&self) -> Vec<PathBuf> {
        Self::discover_inner(&self.root)
    }

    fn get_root(&self) -> PathBuf {
        self.root.clone()
    }
}
impl FileExplorer {
    pub fn new(root: &Path) -> Self {
        FileExplorer {
            root: root.to_path_buf(),
        }
    }
    fn discover_inner(root: &PathBuf) -> Vec<PathBuf> {
        let file_discovery_pattern = format!("{}/**/*", root.to_string_lossy().to_string());

        let glob_options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: true,
        };

        glob_with(&file_discovery_pattern, glob_options)
            .unwrap()
            .filter_map(|file| match file {
                Ok(f) if f.is_file() => Some(f),
                _ => None,
            })
            .collect()
    }
}

impl Iterator for FileExplorer {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        Some(PathBuf::from(""))
    }
}

#[derive(Debug)]
pub struct FakeFileExplorer {
    files_to_analyze: Vec<PathBuf>,
}
impl FakeFileExplorer {
    pub fn _new(files_to_analyze: Vec<PathBuf>) -> Self {
        FakeFileExplorer { files_to_analyze }
    }
}

impl IFileExplorer for FakeFileExplorer {
    fn discover(&self) -> Vec<PathBuf> {
        self.files_to_analyze.clone()
    }

    fn get_root(&self) -> PathBuf {
        PathBuf::from("root")
    }
}
impl Iterator for FakeFileExplorer {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.files_to_analyze.pop()
    }
}
#[cfg(test)]
pub mod file_explorer_tests {
    use crate::data_sources::file_explorer::{FakeFileExplorer, FileExplorer, IFileExplorer};
    use maplit::btreemap;
    use std::collections::{BTreeMap, HashSet};
    use std::fs::{create_dir, create_dir_all, remove_dir_all, File};
    use std::path::PathBuf;

    fn assert_contains_same_items(actual_files: Vec<PathBuf>, expected_files: Vec<PathBuf>) {
        let left: HashSet<&PathBuf> = HashSet::from_iter(expected_files.iter());
        let right: HashSet<&PathBuf> = HashSet::from_iter(actual_files.iter());
        assert_eq!(left, right);
    }
    #[test]
    fn file_explorer_with_root_path_without_files_should_return_an_empty_vector() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("file_explorer")
            .join("empty_root");
        if root.exists() {
            remove_dir_all(&root).unwrap();
        }
        create_dir_all(&root).unwrap(); // When
        let actual_files = FileExplorer::new(&root).discover(); // Then
        let expected_files: Vec<PathBuf> = vec![];
        assert_eq!(actual_files, expected_files);
    }
    #[test]
    fn file_explorer_with_root_path_with_1_file_should_return_the_path_of_the_file() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("file_explorer")
            .join("root_with_1_file");
        if !root.exists() {
            create_dir_all(&root).unwrap();
        }
        let file1 = root.join("file1.txt");
        if !file1.exists() {
            File::create(&file1).unwrap();
        }

        // When
        let actual_files = FileExplorer::new(&root).discover();

        // Then
        let expected_files: Vec<PathBuf> = vec![file1];
        assert_eq!(actual_files, expected_files);
    }
    #[test]
    fn file_explorer_with_root_path_with_2_files_should_return_the_paths_of_the_files() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("file_explorer")
            .join("root_with_2_files");
        if !root.exists() {
            create_dir_all(&root).unwrap();
        }

        let file1 = root.join("file1.txt");
        let file2 = root.join("file2.txt");

        if !file1.exists() {
            File::create(&file1).unwrap();
        }

        if !file2.exists() {
            File::create(&file2).unwrap();
        }

        // When
        let actual_files = FileExplorer::new(&root).discover();

        // Then
        let expected_files: Vec<PathBuf> = vec![file1, file2];
        assert_contains_same_items(actual_files, expected_files);
    }
    #[test]
    fn file_explorer_with_root_path_with_1_subfolder_and_1_file_should_return_the_path_of_the_file()
    {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("file_explorer")
            .join("root_with_1_folder_and_1_file");
        if !root.exists() {
            create_dir_all(&root).unwrap();
        }
        let subfolder = root.join("subfolder");
        let file1 = subfolder.join("file1.txt");
        if !subfolder.exists() {
            create_dir(&subfolder).unwrap();
        }

        if !file1.exists() {
            File::create(&file1).unwrap();
        }

        // When
        let actual_files = FileExplorer::new(&root).discover();

        // Then
        let expected_files: Vec<PathBuf> = vec![file1];
        assert_eq!(actual_files, expected_files);
    }
    #[test]
    fn file_explorer_with_root_path_should_return_a_vector_of_the_files_path() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("file_explorer")
            .join("folder_with_tree");

        if !root.exists() {
            create_dir_all(&root).unwrap();
        }

        let dir1 = root.join("dir1");
        let file1 = dir1.join("file1.txt");
        let dir2 = root.join("dir2");
        let dir21 = dir2.join("dir21");
        let file2 = dir21.join("file2.txt");

        if !dir1.exists() {
            create_dir(&dir1).unwrap();
        }
        if !dir21.exists() {
            create_dir_all(&dir21).unwrap();
        }

        if !file1.exists() {
            File::create(&file1).unwrap();
        }
        if !file2.exists() {
            File::create(&file2).unwrap();
        }

        // When
        let files = FileExplorer::new(&root).discover();
        let actual_files: BTreeMap<_, _> = files
            .iter()
            .map(|file| (file.to_string_lossy().to_string(), file))
            .collect();

        // Then
        let expected_files = btreemap! {file1.to_string_lossy().to_string() => &file1,
        file2.to_string_lossy().to_string() => &file2};
        assert_eq!(actual_files, expected_files);
    }
    #[test]
    fn test_fake_file_explorer_with_empty_list_of_files_should_return_an_empty_list() {
        // Given
        let files_to_analyze = vec![];
        // When
        let fake_explorer1 = FakeFileExplorer::_new(files_to_analyze);
        // Then
        let expected_files_to_analyze: Vec<PathBuf> = vec![];
        assert_eq!(fake_explorer1.discover(), expected_files_to_analyze)
    }
    #[test]
    fn test_fake_file_explorer_with_single_file_should_return_a_single_file() {
        // Given
        let files_to_analyze = vec![PathBuf::from("test_file")];
        // When
        let expected_files_to_analyze: Vec<PathBuf> = vec![PathBuf::from("test_file")];
        let fake_explorer1 = FakeFileExplorer::_new(files_to_analyze);
        // Then
        assert_eq!(fake_explorer1.discover(), expected_files_to_analyze)
    }
    #[test]
    fn test_fake_file_explorer_with_multiple_files_should_return_all_files() {
        // Given
        let files_to_analyze = vec![PathBuf::from("test_file1"), PathBuf::from("test_file2")];
        // When
        let expected_files_to_analyze: Vec<PathBuf> =
            vec![PathBuf::from("test_file1"), PathBuf::from("test_file2")];
        let fake_explorer1 = FakeFileExplorer::_new(files_to_analyze);
        // Then
        assert_eq!(fake_explorer1.discover(), expected_files_to_analyze);
    }
}
