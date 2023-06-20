use crate::analysis_module::analysis::{do_internal_analysis, TopAnalysis};
use crate::data_sources::file_explorer::FileExplorer;
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use std::path::PathBuf;

pub fn do_analysis(root: PathBuf) -> TopAnalysis {
    do_internal_analysis(
        &root,
        &FileExplorer::new(&root),
        &[
            Box::new(LinesCountMetric::new()),
            Box::new(SocialComplexityMetric::new()),
        ],
    )
}
