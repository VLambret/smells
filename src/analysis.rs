use crate::data_sources::file_explorer::IFileExplorer;
use crate::metrics::line_count::count_lines;
use crate::metrics::metric::IMetric;
use crate::metrics::{line_count, social_complexity};
use fxhash::FxHasher;
use maplit::hashmap;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fs::{read_dir, DirEntry, File};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::string::String;

/*
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct AnalysisId(u32);

#[derive(Debug, PartialEq)]
pub struct AnalysisTree {
    pub analysis: HashMap<AnalysisId, AnalysisNew>,
}
impl Hash for AnalysisTree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.analysis {
            key.hash(state);
            value.id.hash(state);
            value.metrics.hash(state);
            value.parent.hash(state);
            value.folder_content.hash(state);
        }
    }
}
*/

// TODO: distinguish root to folders
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnalysisNew {
    pub id: String,
    pub metrics: BTreeMap<String, MetricsValueType>,
    pub parent: Option<String>,
    pub folder_content: Option<Vec<String>>,
}

pub type AnalysisId = String;

// TODO: distinguish root to folders
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Analysis {
    // TODO: Good luck :-)
    //pub parent: Option<&Analysis>,
    pub id: String,
    pub metrics: BTreeMap<String, MetricsValueType>,
    pub content: Option<BTreeMap<String, Analysis>>,
}

pub type AnalysisError = String;

// TODO: rename variants
#[derive(Debug, Hash, Deserialize, Clone, PartialEq)]
pub enum MetricsValueType {
    Score(u32),
    Error(AnalysisError),
}

impl Serialize for MetricsValueType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MetricsValueType::Score(score) => serializer.serialize_u32(*score),
            MetricsValueType::Error(error) => serializer.serialize_str(error),
        }
    }
}

pub fn do_analysis(root: PathBuf) -> Analysis {
    analyse_root(root)
}

fn analyse_folder(item: PathBuf) -> Analysis {
    let folder_content: Vec<Analysis> = sort_files_of_a_path(&item)
        .iter()
        .filter(|f| can_file_be_analysed(&f.path()))
        .map(analyse)
        .collect();

    let mut metrics_content = BTreeMap::new();
    metrics_content.insert(
        "lines_count".to_string(),
        MetricsValueType::Score(line_count::summary_lines_count_metric(&folder_content)),
    );
    metrics_content.insert(
        "social_complexity".to_string(),
        MetricsValueType::Score(social_complexity::social_complexity("")),
    );

    Analysis {
        id: extract_analysed_item_key(&item),
        metrics: metrics_content,
        content: create_btreemap_from_analysis_vector(folder_content),
    }
}

fn create_btreemap_from_analysis_vector(
    analysis_vector: Vec<Analysis>,
) -> Option<BTreeMap<String, Analysis>> {
    let result = analysis_vector
        .iter()
        .map(|a| (a.id.clone(), a.to_owned()))
        .collect::<BTreeMap<_, _>>();
    Some(result)
}

fn analyse(entry: &DirEntry) -> Analysis {
    let analysis: Analysis = if entry.path().is_file() {
        analyse_file(entry)
    } else {
        analyse_folder(entry.path())
    };
    analysis
}

fn analyse_root(root: PathBuf) -> Analysis {
    analyse_folder(root)
}

fn analyse_internal(
    root: &Path,
    file_explorer: Box<dyn IFileExplorer<Item = PathBuf>>,
    metrics: Vec<Box<dyn IMetric>>,
) -> Analysis {
    let root_analysis = AnalysisNew {
        id: String::from(root.to_string_lossy()),
        metrics: BTreeMap::new(),
        parent: None,
        folder_content: Some(vec![]),
    };
    // TODO: hash
    let root_id = hash_pathbuf_to_string(&root.to_path_buf());
    let mut analysis_tree: HashMap<AnalysisId, AnalysisNew> =
        hashmap! {root_id.clone() => root_analysis.clone() };

    for file in file_explorer.discover() {
        let parents = get_parents_ordered_from_root(&file);
        let mut last_parent_of_file_id: AnalysisId = root_id.clone();
        for parent in parents {
            add_parent_analysis_to_analysis_tree(
                &mut analysis_tree,
                &mut last_parent_of_file_id,
                parent,
            );
        }
        let file_analysis = create_and_push_file_analysis_into_analysis_tree(
            &metrics,
            &mut analysis_tree,
            &file,
            &mut last_parent_of_file_id,
        );
        propagate_file_scores_to_parents(&mut analysis_tree, file_analysis);
    }
    //let root_analysis_in_tree = analysis_tree.get(&*root_id).unwrap();

    convert_analysis_hashmap_to_final_analysis(analysis_tree.clone(), &root_id)
}

fn propagate_file_scores_to_parents(
    analysis_tree: &mut HashMap<AnalysisId, AnalysisNew>,
    file_analysis: AnalysisNew,
) {
    let parent_id = file_analysis.parent;
    add_file_metrics_to_parent(analysis_tree, parent_id, file_analysis.metrics);
}

fn add_file_metrics_to_parent(
    analysis_tree: &mut HashMap<AnalysisId, AnalysisNew>,
    parent_id: Option<String>,
    mut file_metrics: BTreeMap<String, MetricsValueType>,
) {
    if let Some(some_parent_id) = parent_id {
        if let Some(parent) = analysis_tree.get_mut(&*some_parent_id) {
            if parent.metrics.is_empty() {
                parent.metrics = file_metrics.clone();
            } else {
                let mut parent_scores = parent.metrics.iter_mut();
                let mut file_scores = file_metrics.iter_mut();

                loop {
                    match (parent_scores.next(), file_scores.next()) {
                        (
                            Some((_, MetricsValueType::Score(ref mut parent_score))),
                            Some((_, MetricsValueType::Score(ref mut file_score))),
                        ) => {
                            *parent_score += *file_score;
                        }
                        (None, None) => break,
                        _ => {}
                    }
                }
            }
            let grand_father = parent.parent.clone();
            add_file_metrics_to_parent(analysis_tree, grand_father, file_metrics);
        }
    }
}

fn create_and_push_file_analysis_into_analysis_tree(
    metrics: &Vec<Box<dyn IMetric>>,
    analysis_tree: &mut HashMap<AnalysisId, AnalysisNew>,
    file: &Path,
    last_parent_of_file_id: &mut AnalysisId,
) -> AnalysisNew {
    let result_file_metrics = get_metrics_score(metrics, file);
    let file_analysis = AnalysisNew {
        id: file.to_string_lossy().into_owned(),
        metrics: result_file_metrics,
        parent: Some(last_parent_of_file_id.to_owned()),
        folder_content: None,
    };
    let file_id = hash_pathbuf_to_string(&file.to_path_buf());
    analysis_tree.insert(file_id.clone(), file_analysis.clone());

    let last_parent_content = analysis_tree
        .get_mut(last_parent_of_file_id)
        .unwrap()
        .folder_content
        .get_or_insert(vec![]);
    last_parent_content.push(file_id);
    file_analysis
}

// TODO: how to change mut
fn add_parent_analysis_to_analysis_tree(
    analysis_tree: &mut HashMap<AnalysisId, AnalysisNew>,
    last_parent_of_file_id: &mut AnalysisId,
    parent: PathBuf,
) {
    let parent_analysis_id = hash_pathbuf_to_string(&parent);
    if analysis_tree.get_mut(&parent_analysis_id).is_none() {
        // Create parent analysis
        let mut parent_analysis = AnalysisNew {
            id: parent.to_string_lossy().into_owned(),
            metrics: BTreeMap::new(),
            parent: None,
            folder_content: Some(vec![]),
        };
        // Get parent of parent if exists
        connect_grand_father_with_parent(analysis_tree, &parent_analysis_id, &mut parent_analysis);
        analysis_tree.insert(parent_analysis_id.clone(), parent_analysis);
    }
    *last_parent_of_file_id = parent_analysis_id;
}

fn hash_pathbuf_to_string(path: &PathBuf) -> String {
    let mut hasher = FxHasher::default();
    path.hash(&mut hasher);
    hasher.finish().to_string()
}

// TODO: how to change mut
fn connect_grand_father_with_parent(
    analysis_tree: &mut HashMap<AnalysisId, AnalysisNew>,
    parent_analysis_id: &str,
    parent_analysis: &mut AnalysisNew,
) {
    if let Some(grand_parent) = PathBuf::from(parent_analysis.id.clone()).parent() {
        let grand_parent_id = hash_pathbuf_to_string(&grand_parent.to_path_buf());
        //String::from(grand_parent.to_string_lossy());
        if let Some(grand_parent_analysis) = analysis_tree.get_mut(&grand_parent_id) {
            // Add parent as child of parent of parent
            grand_parent_analysis
                .folder_content
                .get_or_insert(vec![])
                .push(parent_analysis_id.to_owned());
            // Add grand_parent as parent of parent
            parent_analysis.parent = Some(grand_parent_id);
        }
    }
}

fn get_metrics_score(
    metrics: &Vec<Box<dyn IMetric>>,
    file: &Path,
) -> BTreeMap<String, MetricsValueType> {
    let mut result_file_metrics = BTreeMap::new();
    for metric in metrics {
        let result_metric_analyze = match metric.analyze(file) {
            Ok(file_metric) => MetricsValueType::Score(file_metric),
            Err(error) => MetricsValueType::Error(error.to_string()),
        };
        result_file_metrics.insert(metric.get_key(), result_metric_analyze);
    }
    result_file_metrics
}

fn convert_analysis_hashmap_to_final_analysis(
    analysis_hashmap: HashMap<AnalysisId, AnalysisNew>,
    root_id: &String,
) -> Analysis {
    let root_analysis = analysis_hashmap.get(root_id).unwrap();
    build_final_analysis_structure(root_analysis, &analysis_hashmap)
}

fn build_final_analysis_structure(
    node: &AnalysisNew,
    analysis_tree: &HashMap<AnalysisId, AnalysisNew>,
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

// sort files based on the entry names
fn sort_files_of_a_path(item: &PathBuf) -> Vec<DirEntry> {
    // TODO: handle unwrap()
    let dir_result = read_dir(item);
    let dir = dir_result.unwrap();
    let mut entries: Vec<_> = dir.map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    entries
}

// create the file content for the analysis
fn analyse_file(entry: &DirEntry) -> Analysis {
    // TODO: handle unwrap()
    let path = entry.path();
    let mut file = File::open(&path).unwrap();
    // TODO: remove expect and make metric optional to handle errors when an executable is analyzed
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let mut metrics_content = BTreeMap::new();
    metrics_content.insert(
        "lines_count".to_string(),
        MetricsValueType::Score(count_lines(content)),
    );
    metrics_content.insert(
        "social_complexity".to_string(),
        MetricsValueType::Score(social_complexity::social_complexity("")),
    );

    Analysis {
        id: extract_analysed_item_key(&path),
        metrics: metrics_content,
        content: None,
    }
}

fn can_file_be_analysed(item: &Path) -> bool {
    let file_name = match item.file_name() {
        Some(file) => file,
        _ => return false,
    };
    !file_name.to_string_lossy().starts_with('.')
}

fn extract_analysed_item_key(item: &Path) -> String {
    let item_as_os_str = item.as_os_str();
    let item_key = match item.file_name() {
        Some(item_name) => item_name.to_owned(),
        _ => item_as_os_str.to_owned(),
    };
    item_key.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_sources::file_explorer::{FakeFileExplorer, IFileExplorer};
    use crate::metrics::line_count::LinesCountMetric;

    pub struct FakeMetric {
        pub metric_key: String,
        pub metric_value: u32,
    }

    impl IMetric for FakeMetric {
        fn analyze(&self, _file_path: &Path) -> Result<u32, String> {
            Ok(self.metric_value)
        }
        fn get_key(&self) -> String {
            self.metric_key.to_owned()
        }
    }

    impl FakeMetric {
        pub fn new(metric_value: u32) -> FakeMetric {
            FakeMetric {
                metric_key: format!("fake{}", metric_value),
                metric_value,
            }
        }
    }

    pub struct BrokenMetric {
        pub metric_key: String,
    }

    impl IMetric for BrokenMetric {
        fn analyze(&self, _file_path: &Path) -> Result<u32, String> {
            Err(String::from("Analysis error"))
        }
        fn get_key(&self) -> String {
            self.metric_key.to_owned()
        }
    }

    impl BrokenMetric {
        pub fn new() -> BrokenMetric {
            BrokenMetric {
                metric_key: String::from("broken"),
            }
        }
    }

    fn build_analysis_structure(
        root_name: String,
        metrics: BTreeMap<String, MetricsValueType>,
        content: BTreeMap<String, Analysis>,
    ) -> Analysis {
        let expected_result_analysis = Analysis {
            id: root_name,
            metrics,
            content: Some(content),
        };
        expected_result_analysis
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
        expected_metrics.insert(String::from("fake4"), MetricsValueType::Score(4));
        expected_metrics.insert(String::from("fake10"), MetricsValueType::Score(10));

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
        let error_value = MetricsValueType::Error("Analysis error".to_string());
        expected_metrics.insert(String::from("broken"), error_value);

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
        expected_metrics.insert(String::from("lines_count"), MetricsValueType::Score(5));

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
        let root_name = "empty_root";
        let root = PathBuf::from(root_name);
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
            metrics: BTreeMap::new(),
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
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

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
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

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
        expected_metrics.insert(String::from("fake1"), MetricsValueType::Score(1));

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
        expected_folder_metrics.insert(String::from("fake1"), MetricsValueType::Score(2));

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
