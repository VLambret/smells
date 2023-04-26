use std::path::PathBuf;

pub trait IFileExplorer {
    fn discover(&self, root: &PathBuf) -> Vec<PathBuf>;
}

pub struct FakeFileExplorer {
    files_to_analyze: Vec<PathBuf>,
}

impl IFileExplorer for FakeFileExplorer {
    fn discover(&self, _root: &PathBuf) -> Vec<PathBuf> {
        self.files_to_analyze.clone()
    }
}

impl FakeFileExplorer {
    pub fn new(files_to_analyze: Vec<PathBuf>) -> Self {
        FakeFileExplorer { files_to_analyze }
    }
}

#[cfg(test)]
mod file_explorer_tests {
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use std::path::PathBuf;

    #[test]
    fn test_fake_file_explorer_with_empty_list_of_files_should_return_an_empty_list() {
        // Given
        let root = PathBuf::from("test_folder");
        let files_to_analyze = vec![];
        // When
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
        // Then
        let expected_files_to_analyze: Vec<PathBuf> = vec![];
        assert_eq!(fake_explorer1.discover(&root), expected_files_to_analyze)
    }

    #[test]
    fn test_fake_file_explorer_with_single_file_should_return_a_single_file() {
        // Given
        let root = PathBuf::from("test_folder");
        let files_to_analyze = vec![PathBuf::from("test_file")];
        // When
        let expected_files_to_analyze: Vec<PathBuf> = vec![PathBuf::from("test_file")];
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
        // Then
        assert_eq!(fake_explorer1.discover(&root), expected_files_to_analyze)
    }

    #[test]
    fn test_fake_file_explorer_with_multiple_files_should_return_all_files() {
        // Given
        let root = PathBuf::from("test_folder");
        let files_to_analyze = vec![PathBuf::from("test_file1"), PathBuf::from("test_file2")];
        // When
        let expected_files_to_analyze: Vec<PathBuf> =
            vec![PathBuf::from("test_file1"), PathBuf::from("test_file2")];
        let fake_explorer1 = FakeFileExplorer::new(files_to_analyze);
        // Then
        assert_eq!(fake_explorer1.discover(&root), expected_files_to_analyze);
    }
}
