use crate::analysis_module::analysis::{do_internal_analysis, TopAnalysis};
use crate::data_sources::file_explorer::FileExplorer;
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::IMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use git2::Repository;
use std::path::PathBuf;
use std::process::exit;

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    let is_empty = root.read_dir().unwrap().next().is_none();
    if is_empty {
        eprintln!("WARN: Analysed folder does not contain any file");
        exit(1);
    }

    let mut metrics_to_analyze: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];
    let git_repository_of_root = Repository::discover(&root);
    if let Ok(existing_git_repository_of_root) = git_repository_of_root {
        existing_git_repository_of_root.path().parent().and_then(
            |project_containing_git_repository_of_root| {
                metrics_to_analyze.push(Box::new(SocialComplexityMetric::new(
                    &project_containing_git_repository_of_root.to_path_buf(),
                )));
                Some(())
            },
        );
    } else {
        eprintln!("WARN: Analysed folder is not a git repository");
    }
    do_internal_analysis(&root, &FileExplorer::new(&root), &metrics_to_analyze)
}
