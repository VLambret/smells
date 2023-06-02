use crate::data_sources::file_explorer::{FileExplorer, IFileExplorer};
use crate::metrics::line_count::LinesCountMetric;
use crate::metrics::metric::IMetric;
use crate::metrics::social_complexity::SocialComplexityMetric;
use maplit::hashmap;
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::string::String;

/* **************************************************************** */

/*
General smells :
- output arguments : s.improve() instead of improve(s), structures everywhere ?
- Duplication : 3 initialization of AnalysisInTree, can we abstract it ?
- Inconsistency : parent / parent_analysis ...
- Analysis module too big
- Obscured intent ?
- AnalysisInTree : contains parent and children id, but not the analysis id
- Analysis / content vs folder_content ? Unclear notions
- Unwrap : capture errors
*/

pub struct TreeOfAnalyses {
    root_analysis_id: String,
    analyses: HashMap<AnalysisInTreeId, AnalysisInTree>,
}
pub type AnalysisInTreeId = String;

impl TreeOfAnalyses {
    fn new(root_id: String, analyses: HashMap<AnalysisInTreeId, AnalysisInTree>) -> TreeOfAnalyses {
        TreeOfAnalyses {
            root_analysis_id: root_id,
            analyses,
        }
    }

    fn get_root(&self) -> &String {
        &self.root_analysis_id
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct AnalysisInTree {
    pub file_name: String,
    pub metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    pub parent_id: Option<String>,
    pub folder_content_id: Option<Vec<String>>,
}

/* **************************************************************** */

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Analysis {
    pub file_name: String,
    pub metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    pub folder_content: Option<BTreeMap<String, Analysis>>,
}

pub type AnalysisError = String;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum MetricsValueType {
    Score(u32),
    Error(AnalysisError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsValueAggregable{
    value: MetricsValueType
}

impl MetricsValueAggregable{
    fn new(value: MetricsValueType) -> Self {
        MetricsValueAggregable{
            value
        }
    }

    fn aggregate(&mut self, other: &MetricsValueAggregable){
        if let (MetricsValueType::Score(parent_score), MetricsValueType::Score(file_score)) = (&self.value, &other.value) {
            let mut parent_score = *parent_score;
            parent_score += *file_score;
            self.value = MetricsValueType::Score(parent_score);
        };
    }
}

impl Serialize for MetricsValueAggregable{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match &self.value {
            MetricsValueType::Score(score) => serializer.serialize_u32(*score),
            MetricsValueType::Error(error) => serializer.serialize_str(error),
        }
    }
}

/* **************************************************************** */

pub fn do_analysis(root: PathBuf) -> Analysis {
    do_internal_analysis(
        &root,
        Box::new(FileExplorer::new(&root)),
        vec![
            Box::new(LinesCountMetric::new()),
            Box::new(SocialComplexityMetric::new()),
        ],
    )
}

/* **************************************************************** */

fn do_internal_analysis(
    root: &Path,
    file_explorer: Box<dyn IFileExplorer<Item = PathBuf>>,
    metrics: Vec<Box<dyn IMetric>>,
) -> Analysis {
    let root_analysis = AnalysisInTree {
        file_name: String::from(root.to_string_lossy()),
        metrics: get_metrics_keys(&metrics),
        parent_id: None,
        folder_content_id: Some(vec![]),
    };
    let root_id: AnalysisInTreeId = String::from(root.to_path_buf().to_string_lossy());
    let mut tree_of_analyses = TreeOfAnalyses::new(root_id.clone(),
                                                   hashmap! {root_id => root_analysis });

    for file in file_explorer.discover() {
        analyse_single_file(&metrics, &mut tree_of_analyses, &file);
    }
    convert_analysis_in_tree_of_analyses_to_final_analysis(tree_of_analyses.analyses.clone(), tree_of_analyses.get_root())
}

fn analyse_single_file(metrics: &Vec<Box<dyn IMetric>>, tree_of_analyses: &mut TreeOfAnalyses, file: &Path) {
    let parents = get_parents_ordered_from_root(file);
    let mut last_parent_of_file_id: AnalysisInTreeId = tree_of_analyses.get_root().to_owned();
    for parent in parents {
        add_parent_analysis_to_tree_of_analyses(
            &mut tree_of_analyses.analyses,
            &mut last_parent_of_file_id,
            parent,
        );
    }
    let file_analysis = create_and_push_file_analysis_into_tree_of_analysis(
        metrics,
        &mut tree_of_analyses.analyses,
        file,
        &mut last_parent_of_file_id,
    );
    propagate_file_scores_to_parents_analysis(&mut tree_of_analyses.analyses, file_analysis);
}

fn get_metrics_keys(metrics: &Vec<Box<dyn IMetric>>) -> BTreeMap<&'static str, Option<MetricsValueAggregable>> {
    let mut metrics_keys: BTreeMap<&'static str, Option<MetricsValueAggregable>> = BTreeMap::new();
    for metric in metrics {
        metrics_keys.insert(metric.get_key(), None);
    }
    metrics_keys
}

fn get_parents_ordered_from_root(file: &Path) -> Vec<PathBuf> {
    let mut parents = Vec::new();
    let mut current = file.parent();

    while let Some(parent) = current {
        parents.push(parent.to_path_buf());
        current = parent.parent();
    }

    parents.reverse();
    parents
}

// TODO: how to change mut
fn add_parent_analysis_to_tree_of_analyses(
    analysis_in_tree_of_analyses: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    last_parent_of_file_id: &mut AnalysisInTreeId,
    parent: PathBuf,
) {
    let parent_analysis_id = String::from(parent.to_string_lossy());
    if analysis_in_tree_of_analyses.get_mut(&parent_analysis_id).is_none() {
        let mut parent_analysis = AnalysisInTree{
            file_name: parent.to_string_lossy().into_owned(),
            metrics: BTreeMap::new(),
            parent_id: None,
            folder_content_id: Some(vec![]),
        };
        connect_grand_father_with_parent_in_tree_of_analyses(analysis_in_tree_of_analyses, &parent_analysis_id, &mut parent_analysis);
        analysis_in_tree_of_analyses.insert(parent_analysis_id.clone(), parent_analysis);
    }
    *last_parent_of_file_id = parent_analysis_id;
}

// TODO: how to change mut
fn connect_grand_father_with_parent_in_tree_of_analyses(
    analysis_in_tree_of_analyses: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    parent_analysis_id: &str,
    parent_analysis: &mut AnalysisInTree,
) {
    if let Some(grand_parent) = PathBuf::from(parent_analysis.file_name.clone()).parent() {
        let grand_parent_id = String::from(grand_parent.to_path_buf().to_string_lossy());
        if let Some(grand_parent_analysis) = analysis_in_tree_of_analyses.get_mut(&grand_parent_id) {
            grand_parent_analysis
                .folder_content_id
                .get_or_insert(vec![])
                .push(parent_analysis_id.to_owned());
            parent_analysis.parent_id = Some(grand_parent_id);
        }
    }
}

// SMELLS: too many arguments
// SMELLS: do two different things
fn create_and_push_file_analysis_into_tree_of_analysis(
    metrics: &Vec<Box<dyn IMetric>>,
    analyses_in_tree_of_analyses: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    file: &Path,
    last_parent_of_file_id: &mut AnalysisInTreeId,
) -> AnalysisInTree {
    let result_file_metrics = get_file_metrics_score(metrics, file);
    let file_analysis = AnalysisInTree {
        file_name: file.to_string_lossy().into_owned(),
        metrics: result_file_metrics,
        parent_id: Some(last_parent_of_file_id.to_owned()),
        folder_content_id: None,
    };
    let file_id = String::from(file.to_path_buf().to_string_lossy());
    analyses_in_tree_of_analyses.insert(file_id.clone(), file_analysis.clone());

    let last_parent_content = analyses_in_tree_of_analyses
        .get_mut(last_parent_of_file_id)
        .unwrap()
        .folder_content_id
        .get_or_insert(vec![]);
    last_parent_content.push(file_id);
    file_analysis
}

fn get_file_metrics_score(
    metrics: &Vec<Box<dyn IMetric>>,
    file: &Path,
) -> BTreeMap<&'static str, Option<MetricsValueAggregable>> {
    let mut result_file_metrics = BTreeMap::new();
    for metric in metrics {
        let result_metric_analyze = match metric.analyze(file) {
            Ok(file_metric) => {
                MetricsValueAggregable::new(MetricsValueType::Score(file_metric))
            },
            Err(error) => {
                MetricsValueAggregable::new(MetricsValueType::Error(error.to_string()))
            },
        };
        result_file_metrics.insert(metric.get_key(), Some(result_metric_analyze));
    }
    result_file_metrics
}

// SMELLS: give file_analysis directly to add_file
fn propagate_file_scores_to_parents_analysis(
    analyses_in_tree_of_analyses: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    file_analysis: AnalysisInTree,
) {
    let parent_id = file_analysis.parent_id;
    add_file_metrics_to_parents_analysis(analyses_in_tree_of_analyses, parent_id, file_analysis.metrics);
}

fn add_file_metrics_to_parents_analysis(
    analyses_in_tree_of_analyses: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    parent_id: Option<String>,
    mut file_metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
) {
    if let Some(some_parent_id) = parent_id {
        if let Some(parent) = analyses_in_tree_of_analyses.get_mut(&*some_parent_id) {
            if parent.metrics.is_empty() || parent.metrics.iter().all(|(_, value)| value.is_none())
            {
                parent.metrics = file_metrics.clone();
            } else {
                aggregate_metrics(&mut file_metrics, parent)
            }
            let grand_father = parent.parent_id.clone();
            add_file_metrics_to_parents_analysis(analyses_in_tree_of_analyses, grand_father, file_metrics);
        }
    }
}

fn aggregate_metrics(
    file_metrics: &mut BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    parent: &mut AnalysisInTree,
) {
    let mut parent_metrics_iterable = parent.metrics.iter_mut();
    let mut file_scores_iterable = file_metrics.iter();

    while let (
        Some((_, Some(parent_aggregable))),
        Some((_, Some(file_aggregable)))
    ) = (parent_metrics_iterable.next(), file_scores_iterable.next())
    {
        parent_aggregable.aggregate(file_aggregable);
    }
}

// SMELLS: give directly tree_of_analysis
fn convert_analysis_in_tree_of_analyses_to_final_analysis(
    analyses_in_tree_of_analyses: HashMap<AnalysisInTreeId, AnalysisInTree>,
    root_id: &String,
) -> Analysis {
    let root_analysis = analyses_in_tree_of_analyses.get(root_id).unwrap();
    build_final_analysis_structure(root_analysis, &analyses_in_tree_of_analyses)
}

fn build_final_analysis_structure(
    current_analysis_in_tree: &AnalysisInTree,
    analyses_in_tree_of_analyses: &HashMap<AnalysisInTreeId, AnalysisInTree>,
) -> Analysis {
    let mut current_analysis = Analysis {
        file_name: String::from(
            PathBuf::from(current_analysis_in_tree.file_name.clone())
                .file_name()
                .unwrap()
                .to_string_lossy(),
        ),
        metrics: current_analysis_in_tree.metrics.clone(),
        folder_content: None,
    };
    if let Some(folder_content_id) = &current_analysis_in_tree.folder_content_id {
        if folder_content_id.is_empty() {
            current_analysis.folder_content = Some(BTreeMap::new());
        } else {
            for child_file_id in folder_content_id {
                if let Some(child_analysis_in_tree) = analyses_in_tree_of_analyses.get(child_file_id) {
                    let child_analysis = build_final_analysis_structure(child_analysis_in_tree, analyses_in_tree_of_analyses);
                    add_child_analysis_to_current_analysis_content(
                        &mut current_analysis,
                        &child_analysis,
                    );
                }
            }
        }
    }
    current_analysis
}

// TODO: mut
fn add_child_analysis_to_current_analysis_content(
    new_analysis: &mut Analysis,
    child_analysis: &Analysis,
) {
    if let Some(new_analysis_content) = new_analysis.folder_content.as_mut() {
        new_analysis_content.insert(child_analysis.file_name.clone(), child_analysis.clone());
    } else {
        new_analysis.folder_content = Some(BTreeMap::new());
        new_analysis
            .folder_content
            .as_mut()
            .unwrap()
            .insert(child_analysis.file_name.clone(), child_analysis.clone());
    }
}

/* **************************************************************** */

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use super::*;
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};

    pub struct FakeMetric {
        pub metric_key: &'static str,
        pub metric_value: u32
    }

    impl IMetric for FakeMetric {
        fn analyze(&self, _file_path: &Path) -> Result<u32, String> {
            Ok(self.metric_value)
        }
        fn get_key(&self) -> &'static str {
            self.metric_key
        }
    }

    impl FakeMetric {
        pub fn new(metric_value: u32) -> FakeMetric {
            // It's important to note that this approach has some potential risks.
            // Storing a dynamically allocated &'static str like this may lead to memory leaks
            // if not managed properly, as it bypasses Rust's usual memory management guarantees.
            // Ensure that the FakeMetric instances are properly dropped when no longer needed
            // to avoid leaking memory.
            let metric_key = Box::leak(Box::new(format!("fake{}", metric_value))) as &'static str;
            FakeMetric {
                metric_key,
                metric_value,
            }
        }
    }

    pub struct BrokenMetric {
        pub metric_key: &'static str
    }

    impl IMetric for BrokenMetric {
        fn analyze(&self, _file_path: &Path) -> Result<u32, String> {
            Err(String::from("Analysis error"))
        }
        fn get_key(&self) -> &'static str{
            self.metric_key
        }
    }

    impl BrokenMetric {
        pub fn new() -> BrokenMetric {
            BrokenMetric {
                metric_key: "broken",
            }
        }
    }

    fn build_analysis_structure(
        root_name: String,
        metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
        content: BTreeMap<String, Analysis>,
    ) -> Analysis {
        Analysis {
            file_name: root_name,
            metrics,
            folder_content: Some(content),
        }
    }

    #[test]
    fn analyse_internal_with_empty_root_and_empty_metrics() {
        // Given
        let root = PathBuf::from("root");
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(vec![]));

        // When
        let actual_result_analysis = do_internal_analysis(&root, fake_file_explorer, vec![]);

        // Then
        let expected_result_analysis = build_analysis_structure(
            root.to_string_lossy().to_string(),
            BTreeMap::new(),
            BTreeMap::new(),
        );

        assert_eq!(expected_result_analysis, actual_result_analysis);
    }
    #[test]
    fn analyse_internal_with_2_files_and_empty_metrics() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(&root).join("file1"),
            PathBuf::from(&root).join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics = vec![];

        // When
        let actual_result_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let first_file_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: BTreeMap::new(),
            folder_content: None,
        };
        let second_file_analysis = Analysis {
            file_name: String::from("file2"),
            metrics: BTreeMap::new(),
            folder_content: None,
        };
        let mut expected_file_analysis = BTreeMap::new();
        expected_file_analysis.insert(first_file_analysis.file_name.clone(), first_file_analysis);
        expected_file_analysis.insert(second_file_analysis.file_name.clone(), second_file_analysis);

        //let expected_file_analysis = vec![first_file_analysis, second_file_analysis];
        let expected_result_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: BTreeMap::new(),
            folder_content: Some(expected_file_analysis),
        };
        assert_eq!(expected_result_analysis, actual_result_analysis);
    }

    #[test]
    fn analyse_internal_with_1_file_and_fakemetric4_and_fakemetric10() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> =
            vec![Box::new(FakeMetric::new(4)), Box::new(FakeMetric::new(10))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake4", Some(MetricsValueAggregable::new(MetricsValueType::Score(4))));
        expected_metrics.insert("fake10", Some(MetricsValueAggregable::new(MetricsValueType::Score(10))));

        let expected_file_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.file_name.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn analyse_internal_with_1_file_and_brokenmetric() {
        // Given
        let root_name = "root";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(BrokenMetric::new())];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        let error_value = Some(MetricsValueAggregable::new(MetricsValueType::Error("Analysis error".to_string())));
        expected_metrics.insert("broken", error_value);

        let expected_file_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.file_name.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: expected_metrics.clone(),
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    #[test]
    fn analyse_internal_with_one_5_lines_file() {
        // Given
        let root = PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files");
        let files_to_analyze = vec![PathBuf::from("tests")
            .join("data")
            .join("folder_with_multiple_files")
            .join("file5.txt")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(LinesCountMetric::new())];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert(
            "lines_count",
            Some(MetricsValueAggregable::new(MetricsValueType::Score(5))),
        );

        let expected_file_analysis = Analysis {
            file_name: "file5.txt".to_string(),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.file_name.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            file_name: String::from("folder_with_multiple_files"),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis);
    }

    // agreggate tests
    #[test]
    fn internal_analyse_with_empty_root_and_fakemetric0() {
        // Given
        let files_to_analyze = vec![];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(0))];

        // When
        let actual_root_analysis =
            do_internal_analysis(&PathBuf::from("empty_root"), fake_file_explorer, metrics);

        // Then
        let expected_root_analysis = Analysis {
            file_name: String::from("empty_root"),
            metrics: btreemap!{"fake0" => None},
            folder_content: Some(BTreeMap::new()),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    fn internal_analyse_with_1_file_and_fakemetric1() {
        // Given
        let root_name = "root_with_1_file";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(&root).join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.file_name.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    fn analyse_internal_of_a_file_in_a_folder_with_fakemetric1() {
        // Given
        let root_name = "root_with_1_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![PathBuf::from(root_name).join("folder1").join("file1")];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.file_name.clone(), expected_file_analysis);
        let expected_folder1_analysis = Analysis {
            file_name: String::from("folder1"),
            metrics: expected_metrics.clone(),
            folder_content: Some(expected_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.file_name.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: expected_metrics,
            folder_content: Some(expected_root_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }

    #[test]
    fn analyse_internal_of_2_file_in_1_folder_in_root_with_fakemetric1_should_add_files_scores() {
        // Given
        /*        let fake_file_explorer = FakeFileExplorer('folder3/file1',
        'folder3/file2',
        'file3');*/
        let root_name = "root_with_2_file_in_1_folder";
        let root = PathBuf::from(root_name);
        let files_to_analyze = vec![
            PathBuf::from(root_name).join("folder1").join("file1"),
            PathBuf::from(root_name).join("folder1").join("file2"),
        ];
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(files_to_analyze));
        let metrics: Vec<Box<dyn IMetric>> = vec![Box::new(FakeMetric::new(1))];

        // When
        let actual_root_analysis = do_internal_analysis(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file1_analysis = Analysis {
            file_name: String::from("file1"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };
        let expected_file2_analysis = Analysis {
            file_name: String::from("file2"),
            metrics: expected_metrics.clone(),
            folder_content: None,
        };

        let mut expected_folder_metrics = BTreeMap::new();
        expected_folder_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(2))));

        let mut expected_folder1_analysis_content = BTreeMap::new();
        expected_folder1_analysis_content
            .insert(expected_file1_analysis.file_name.clone(), expected_file1_analysis);
        expected_folder1_analysis_content
            .insert(expected_file2_analysis.file_name.clone(), expected_file2_analysis);

        let expected_folder1_analysis = Analysis {
            file_name: String::from("folder1"),
            metrics: expected_folder_metrics.clone(),
            folder_content: Some(expected_folder1_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.file_name.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = Analysis {
            file_name: String::from(root_name),
            metrics: expected_folder_metrics,
            folder_content: Some(expected_root_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }
}