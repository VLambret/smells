use crate::metrics::metric::MetricScoreType::Score;
use crate::metrics::metric::MetricValueType::Authors;
use crate::metrics::metric::SmellsError::*;
use crate::metrics::metric::{
    AnalysisError, IMetric, IMetricValue, MetricScoreType, MetricValueType, OptionError,
    ResultError, SmellsError,
};
use git2::Repository;
use log::{info, warn};
use std::fmt::Debug;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SocialComplexityMetric {
    analyzed_folder: PathBuf,
    git_repo_of_analyzed_folder: PathBuf,
}

impl SocialComplexityMetric {
    pub fn new(
        analyzed_folder: &PathBuf,
        git_repo_of_analyzed_folder: &PathBuf,
    ) -> SocialComplexityMetric {
        SocialComplexityMetric {
            analyzed_folder: analyzed_folder.to_owned(),
            git_repo_of_analyzed_folder: git_repo_of_analyzed_folder.to_owned(),
        }
    }
}

impl IMetric for SocialComplexityMetric {
    fn analyse(&self, file_path: &Path) -> Option<Box<dyn IMetricValue>> {
        if let Ok(relative_file_path) =
            get_relative_file_path(file_path, &self.git_repo_of_analyzed_folder)
        {
            if !is_file_versioned(&self.git_repo_of_analyzed_folder, &relative_file_path) {
                //info!("{:?} NOT VERSIONED", &file_path);
                return None;
            } else {
                //warn!("{:?} VERSIONED OK", &relative_file_path);
                match get_authors_of_file(&self.git_repo_of_analyzed_folder, &relative_file_path) {
                    Ok(Some(authors)) => Some(Box::new(SocialComplexityValue {
                        authors: Ok(authors),
                    })),
                    Ok(None) => None,
                    _ => Some(Box::new(SocialComplexityValue {
                        authors: Err("Analysis Error".to_string()),
                    })),
                }
            }
        } else {
            Some(Box::new(SocialComplexityValue {
                authors: Err("Analysis Error".to_string()),
            }))
        }
    }
}

fn is_file_versioned(git_repo: &Path, file: &Path) -> bool {
    //info!("repo: {:?}, file: {:?}", git_repo, file);
    match Repository::open(git_repo) {
        Ok(repo) => {
            //TODO: check MAIN also

            if let Ok(index) = repo.index() {
                let file_in_index = index.get_path(file, 0);
                file_in_index.is_some()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn get_authors_of_file(
    git_repo: &PathBuf,
    file: &Path,
) -> Result<Option<Vec<String>>, SmellsError> {
    let repo = Repository::open(git_repo).unwrap();
    let blame = repo.blame_file(file, None).unwrap();
    //TODO: find a robust solution
    let standardized_path = file.to_string_lossy().replace('\\', "/");
    let spec = format!("HEAD:{}", standardized_path);
    let object = repo.revparse_single(&spec).unwrap();
    let blob = repo.find_blob(object.id()).unwrap();

    let reader = BufReader::new(blob.content());
    let mut authors: Vec<String> = vec![];
    for (line_nb, _line_content) in reader.lines().enumerate() {
        if let Some(hunk) = blame.get_line(line_nb + 1) {
            let signature = hunk.orig_signature();
            let author_name = signature
                .name()
                .map(|name| name.to_string())
                .ok_or(OptionError);
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

fn get_relative_file_path(file: &Path, git_repository: &Path) -> Result<PathBuf, ResultError> {
    match file.strip_prefix(git_repository) {
        Ok(relative_file_path) => Ok(PathBuf::from(relative_file_path)),
        Err(_) => Err(ResultError::new(String::new())),
    }
}

#[derive(Debug, PartialEq, Clone)]
struct SocialComplexityValue {
    authors: Result<Vec<String>, AnalysisError>,
}

impl IMetricValue for SocialComplexityValue {
    fn get_key(&self) -> &'static str {
        "social_complexity"
    }

    fn get_score(&self) -> Result<MetricScoreType, AnalysisError> {
        match &self.authors {
            Ok(authors) => Ok(Score(authors.len() as u64)),
            Err(error) => Err(error.to_owned()),
        }
    }

    fn get_value(&self) -> Result<MetricValueType, AnalysisError> {
        match &self.authors {
            Ok(authors) => Ok(Authors(authors.to_owned())),
            Err(error) => Err(error.to_owned()),
        }
    }

    fn aggregate(&self, other: Box<dyn IMetricValue>) -> Box<dyn IMetricValue> {
        let combined_authors: Result<Vec<String>, AnalysisError> = {
            match (self.authors.as_ref(), other.get_value().as_ref()) {
                (Err(_), Err(_)) => Err(String::from("Analysis error")),
                (Err(_), _) => match other.get_value() {
                    Ok(Authors(authors)) => Ok(authors),
                    //TODO: ok ?
                    _ => Ok(vec![]),
                },
                (_, Err(_)) => match self.get_value() {
                    Ok(Authors(authors)) => Ok(authors),
                    _ => Ok(vec![]),
                },
                _ => {
                    let self_authors = match self.get_value() {
                        Ok(Authors(authors)) => authors,
                        _ => {
                            vec![]
                        }
                    };
                    let other_authors = match other.get_value() {
                        Ok(Authors(authors)) => authors,
                        _ => {
                            vec![]
                        }
                    };
                    let mut unique_authors = self_authors;
                    for other_author in other_authors {
                        if !unique_authors.contains(&other_author) {
                            unique_authors.push(other_author.to_owned());
                        }
                    }
                    Ok(unique_authors)
                }
            }
        };
        Box::new(SocialComplexityValue {
            authors: combined_authors,
        })
    }
}
