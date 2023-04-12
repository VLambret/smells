use std::fs::read_dir;
use std::path::{Path, PathBuf};
use git2::Repository;

//print_number_of_authors_of_entire_repo()
pub fn social_complexity(root_path: &str) -> u32{
    return 0;
}

// TODO: handle unwrap() + link to social_complexity + on va open le repo dans social_complexity
// print_number_of_authors_of_repo_dir2()
fn get_number_of_authors_of_repo_dir(repo: &Repository, path: PathBuf) -> u32{
    let mut authors_number = 0;
    for file in read_dir(path).unwrap(){
        let file_path = file.unwrap().path();
        let relative = get_relative_path(repo.path(), &file_path);
        //println!("{:?}", relative);
        if file_path.is_file(){
            // apres 1ere iter on n a plus un repo mais un file donc on peut pas open le repo dans la fct
            authors_number = get_number_of_authors_of_a_file(repo, &relative.to_path_buf());
        } else {
            get_number_of_authors_of_repo_dir(&repo, file_path);
        }
    }
    authors_number
}

fn get_number_of_authors_of_a_file(repo: &Repository, file: &PathBuf) -> u32{
    let relative_file_path = get_relative_path(repo.path(),&file);

    let blame = match repo.blame_file(&relative_file_path, None){
        Ok(blame) => blame,
        // TODO: a voir si c est vraiment ce qu on veut quand on a l erreur
        Err(_) => return 0
    };

    let authors: Vec<String> = blame.iter().map(|blame_hunk| blame_hunk.final_signature().name().unwrap().to_owned()).collect();
    authors.len() as u32
}

fn get_relative_path(path_to_repo: &Path, path: &PathBuf) -> PathBuf{
    let mut relative_path = path.clone();
    if path.is_absolute(){
        relative_path = path.strip_prefix(path_to_repo).unwrap().to_path_buf();
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

    // TODO: refactor le test (?)
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

        // TODO: handle temp folder issue
        let git_temp_repo_path = "git_repo_test";
        let repo: Repository;
        // TODO: remove if
        if authors.len() > 0{
            repo = setup_repo_with_a_file(file, &authors[0], git_temp_repo_path);
            for author in authors{
                author_commit_an_modified_file(&repo, file, &author);
            }
        }
        else{
            repo = initialize_repo_in_folder("tests/data");
        }

        let committed_file_path = repo.path().join(file);
        let numbers_of_authors_of_specified_file = get_number_of_authors_of_a_file(&repo, &committed_file_path);
        assert_eq!(numbers_of_authors_of_specified_file, expected_authors);
    }
    // TODO: ca marche pas = erreur sur le StripPrefix
    #[rstest(path_to_repo, expected_authors,
    case("git_repo_test", 1), )]
    #[ignore]
    fn smells_get_numbers_of_authors_of_files_of_a_repo(path_to_repo: &str, expected_authors: u32){
        let mut authors = Vec::new();
        for author_index in 1..=expected_authors{
            authors.push(generate_author(author_index));
        }

        let file = "file1.txt";
        let repo = setup_repo_with_a_file(file, &authors[0], path_to_repo);
        let root_path = repo.path().parent().unwrap().to_path_buf();

        for author in authors{
            author_commit_an_modified_file(&repo, file, &author);
        }
        //assert_eq!(get_number_of_authors_of_repo_dir(repo.path()), predicate::str::contains("1"));
        assert_eq!(get_number_of_authors_of_repo_dir(&repo, root_path), 1);
    }

    fn setup_repo_with_a_file(file: &str, author: &Signature, git_repo_path: &str) -> Repository{
        let temp_git_repo = create_temp_folder(git_repo_path);
        //let temp_git_repo = create_folder(git_repo_path); // concrete folder
        let repo = initialize_repo_in_folder(temp_git_repo.to_string_lossy().as_ref());
        create_initial_commit(&repo, author);
        create_file(&repo, file);
        repo
    }

    fn author_commit_an_modified_file(repo: &Repository, file: &str, author: &Signature){
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

    fn create_folder(git_repo_path: &str) -> PathBuf{
        let git_repo_path = Path::new(git_repo_path).to_owned();
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