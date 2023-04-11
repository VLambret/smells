use std::path::{Path, PathBuf};
use git2::Repository;

pub fn social_complexity(root_path: &str) -> u32{
    return 0;
}

fn get_number_of_authors_of_a_file(path_to_repo: &Path, file: &PathBuf) -> u32{
    let repo = Repository::open(path_to_repo).expect("Cannot open repository");
    let relative_file_path = get_relative_path(&repo,&file);

    let blame = match repo.blame_file(&relative_file_path, None){
        Ok(blame) => blame,
        Err(_) => return 0
    };

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
    use std::fs::{File, remove_dir_all};
    use git2::{Signature};
    use rstest::rstest;
    use tempdir::TempDir;
    use std::io::Write;
    use std::path::Path;

    // TODO: gerer le cas 0
    #[rstest(file, expected_authors,
    case("file1.txt", 1),
    case("file2.txt", 2),
    case("file10.txt", 10),
    case("file0.txt", 0)
    )]
    fn smells_get_number_of_authors_of_a_file(file: &str, expected_authors: u32){
        let mut authors = Vec::new();
        for author_index in 1..=expected_authors{
            authors.push(generate_author(author_index));
        }

        let git_temp_repo_path = "git_repo_test";
        let repo: Repository;
        if authors.len() > 0{
            repo = setup_repo_with_an_empty_file(file, &authors[0], git_temp_repo_path);
            for author in authors{
                author_commit_an_updated_file(&repo, file, &author);
            }
        }
        else{
            repo = initialize_repo_in_folder("tests/data");
        }

        let committed_file_path = repo.path().join(file);
        let numbers_of_authors_of_specified_file = get_number_of_authors_of_a_file(repo.path(), &committed_file_path);
        assert_eq!(numbers_of_authors_of_specified_file, expected_authors);
    }



    fn setup_repo_with_an_empty_file(file: &str, author: &Signature, git_repo_path: &str) -> Repository{
        let temp_git_repo = create_temp_folder(git_repo_path);
        //let temp_git_repo = create_folder(); // concrete folder
        let repo = initialize_repo_in_folder(temp_git_repo.to_string_lossy().as_ref());
        create_initial_commit(&repo, author);
        create_file(&repo, file);
        repo
    }

    fn author_commit_an_updated_file(repo: &Repository, file: &str, author: &Signature){
        update_file(&repo, file);
        add_file_to_the_staging_area(&repo, file);
        commit_changes_to_repo(&repo, author);
    }

    fn generate_author<'a>(author_index: u32) -> Signature<'a> {
        Signature::now(format!("author{}", author_index).as_str(), "mail").unwrap()
    }

    fn create_temp_folder(git_repo_path: &str) -> PathBuf {
        let temp_folder = TempDir::new(git_repo_path).unwrap();
        temp_folder.path().to_path_buf()
    }

    fn create_temp_folder2() -> PathBuf {
        let git_repo_path = "tests/data/git_repo_test";
        let git_repo_absolute_path = std::env::current_dir().unwrap().join(git_repo_path);
        println!("{:?}", git_repo_absolute_path);
        let git_repo_absolute_path_as_str = git_repo_absolute_path.to_string_lossy();
        println!("{:?}", git_repo_absolute_path_as_str);
        let temp_folder = TempDir::new(git_repo_absolute_path_as_str.as_ref()).unwrap();
        temp_folder.path().to_path_buf()
    }

    fn create_folder() -> PathBuf{
        let git_repo_path = Path::new("tests/data/git_repo_test").to_owned();
        delete_folder_if_exists(&git_repo_path);
        std::fs::create_dir(&git_repo_path).unwrap();
        git_repo_path
    }

    fn delete_folder_if_exists(folder_path: &PathBuf){
        if std::fs::metadata(&folder_path).is_ok() {
            remove_dir_all(folder_path).unwrap();
        }
    }

    fn initialize_repo_in_folder(temp_git_repo: &str) -> Repository {
       Repository::init(temp_git_repo).unwrap()
    }

    fn create_initial_commit(repo: &Repository, author: &Signature) {
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
        std::fs::write(&path, "").unwrap();
    }

    fn update_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        let mut file = File::options()
            .append(true)
            .open(path)
            .unwrap();
        writeln!(&mut file, "a").unwrap();
    }

    fn add_file_to_the_staging_area(repo: &Repository, file: &str){
        let mut index = repo.index().unwrap(); // index = staging_area
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }

    fn commit_changes_to_repo(repo: &Repository, author: &Signature){
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