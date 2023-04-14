use std::collections::HashSet;
use std::path::PathBuf;

fn collect_files(path: &PathBuf, root: &PathBuf) -> HashSet<String> {
    let mut files = HashSet::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();
        if entry_path.is_file() {
            let relative_file_path = entry_path.strip_prefix(root).unwrap().to_owned();
            let full_file_path = relative_file_path.to_string_lossy().to_string();
            files.insert(full_file_path);
        } else if entry_path.is_dir() {
            files.extend(collect_files(&entry_path, root));
        }
    }
    files
}

fn collect_directories(path: &PathBuf, root: &PathBuf) -> HashSet<String> {
    let mut directories = HashSet::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();
        if entry_path.is_dir() {
            let relative_file_path = entry_path.strip_prefix(root).unwrap().to_owned();
            let full_dir_path = relative_file_path.to_string_lossy().to_string();
            directories.insert(full_dir_path);
            directories.extend(collect_directories(&entry_path, root));
        }
    }
    directories
}

mod tests{
    use super::*;
    const PATH_TO_DIR: &str = "tests/data/file_system";

    #[test]
    fn walk_through_directories_and_get_files(){

        let mut expected_output_files = HashSet::new();
        expected_output_files.insert("file3.txt".to_string());
        expected_output_files.insert("subfolder1/file1.txt".to_string());
        expected_output_files.insert("subfolder2/file2.txt".to_string());

        let actual_files = collect_files(&PathBuf::from(PATH_TO_DIR), &PathBuf::from(PATH_TO_DIR));
        println!("{:?}", actual_files);
        assert_eq!(actual_files, expected_output_files);
    }

    #[test]
    fn walk_through_directories_and_get_directories(){
        let mut expected_output_dir = HashSet::new();
        expected_output_dir.insert("folder1".to_string());
        expected_output_dir.insert("subfolder1".to_string());
        expected_output_dir.insert("subfolder2".to_string());
        expected_output_dir.insert("folder1/folder_in_folder1".to_string());

        let actual_dir = collect_directories(&PathBuf::from(PATH_TO_DIR), &PathBuf::from(PATH_TO_DIR));
        println!("{:?}", actual_dir);
        assert_eq!(actual_dir, expected_output_dir);
    }
}