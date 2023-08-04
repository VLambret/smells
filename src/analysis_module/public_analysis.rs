use crate::analysis_module::analysis::{do_internal_analysis, TopAnalysis};
use crate::data_sources::file_explorer::FileExplorer;
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::IMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use git2::Repository;
use log::warn;
use std::path::PathBuf;

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    let mut metrics_to_analyze: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];
    if is_git_repository(&root) {
        metrics_to_analyze.push(Box::new(SocialComplexityMetric::new(&root, &root)));
    }
    //TODO: remove repetition
    else if get_containing_git_repository(&root).is_some() {
        let root_git_repo = get_containing_git_repository(&root).unwrap();
        metrics_to_analyze.push(Box::new(SocialComplexityMetric::new(&root, &root_git_repo)));
    } else {
        eprintln!("WARN: Analysed folder is not a git repository");
    };
    do_internal_analysis(&root, &FileExplorer::new(&root), &metrics_to_analyze)
}

fn get_containing_git_repository(folder: &PathBuf) -> Option<PathBuf> {
    // TODO: use Repository::discover(git_repo)
    warn!("-> Current folder {:?}", &folder);
    match Repository::open(folder) {
        Ok(_) => {
            warn!("is a git repo");
            Some(folder.to_owned())
        }
        Err(_) => {
            if folder.parent().is_none() || folder.parent() == Some("".as_ref()) {
                None
            } else {
                get_containing_git_repository(&folder.parent().unwrap().to_path_buf())
            }
        }
    }
}

fn is_git_repository(folder: &PathBuf) -> bool {
    Repository::open(folder).is_ok()
}
