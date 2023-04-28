use std::fs::read_dir;
use std::io;
use std::path::PathBuf;

pub trait IFileExplorer {
    fn discover(&self, root: &PathBuf) -> Vec<PathBuf>;
}

pub struct FileExplorer {}

// TODO: (Vec<PathBuf>, Vec<std::io::Error>)
impl IFileExplorer for FileExplorer {
    fn discover(&self, root: &PathBuf) -> Vec<PathBuf> {
        let mut files = vec![];
        for file in read_dir(root).unwrap() {
            let file = file.unwrap().path();
            if file.is_file() {
                files.push(file);
            } else {
                files.extend(self.discover(&file));
            }
        }
        files
    }
}

impl FileExplorer {
    pub fn new() -> Self {
        FileExplorer {}
    }
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
    use crate::data_sources::file_explorer::{FakeFileExplorer, FileExplorer, IFileExplorer};
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;

    fn create_fake_file_tree(root: &PathBuf) {
        let dir1 = root.join("dir1");
        let file1 = dir1.join("file1.txt");

        let dir2 = root.join("dir2");
        let dir21 = dir2.join("dir21");
        let file2 = dir21.join("file2.txt");

        fs::create_dir(&dir1).unwrap();
        fs::create_dir(&dir2).unwrap();
        fs::create_dir(&dir21).unwrap();

        let mut f1 = File::create(&file1).unwrap();
        let mut f2 = File::create(&file2).unwrap();
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
        let actual_files = FileExplorer::new().discover(&root);

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
        let actual_files = FileExplorer::new().discover(&root);

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
        let actual_files = FileExplorer::new().discover(&root);

        // Then
        let expected_files: Vec<PathBuf> = vec![file1, file2];
        assert_eq!(actual_files, expected_files);
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
        let actual_files = FileExplorer::new().discover(&root);

        // Then
        let expected_files: Vec<PathBuf> = vec![file1];
        assert_eq!(actual_files, expected_files);
    }

    /*    #[test]
    fn file_explorer_with_root_path_should_return_a_vector_of_the_files_path() {
        // Given
        let root = PathBuf::from("tests").join("data");
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir(&root).unwrap();
        create_fake_file_tree(&root);

        // When
        let actual_files = FileExplorer::new().discover(&root);

        // Then
        let file1 = PathBuf::from(&root).join("dir1").join("file1.txt");
        let file2 = PathBuf::from(&root).join("dir2").join("dir21").join("file2.txt");
        let expected_files : Vec<PathBuf> = vec![file1, file2];
        assert_eq!(actual_files, expected_files);

    }*/

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
