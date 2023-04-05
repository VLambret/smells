use std::fs::File;
use std::io::Read;
use crate::analysis::models::Analysis;

pub fn summary_lines_count_metric(folder_contents: &Vec<Analysis>) -> usize {
    folder_contents
        .iter()
        .filter_map(|content| {
            if let Analysis::FileAnalysis(file) = content {
                Some(file.metrics.lines_count)
            } else if let Analysis::FolderAnalysis(folder) = content{
                Some(folder.metrics.lines_count)
            }else{
                None
            }
        })
        .sum()
}

pub fn compute_lines_count_metric(file: &mut File) -> Result<usize, std::io::Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(count_lines(content))
}

fn count_lines(content: String) -> usize {
    content.lines().count()
}

#[cfg(test)]
mod tests{
    use rstest::rstest;
    use super::*;
    #[rstest(input, expected,
    case("", 0),
    case("line1", 1),
    case("line1\nline2", 2),
    case("line1\nline2\nline3", 3),
    case("\n", 1),
    case("\n\n\n", 3)
    )]
    fn test_count_lines(input: &str, expected: usize) {
        let line_count = count_lines(input.to_owned());
        assert_eq!(line_count, expected);
    }
}
