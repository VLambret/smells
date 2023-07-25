use crate::analysis_module::analysis::{do_internal_analysis, TopAnalysis};
use crate::data_sources::file_explorer::FileExplorer;
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::IMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use git2::Repository;
use std::path::PathBuf;

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    let mut metrics_to_analyze: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];
    if is_git_repository(&root) {
        metrics_to_analyze.push(Box::new(SocialComplexityMetric::new(&root)));
    } else {
        eprintln!("WARN: Analysed folder is not a git repository");
    };
    do_internal_analysis(&root, &FileExplorer::new(&root), &metrics_to_analyze)
}

fn is_git_repository(path: &PathBuf) -> bool {
    Repository::open(path).is_ok()
}
