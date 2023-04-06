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
    use git2::Oid;
    use super::*;
    use rstest::rstest;
    use tempdir::TempDir;

    #[rstest(file, expected_authors,
    case("file1.txt", 1),  // file1.txt has 3 authors
    )]
    fn smells_get_number_of_authors_of_a_file(file: &str, expected_authors: u32){
        // Create a temporary directory for testing
        /*let temp_dir = TempDir::new("temp_folder").unwrap();
        let temp_git_repo = temp_dir.path().join("test_repo");
        eprintln!("{:?}", temp_git_repo);*/

        // Create directory
        let git_repo_path = "tests/git_repo_test";
        std::fs::create_dir(git_repo_path).unwrap();

        // Initialize a repository in the directory
        let repo = Repository::init(&git_repo_path).unwrap();

        create_initial_commit(&repo);
        create_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo);

        // Commit the changes
        /*let tree_id = commit_changes(&repo, oid);

        // Get the path of the committed file
        let file_path = repo_path.join(file);

        // Call the function to get the number of authors for the file
        let actual_authors = get_number_of_authors_of_a_file(&repo, &file_path);

        // Assert that the actual number of authors matches the expected number of authors
        assert_eq!(actual_authors, expected_authors);*/
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
        let path = repo.workdir().unwrap().join(file);
        std::fs::write(&path, "").unwrap();
    }

    fn add_file_to_the_staging_area(repo: &Repository, file: &str){
        let mut index = repo.index().unwrap();
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }

    // create a branch with a specified name
    fn create_branch(repo: &Repository, branch_name: &str){
        let head = repo.head().unwrap();
        let head_commit = repo.find_commit(head.target().unwrap()).unwrap();
        repo.branch(branch_name, &head_commit, false).unwrap();
    }

    fn commit_changes_to_repo(repo: &Repository) -> Oid{
        let head = repo.head().unwrap();
        let parent = repo.find_commit(head.target().unwrap()).unwrap();
        let signature = repo.signature().unwrap();
        let tree = repo.find_tree(repo.index().unwrap().write_tree().unwrap()).unwrap();
        let tree_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Test commit",
            &tree,
            &[&parent],
        ).unwrap();
        tree_id
    }

}