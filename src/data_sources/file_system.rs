use std::collections::HashSet;
use std::path::PathBuf;

fn _file_system(path: PathBuf) -> HashSet<PathBuf>{
    let collection_of_files_and_folders_path = _collect_entries(&path, &path);
    println!("{:?}", collection_of_files_and_folders_path);
    collection_of_files_and_folders_path
}

// TODO: handle unwrap()
fn _collect_entries(path: &PathBuf, root: &PathBuf) -> HashSet<PathBuf>{
    let mut entries = HashSet::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();
        let relative_entry_path = entry_path.strip_prefix(root).unwrap().to_owned();
        if entry_path.is_file() {
            entries.insert(relative_entry_path);
        }
        else{
            entries.insert(relative_entry_path);
            entries.extend(_collect_entries(&entry_path, root));
        }
    }
    entries
}

/* mod tests{
    use super::*;
    const _PATH_TO_DIR: &str = "tests/data/file_system";

    #[test]
    fn test_file_system(){
        let mut expected_output = HashSet::new();
        expected_output.insert(PathBuf::from("file3.txt"));
        expected_output.insert(PathBuf::from("subfolder1/file1.txt"));
        expected_output.insert(PathBuf::from("subfolder2/file2.txt"));
        expected_output.insert(PathBuf::from("folder1"));
        expected_output.insert(PathBuf::from("subfolder1"));
        expected_output.insert(PathBuf::from("subfolder2"));
        expected_output.insert(PathBuf::from("folder1/folder_in_folder1"));

        let actual_dir = _file_system(PathBuf::from(_PATH_TO_DIR));
        assert_eq!(actual_dir, expected_output);
    }
} */