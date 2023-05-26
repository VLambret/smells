use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub trait IFileExplorer: Iterator<Item = PathBuf> {
    fn discover(&self) -> Vec<PathBuf>;
}

pub struct FileExplorer {
    root: PathBuf,
}

// TODO: (Vec<PathBuf>, Vec<std::io::Error>)
impl IFileExplorer for FileExplorer {
    fn discover(&self) -> Vec<PathBuf> {
        Self::discover_inner(&self.root)
    }
}

impl FileExplorer {
    pub fn new(root: &Path) -> Self {
        FileExplorer {
            root: root.to_path_buf(),
        }
    }

    fn discover_inner(root: &PathBuf) -> Vec<PathBuf> {
        let mut files = vec![];
        for file in read_dir(root).unwrap() {
            let file = file.unwrap().path();
            if file.is_file() {
                files.push(file);
            } else {
                files.extend(Self::discover_inner(&file));
            }
        }
        files
    }
}

impl Iterator for FileExplorer {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        Some(PathBuf::from(""))
    }
}

pub struct FakeFileExplorer {
    files_to_analyze: Vec<PathBuf>,
}

impl IFileExplorer for FakeFileExplorer {
    fn discover(&self) -> Vec<PathBuf> {
        self.files_to_analyze.clone()
    }
}

impl Iterator for FakeFileExplorer {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        self.files_to_analyze.pop()
    }
}

#[cfg(test)]
mod file_explorer_tests {
    use crate::data_sources::file_explorer::{FakeFileExplorer, FileExplorer, IFileExplorer};
    use std::collections::HashSet;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;

    impl FakeFileExplorer {
        pub fn new(files_to_analyze: Vec<PathBuf>) -> Self {
            FakeFileExplorer { files_to_analyze }
        }
    }

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
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();

        // When
        let actual_files = FileExplorer::new(&root).discover();

        // Then
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
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let file1 = root.join("file1.txt");
        File::create(&file1).unwrap();

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
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let file1 = root.join("file1.txt");
        let file2 = root.join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

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
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let subfolder = root.join("subfolder");
        let file1 = subfolder.join("file1.txt");

        fs::create_dir(&subfolder).unwrap();
        File::create(&file1).unwrap();

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
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let dir1 = root.join("dir1");
        let file1 = dir1.join("file1.txt");

        let dir2 = root.join("dir2");
        let dir21 = dir2.join("dir21");
        let file2 = dir21.join("file2.txt");

        fs::create_dir(&dir1).unwrap();
        fs::create_dir(&dir2).unwrap();
        fs::create_dir(&dir21).unwrap();

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        // When
        let actual_files = FileExplorer::new(&root).discover();

        // Then
        let expected_files: Vec<PathBuf> = vec![file1, file2];
        assert_eq!(actual_files, expected_files);
    }

    #[test]
    fn test_fake_file_explorer_with_empty_list_of_files_should_return_an_empty_list() {
        // Given
        let files_to_analyze = vec![];
        // When
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
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
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
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
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
        // Then
        assert_eq!(fake_explorer1.discover(), expected_files_to_analyze);
    }
}
