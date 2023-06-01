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

pub struct AnalysesTree {
    root_id: String,
    analyses: HashMap<AnalysisInTreeId, AnalysisInTree>,
}
pub type AnalysisInTreeId = String;

impl AnalysesTree {
    fn new(root_id: String, analyses: HashMap<AnalysisInTreeId, AnalysisInTree>) -> AnalysesTree {
        AnalysesTree {
            root_id,
            analyses,
        }
    }

    fn get_root(&self) -> &String {
        &self.root_id
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct AnalysisInTree {
    pub id: String,
    pub metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    pub parent: Option<String>,
    pub folder_content: Option<Vec<String>>,
}

/* **************************************************************** */

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Analysis {
    pub id: String,
    pub metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    pub content: Option<BTreeMap<String, Analysis>>,
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

    fn get_score(&self) -> MetricsValueType {
        self.value.clone()
    }

    fn aggregate(&self, other: MetricsValueAggregable) -> MetricsValueAggregable{
        todo!()
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
    analyse_internal(
        &root,
        Box::new(FileExplorer::new(&root)),
        vec![
            Box::new(LinesCountMetric::new()),
            Box::new(SocialComplexityMetric::new()),
        ],
    )
}

/* **************************************************************** */

fn analyse_internal(
    root: &Path,
    file_explorer: Box<dyn IFileExplorer<Item = PathBuf>>,
    metrics: Vec<Box<dyn IMetric>>,
) -> Analysis {
    let root_analysis = AnalysisInTree {
        id: String::from(root.to_string_lossy()),
        metrics: get_metrics_keys(&metrics),
        parent: None,
        folder_content: Some(vec![]),
    };
    let root_id: AnalysisInTreeId = String::from(root.to_path_buf().to_string_lossy());
    let mut analyses = AnalysesTree::new(root_id.clone(),
                                         hashmap! {root_id => root_analysis });

    for file in file_explorer.discover() {
        let parents = get_parents_ordered_from_root(&file);
        let mut last_parent_of_file_id: AnalysisInTreeId = analyses.get_root().to_owned();
        for parent in parents {
            add_parent_analysis_to_analysis_tree(
                &mut analyses.analyses,
                &mut last_parent_of_file_id,
                parent,
            );
        }
        let file_analysis = create_and_push_file_analysis_into_analysis_tree(
            &metrics,
            &mut analyses.analyses,
            &file,
            &mut last_parent_of_file_id,
        );
        propagate_file_scores_to_parents(&mut analyses.analyses, file_analysis);
    }
    convert_analysis_hashmap_to_final_analysis(analyses.analyses.clone(), analyses.get_root())
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
fn add_parent_analysis_to_analysis_tree(
    analysis_tree: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    last_parent_of_file_id: &mut AnalysisInTreeId,
    parent: PathBuf,
) {
    let parent_analysis_id = String::from(parent.to_string_lossy());
    if analysis_tree.get_mut(&parent_analysis_id).is_none() {
        let mut parent_analysis = AnalysisInTree{
            id: parent.to_string_lossy().into_owned(),
            metrics: BTreeMap::new(),
            parent: None,
            folder_content: Some(vec![]),
        };
        connect_grand_father_with_parent(analysis_tree, &parent_analysis_id, &mut parent_analysis);
        analysis_tree.insert(parent_analysis_id.clone(), parent_analysis);
    }
    *last_parent_of_file_id = parent_analysis_id;
}

// TODO: how to change mut
fn connect_grand_father_with_parent(
    analysis_tree: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    parent_analysis_id: &str,
    parent_analysis: &mut AnalysisInTree,
) {
    if let Some(grand_parent) = PathBuf::from(parent_analysis.id.clone()).parent() {
        let grand_parent_id = String::from(grand_parent.to_path_buf().to_string_lossy());
        if let Some(grand_parent_analysis) = analysis_tree.get_mut(&grand_parent_id) {
            grand_parent_analysis
                .folder_content
                .get_or_insert(vec![])
                .push(parent_analysis_id.to_owned());
            parent_analysis.parent = Some(grand_parent_id);
        }
    }
}

fn create_and_push_file_analysis_into_analysis_tree(
    metrics: &Vec<Box<dyn IMetric>>,
    analysis_tree: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    file: &Path,
    last_parent_of_file_id: &mut AnalysisInTreeId,
) -> AnalysisInTree {
    let result_file_metrics = get_metrics_score(metrics, file);
    let file_analysis = AnalysisInTree {
        id: file.to_string_lossy().into_owned(),
        metrics: result_file_metrics,
        parent: Some(last_parent_of_file_id.to_owned()),
        folder_content: None,
    };
    let file_id = String::from(file.to_path_buf().to_string_lossy());
    analysis_tree.insert(file_id.clone(), file_analysis.clone());

    let last_parent_content = analysis_tree
        .get_mut(last_parent_of_file_id)
        .unwrap()
        .folder_content
        .get_or_insert(vec![]);
    last_parent_content.push(file_id);
    file_analysis
}

fn get_metrics_score(
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

fn propagate_file_scores_to_parents(
    analysis_tree: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    file_analysis: AnalysisInTree,
) {
    let parent_id = file_analysis.parent;
    add_file_metrics_to_parent(analysis_tree, parent_id, file_analysis.metrics);
}

fn add_file_metrics_to_parent(
    analysis_tree: &mut HashMap<AnalysisInTreeId, AnalysisInTree>,
    parent_id: Option<String>,
    mut file_metrics: BTreeMap<&'static str, Option<MetricsValueAggregable>>,
) {
    if let Some(some_parent_id) = parent_id {
        if let Some(parent) = analysis_tree.get_mut(&*some_parent_id) {
            if parent.metrics.is_empty() || parent.metrics.iter().all(|(_, value)| value.is_none())
            {
                parent.metrics = file_metrics.clone();
            } else {
                aggregate_metrics(&mut file_metrics, parent)
            }
            let grand_father = parent.parent.clone();
            add_file_metrics_to_parent(analysis_tree, grand_father, file_metrics);
        }
    }
}

fn aggregate_metrics(
    file_metrics: &mut BTreeMap<&'static str, Option<MetricsValueAggregable>>,
    parent: &mut AnalysisInTree,
) {
    let mut parent_metrics_iterable = parent.metrics.iter_mut();
    let mut file_scores_iterable = file_metrics.iter_mut();

    loop {
        match (parent_metrics_iterable.next(), file_scores_iterable.next()) {
            (
                Some((_, Some(parent_aggregable))),
                Some((_, Some(file_aggregable))),
            ) => {
                if let MetricsValueType::Score(parent_score) = &mut parent_aggregable.value {
                    if let MetricsValueType::Score(file_score) = &mut file_aggregable.value {
                        *parent_score += *file_score;
                    }
                }
            }
            (None, None) => break,
            _ => {}
        }
    }
}

fn convert_analysis_hashmap_to_final_analysis(
    analysis_hashmap: HashMap<AnalysisInTreeId, AnalysisInTree>,
    root_id: &String,
) -> Analysis {
    let root_analysis = analysis_hashmap.get(root_id).unwrap();
    build_final_analysis_structure(root_analysis, &analysis_hashmap)
}

fn build_final_analysis_structure(
    node: &AnalysisInTree,
    analysis_tree: &HashMap<AnalysisInTreeId, AnalysisInTree>,
) -> Analysis {
    let mut current_analysis = Analysis {
        id: String::from(
            PathBuf::from(node.id.clone())
                .file_name()
                .unwrap()
                .to_string_lossy(),
        ),
        metrics: node.metrics.clone(),
        content: None,
    };
    if let Some(folder_content) = &node.folder_content {
        if folder_content.is_empty() {
            current_analysis.content = Some(BTreeMap::new());
        } else {
            for child_id in folder_content {
                if let Some(child_node) = analysis_tree.get(child_id) {
                    let child_analysis = build_final_analysis_structure(child_node, analysis_tree);
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
    if let Some(new_analysis_content) = new_analysis.content.as_mut() {
        new_analysis_content.insert(child_analysis.id.clone(), child_analysis.clone());
    } else {
        new_analysis.content = Some(BTreeMap::new());
        new_analysis
            .content
            .as_mut()
            .unwrap()
            .insert(child_analysis.id.clone(), child_analysis.clone());
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
            id: root_name,
            metrics,
            content: Some(content),
        }
    }

    #[test]
    fn analyse_internal_with_empty_root_and_empty_metrics() {
        // Given
        let root = PathBuf::from("root");
        let fake_file_explorer: Box<dyn IFileExplorer<Item = PathBuf>> =
            Box::new(FakeFileExplorer::new(vec![]));

        // When
        let actual_result_analysis = analyse_internal(&root, fake_file_explorer, vec![]);

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
        let actual_result_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let first_file_analysis = Analysis {
            id: String::from("file1"),
            metrics: BTreeMap::new(),
            content: None,
        };
        let second_file_analysis = Analysis {
            id: String::from("file2"),
            metrics: BTreeMap::new(),
            content: None,
        };
        let mut expected_file_analysis = BTreeMap::new();
        expected_file_analysis.insert(first_file_analysis.id.clone(), first_file_analysis);
        expected_file_analysis.insert(second_file_analysis.id.clone(), second_file_analysis);

        //let expected_file_analysis = vec![first_file_analysis, second_file_analysis];
        let expected_result_analysis = Analysis {
            id: String::from(root_name),
            metrics: BTreeMap::new(),
            content: Some(expected_file_analysis),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake4", Some(MetricsValueAggregable::new(MetricsValueType::Score(4))));
        expected_metrics.insert("fake10", Some(MetricsValueAggregable::new(MetricsValueType::Score(10))));

        let expected_file_analysis = Analysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.id.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: Some(expected_analysis_content),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        let error_value = Some(MetricsValueAggregable::new(MetricsValueType::Error("Analysis error".to_string())));
        expected_metrics.insert("broken", error_value);

        let expected_file_analysis = Analysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.id.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            id: String::from(root_name),
            metrics: expected_metrics.clone(),
            content: Some(expected_analysis_content),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert(
            "lines_count",
            Some(MetricsValueAggregable::new(MetricsValueType::Score(5))),
        );

        let expected_file_analysis = Analysis {
            id: "file5.txt".to_string(),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.id.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            id: String::from("folder_with_multiple_files"),
            metrics: expected_metrics,
            content: Some(expected_analysis_content),
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
            analyse_internal(&PathBuf::from("empty_root"), fake_file_explorer, metrics);

        // Then
        let expected_root_analysis = Analysis {
            id: String::from("empty_root"),
            metrics: btreemap!{"fake0" => None},
            content: Some(BTreeMap::new()),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file_analysis = Analysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.id.clone(), expected_file_analysis);

        let expected_root_analysis = Analysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: Some(expected_analysis_content),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file_analysis = Analysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_analysis_content = BTreeMap::new();
        expected_analysis_content.insert(expected_file_analysis.id.clone(), expected_file_analysis);
        let expected_folder1_analysis = Analysis {
            id: String::from("folder1"),
            metrics: expected_metrics.clone(),
            content: Some(expected_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.id.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = Analysis {
            id: String::from(root_name),
            metrics: expected_metrics,
            content: Some(expected_root_analysis_content),
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
        let actual_root_analysis = analyse_internal(&root, fake_file_explorer, metrics);

        // Then
        let mut expected_metrics = BTreeMap::new();
        expected_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(1))));

        let expected_file1_analysis = Analysis {
            id: String::from("file1"),
            metrics: expected_metrics.clone(),
            content: None,
        };
        let expected_file2_analysis = Analysis {
            id: String::from("file2"),
            metrics: expected_metrics.clone(),
            content: None,
        };

        let mut expected_folder_metrics = BTreeMap::new();
        expected_folder_metrics.insert("fake1", Some(MetricsValueAggregable::new(MetricsValueType::Score(2))));

        let mut expected_folder1_analysis_content = BTreeMap::new();
        expected_folder1_analysis_content
            .insert(expected_file1_analysis.id.clone(), expected_file1_analysis);
        expected_folder1_analysis_content
            .insert(expected_file2_analysis.id.clone(), expected_file2_analysis);

        let expected_folder1_analysis = Analysis {
            id: String::from("folder1"),
            metrics: expected_folder_metrics.clone(),
            content: Some(expected_folder1_analysis_content),
        };
        let mut expected_root_analysis_content = BTreeMap::new();
        expected_root_analysis_content.insert(
            expected_folder1_analysis.id.clone(),
            expected_folder1_analysis,
        );
        let expected_root_analysis = Analysis {
            id: String::from(root_name),
            metrics: expected_folder_metrics,
            content: Some(expected_root_analysis_content),
        };
        assert_eq!(expected_root_analysis, actual_root_analysis)
    }
}