use crate::metrics::metric::MetricScoreType::Score;
use crate::metrics::metric::{
    AnalysisError, IMetric, IMetricValue, MetricScoreType, MetricValueType,
};
use git2::{Error as git2Error, Repository};
use std::fmt::{Debug, Error as OtherError};
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
    //TODO: homogenize analyse between metrics
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
    //TODO:unwrap
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
