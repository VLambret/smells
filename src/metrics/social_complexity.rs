use std::path::PathBuf;
use git2::Repository;

pub fn social_complexity(_root_path: &str) -> u32{
    return 0;
}

fn get_number_of_authors_of_a_file(repo: &Repository, file: &PathBuf) -> u32{
    let relative_file_path = get_relative_path(&repo,&file);
    let blame = repo.blame_file(&relative_file_path, None).unwrap();
    let authors: Vec<String> = blame.iter().map(|blame_hunk| blame_hunk.final_signature().name().unwrap().to_owned()).collect();
    println!("{:?}", authors);
    authors.len() as u32
}

fn get_relative_path(repo: &Repository, path: &PathBuf) -> PathBuf{
    let mut relative_path = path.clone();
    if path.is_absolute(){
        relative_path = path.strip_prefix(repo.path()).unwrap().to_path_buf();
    }
    relative_path
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::fs::File;
    use git2::{Signature, Time};
    use rstest::rstest;
    use tempdir::TempDir;
    use std::io::Write;

    //TODO: generation auteur + utiliser le path du repo et pas le repo direct
    // + folder concret dans tests/data
    #[rstest(file, expected_authors,
    case("file1.txt", 1),
    //case("file2.txt", 2)
    )]
    fn smells_get_number_of_authors_of_a_file(file: &str, expected_authors: u32){
        let repo = routine1(file);
        let committed_file_path = repo.path().join(file);
        let numbers_of_authors_of_specified_file = get_number_of_authors_of_a_file(&repo, &committed_file_path);
        assert_eq!(numbers_of_authors_of_specified_file, expected_authors);
    }

    fn routine1(file: &str) -> Repository{
        //let temp_git_repo = create_temp_folder();
        let temp_git_repo = create_folder();
        let repo = initialize_repo_in_folder(temp_git_repo);
        create_initial_commit(&repo);
        create_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        let author = generate_author(1);
        commit_changes_to_repo(&repo, author);
        repo
    }

    fn routine2(file: &str) -> Repository{
        //let temp_git_repo = create_temp_folder();
        let temp_git_repo = create_folder();
        let repo = initialize_repo_in_folder(temp_git_repo);
        create_initial_commit(&repo);
        create_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        let original_author = repo.signature().unwrap();
        commit_changes_to_repo(&repo, original_author);

        let second_author = Signature::new(
            "author2",
            "mail",
            &Time::new(0, 0))
            .unwrap();
        update_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo, second_author);
        repo
    }

    fn generate_author<'a>(author_index: u32) -> Signature<'a>{
        let author_string = "author".to_string();
        let author_index = author_index.to_string();
        let author_name = author_string+&author_index;
        let author_name_str: &str = &author_name.to_string()[..];
        Signature::new(
            author_name_str,
            "mail",
            &Time::new(0, 0))
            .unwrap()
    }

    fn create_temp_folder() -> PathBuf {
        let git_repo_path = "git_repo_test";
        let temp_folder = TempDir::new(git_repo_path).unwrap();
        temp_folder.path().to_path_buf()
    }

    fn create_folder() -> PathBuf{
        let git_repo_path = "tests/git_repo_test";
        std::fs::create_dir(git_repo_path).unwrap();
        PathBuf::from(git_repo_path)
    }

    fn initialize_repo_in_folder(temp_git_repo: PathBuf) -> Repository {
       Repository::init(temp_git_repo).unwrap()
    }

    fn create_initial_commit(repo: &Repository) {
        let author = generate_author(1);
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &author,
            &author,
            "Initial commit",
            &tree,
            &[])
            .unwrap();
    }

    fn create_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        std::fs::write(&path, "a\n").unwrap();
    }

    fn update_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        let mut file = File::options()
            .append(true)
            .open(path)
            .unwrap();
        writeln!(&mut file, "b").unwrap();
    }

    fn add_file_to_the_staging_area(repo: &Repository, file: &str){
        let mut index = repo.index().unwrap(); // index = staging_area
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }

    fn commit_changes_to_repo(repo: &Repository, author: Signature){
        let head = repo.head().unwrap();
        let parent = repo.find_commit(head.target().unwrap()).unwrap();
        let tree = repo.find_tree(repo.index().unwrap().write_tree().unwrap()).unwrap();
        repo.commit(
            Some("HEAD"),
            &author,
            &author,
            "Test commit",
            &tree,
            &[&parent],
        ).unwrap();
    }
}