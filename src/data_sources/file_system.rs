use std::fs::File;
use std::path::PathBuf;

pub fn get_file_from_path(path: &PathBuf) -> File{
    // TODO: handle unwrap()
    File::open(path).unwrap()
}