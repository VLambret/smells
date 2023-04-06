use std::path::PathBuf;
use git2::Repository;

pub fn social_complexity(root_path: &str) -> u32{
    compute_social_complexity_of_a_file(PathBuf::from(root_path))
}

fn compute_social_complexity_of_a_file(analysed_file_path: PathBuf) -> u32{
    return 0;
}

/*fn get_number_of_authors_of_a_file(repo_path: &PathBuf, file: &PathBuf) -> u32{
    let repo = Repository::open(repo_path).expect("can't open repo");
    let blame = repo.blame_file(&file, None).unwrap();
    let mut authors = Vec::new();

    for blame_hunk in blame.iter(){
        let signature = blame_hunk.final_signature();
        let name = signature.name().unwrap().to_owned();
        if !authors.contains(&name){
            authors.push(name);
        }
    }
    authors.len() as u32
}*/

fn get_number_of_authors_of_a_file(_repo_path: &Repository, _file: &PathBuf) -> u32{
    return 0;
}

#[cfg(test)]
mod tests{
    use super::*;
    use rstest::rstest;
    use tempdir::TempDir;

    #[rstest(file, expected_authors,
    case("file0.txt", 0),
    )]
    fn smells_get_number_of_authors_of_a_file(file: &str, expected_authors: u32){
        let repo = routine(file);
        let committed_file_path = repo.path().join(file);
        let numbers_of_authors_of_specified_file = get_number_of_authors_of_a_file(&repo, &committed_file_path);
        assert_eq!(numbers_of_authors_of_specified_file, expected_authors);
    }

    fn routine(file: &str) -> Repository{
        let temp_git_repo = create_temp_folder();
        let repo = initialize_repo_in_folder(temp_git_repo);
        create_initial_commit(&repo);
        create_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo);
        repo
    }

    fn create_temp_folder() -> PathBuf {
        let git_repo_path = "git_repo_test";
        let temp_folder = TempDir::new(git_repo_path).unwrap();
        temp_folder.path().to_path_buf()
    }

    fn initialize_repo_in_folder(temp_git_repo: PathBuf) -> Repository {
       Repository::init(temp_git_repo).unwrap()
    }

    fn create_initial_commit(repo: &Repository) {
        let signature = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[])
            .unwrap();
    }

    fn create_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        std::fs::write(&path, "").unwrap();
    }

    fn add_file_to_the_staging_area(repo: &Repository, file: &str){
        let mut index = repo.index().unwrap();
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }

    fn commit_changes_to_repo(repo: &Repository){
        let head = repo.head().unwrap();
        let parent = repo.find_commit(head.target().unwrap()).unwrap();
        let signature = repo.signature().unwrap();
        let tree = repo.find_tree(repo.index().unwrap().write_tree().unwrap()).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Test commit",
            &tree,
            &[&parent],
        ).unwrap();
    }
}