use crate::metrics::metric::MetricScoreType::Score;
use crate::metrics::metric::{
    AnalysisError, IMetric, IMetricValue, MetricScoreType, MetricValueType,
};
use git2::{Commit, Error as git2Error, ObjectType, Repository};
use log::{error, info, warn};
use std::fmt::{Debug, Error as OtherError};
use std::fs::read_dir;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SocialComplexityMetric {
    root: PathBuf,
}

impl SocialComplexityMetric {
    pub fn new(root: &PathBuf) -> SocialComplexityMetric {
        SocialComplexityMetric {
            root: root.to_owned(),
        }
    }
}

impl IMetric for SocialComplexityMetric {
    fn analyse(&self, file_path: &Path) -> Option<Box<dyn IMetricValue>> {
        let relative_file_path = get_relative_file_path(file_path, &self.root);
        if is_file_versioned(&self.root, &relative_file_path) {
            let authors_if_some = get_authors_of_file(&self.root, &relative_file_path);
            if let Ok(Some(authors)) = authors_if_some {
                Some(Box::new(SocialComplexityValue { authors }))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn is_file_versioned(repo: &Path, file: &Path) -> bool {
    match Repository::discover(repo) {
        Ok(repo) => repo.status_file(file).is_ok(),
        Err(_) => false,
    }
}

fn get_authors_of_file(root: &PathBuf, file: &Path) -> Result<Option<Vec<String>>, git2Error> {
    let repo = Repository::open(root)?;
    let blame = repo.blame_file(file, None)?;

    let spec = "HEAD:".to_owned() + file.to_string_lossy().as_ref();
    let object = repo.revparse_single(&spec)?;
    let blob = repo.find_blob(object.id())?;

    let reader = BufReader::new(blob.content());
    let mut authors: Vec<String> = vec![];
    for (line_nb, _line_content) in reader.lines().enumerate() {
        if let Some(hunk) = blame.get_line(line_nb + 1) {
            let signature = hunk.orig_signature();
            //TODO: use std::error:Error for function return
            let author_name = signature
                .name()
                .map(|name| name.to_string())
                .ok_or(OtherError);
            if let Ok(valid_author_name) = author_name {
                if !authors.contains(&valid_author_name) {
                    authors.push(valid_author_name);
                }
            }
        }
    }
    if !authors.is_empty() {
        Ok(Some(authors))
    } else {
        Ok(None)
    }
}

fn get_relative_file_path(file: &Path, root: &Path) -> PathBuf {
    let relative = file.strip_prefix(root).unwrap();
    relative.to_owned()
}

#[derive(Debug, PartialEq, Clone)]
struct SocialComplexityValue {
    authors: Vec<String>,
}

impl IMetricValue for SocialComplexityValue {
    fn get_key(&self) -> &'static str {
        "social_complexity"
    }

    fn get_score(&self) -> Result<MetricScoreType, AnalysisError> {
        Ok(Score(self.authors.len() as u64))
    }

    fn get_value(&self) -> Result<MetricValueType, AnalysisError> {
        Ok(MetricValueType::Authors(self.authors.to_owned()))
    }

    fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
        let mut combined_authors = self.authors.clone();
        if let Ok(MetricValueType::Authors(other_authors)) = other.get_value() {
            for other_author in other_authors {
                if !combined_authors.contains(&other_author) {
                    combined_authors.push(other_author.to_owned());
                }
            }
        }
        Box::new(SocialComplexityValue {
            authors: combined_authors,
        })
    }
}

// TODO: handle unwrap() + link to social_complexity + on va open le repo dans social_complexity
// print_number_of_authors_of_repo_dir2()
fn _get_number_of_authors_of_repo_dir(repo: &Repository, path: PathBuf) -> u32 {
    let mut authors_number = 0;
    for file in read_dir(path).unwrap() {
        let file_path = file.unwrap().path();
        let relative = _get_relative_path(repo.path(), &file_path);
        if file_path.is_file() {
            // apres 1ere iter on n a plus un repo mais un file donc on peut pas open le repo dans la fct
            authors_number = _get_file_social_complexity(repo, &relative.to_path_buf());
        } else {
            _get_number_of_authors_of_repo_dir(repo, file_path);
        }
    }
    authors_number
}

fn _get_file_social_complexity(repo: &Repository, file: &Path) -> u32 {
    let relative_file_path = _get_relative_path(repo.path(), file);
    let blame = match repo.blame_file(&relative_file_path, None) {
        Ok(blame) => blame,
        // TODO: a voir si c est vraiment ce qu on veut quand on a l erreur
        Err(_) => return 0,
    };
    blame.iter().count() as u32
}

fn _get_relative_path(path_to_repo: &Path, path: &Path) -> PathBuf {
    let mut relative_path = path.to_path_buf();
    if path.is_absolute() {
        relative_path = path.strip_prefix(path_to_repo).unwrap().to_path_buf();
    }
    relative_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use git2::{Commit, Signature, Tree};
    use rstest::rstest;
    use std::fs::{create_dir_all, remove_dir_all, File};
    use std::io::Write;

    #[rstest(expected_social_complexity, case(0), case(1), case(2), case(10))]
    fn file_social_complexity(expected_social_complexity: u32) {
        let repo_name = format!("repo_with_{}_authors", expected_social_complexity);
        let repo = create_git_test_repository(repo_name);
        let multi_author_file = "file.txt";
        for author_seed in 1..=expected_social_complexity {
            commit_line_change_authored_by(&repo, multi_author_file, &generate_author(author_seed));
        }
        let committed_file_path = repo.path().join(multi_author_file);
        let actual_social_complexity = _get_file_social_complexity(&repo, &committed_file_path);
        assert_eq!(expected_social_complexity, actual_social_complexity);
    }

    // TODO: ca marche pas = erreur sur le StripPrefix
    #[rstest(path_to_repo, expected_authors, case("git_repo_test", 1))]
    #[ignore]
    fn smells_get_numbers_of_authors_of_files_of_a_repo(
        path_to_repo: String,
        expected_authors: u32,
    ) {
        let mut authors = Vec::new();
        for author_index in 1..=expected_authors {
            authors.push(generate_author(author_index));
        }
        let file = "file1.txt";
        let author = &authors[0];
        let repo = create_git_test_repository(path_to_repo);
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), author, author, "Initial commit", &tree, &[])
            .unwrap();
        create_file(&repo, file);
        let root_path = repo.path().parent().unwrap().to_path_buf();
        for author in authors {
            commit_line_change_authored_by(&repo, file, &author);
        }
        //assert_eq!(get_number_of_authors_of_repo_dir(repo.path()), predicate::str::contains("1"));
        assert_eq!(1, _get_number_of_authors_of_repo_dir(&repo, root_path));
    }
    fn commit_line_change_authored_by(repo: &Repository, file: &str, author: &Signature) {
        update_file(repo, file);
        add_file_to_the_staging_area(repo, file);
        commit_changes_to_repo(repo, author);
    }
    fn generate_author<'a>(author_index: u32) -> Signature<'a> {
        Signature::now(format!("author{}", author_index).as_str(), "mail").unwrap()
    }
    fn create_git_test_repository(repo_name: String) -> Repository {
        // TODO : Repository::init doesn't work on Windows, it automatically add ./ to the path
        let repo = std::env::current_dir().unwrap().join(
            PathBuf::from("tests")
                .join("data")
                .join("git_repositories")
                .join(repo_name),
        );
        if repo.exists() {
            remove_dir_all(&repo).unwrap();
        }
        create_dir_all(&repo).unwrap();
        //TODO: handle unwrap()
        Repository::init(repo).unwrap()
    }
    fn create_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        std::fs::write(&path, "").unwrap();
    }
    fn update_file(repo: &Repository, file: &str) {
        let path = repo.path().parent().unwrap().join(file);
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(&mut file, "a").unwrap();
    }
    fn add_file_to_the_staging_area(repo: &Repository, file: &str) {
        let mut index = repo.index().unwrap(); // index = staging_area
        index.add_path(&PathBuf::from(file)).unwrap();
        index.write().unwrap();
    }
    fn commit_changes_to_repo(repo: &Repository, author: &Signature) {
        match repo.head() {
            Ok(head) => {
                let parent = repo.find_commit(head.target().unwrap()).unwrap();
                let tree = repo
                    .find_tree(repo.index().unwrap().write_tree().unwrap())
                    .unwrap();
                let parents = &[&parent];
                create_test_commit(repo, author, &tree, parents);
            }
            Err(_) => {
                let tree_id = {
                    let mut index = repo.index().unwrap();
                    index.write_tree().unwrap()
                };
                let tree = repo.find_tree(tree_id).unwrap();
                let parents = &[];
                create_test_commit(repo, author, &tree, parents);
            }
        }
    }
    fn create_test_commit(repo: &Repository, author: &Signature, tree: &Tree, parents: &[&Commit]) {
        repo.commit(
            Some("HEAD"),
            author,
            author,
            "Commit message",
            tree,
            parents,
        )
        .unwrap();
    }
}
