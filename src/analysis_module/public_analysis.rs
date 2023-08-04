use crate::analysis_module::analysis::{do_internal_analysis, TopAnalysis};
use crate::data_sources::file_explorer::FileExplorer;
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::IMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use git2::Repository;
use std::path::PathBuf;

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    let mut metrics_to_analyze: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];
    let repository_git_folder_of_root = Repository::discover(&root);
    if repository_git_folder_of_root.is_ok() {
        let existing_repository_git_folder_of_root = PathBuf::from(repository_git_folder_of_root.unwrap().path());
        let existing_git_repository_project = existing_repository_git_folder_of_root.parent().unwrap();
        metrics_to_analyze.push(Box::new(SocialComplexityMetric::new(&root, &existing_git_repository_project.to_path_buf())));
    } else {
        eprintln!("WARN: Analysed folder is not a git repository");
    }
    do_internal_analysis(&root, &FileExplorer::new(&root), &metrics_to_analyze)
}
