use glob::Pattern;
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct ContextConfig {
    pub version: u8,
    pub dest: Option<String>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Repo {
        name: String,
        repo: String,
        branch: Option<String>,
        dest: String,
        files: Option<Vec<String>>,
    },
    Url {
        name: String,
        url: String,
        dest: String,
        files: Option<Vec<String>>,
    },
    Path {
        name: String,
        path: String,
        dest: String,
        files: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone)]
pub enum FileRule {
    Keep(Pattern),
    Delete(Pattern),
}

pub fn parse_file_rules(files: &[String]) -> Vec<FileRule> {
    files
        .iter()
        .map(|s| {
            if let Some(rest) = s.strip_prefix('!') {
                FileRule::Delete(Pattern::new(rest).unwrap())
            } else {
                FileRule::Keep(Pattern::new(s).unwrap())
            }
        })
        .collect()
}

pub fn match_files_and_mark(
    root: &std::path::Path,
    rules: &[FileRule],
) -> Vec<(std::path::PathBuf, bool)> {
    let mut results = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let rel_path = entry.path().strip_prefix(root).unwrap();
        let rel_str = rel_path.to_string_lossy();
        let mut action = None;
        for rule in rules {
            match rule {
                FileRule::Delete(pat) if pat.matches(&rel_str) => {
                    action = Some(false);
                    break;
                }
                FileRule::Keep(pat) if pat.matches(&rel_str) => {
                    action = Some(true);
                    break;
                }
                _ => {}
            }
        }
        // Default: keep
        let keep = action.unwrap_or(true);
        results.push((entry.path().to_path_buf(), keep));
    }
    results
}

pub fn load_config(path: &str) -> Result<ContextConfig, Box<dyn std::error::Error>> {
    let f = std::fs::read_to_string(path)?;
    let config: ContextConfig = toml::from_str(&f)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_file_rules() {
        let rules = parse_file_rules(&[
            "*".to_string(),
            "!foo.log".to_string(),
            "bar.txt".to_string(),
        ]);
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_match_files_and_mark_keep_and_delete() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("foo.log");
        let file2 = dir.path().join("bar.txt");
        let file3 = dir.path().join("baz.md");
        fs::write(&file1, "test").unwrap();
        fs::write(&file2, "test").unwrap();
        fs::write(&file3, "test").unwrap();
        // Correct rule order: delete first, then keep all, then explicit keep
        let rules = parse_file_rules(&[
            "!foo.log".to_string(),
            "*".to_string(),
            "bar.txt".to_string(),
        ]);
        let results = match_files_and_mark(dir.path(), &rules)
            .into_iter()
            .filter(|(p, _)| p.parent() == Some(dir.path()) && p.is_file())
            .collect::<Vec<_>>();
        let mut keep = vec![];
        let mut delete = vec![];
        for (path, k) in &results {
            let fname = path.file_name().map(|f| f.to_string_lossy().to_string());
            if let Some(name) = fname {
                if *k {
                    keep.push(name);
                } else {
                    delete.push(name);
                }
            }
        }
        assert!(keep.contains(&"bar.txt".to_string()));
        assert!(keep.contains(&"baz.md".to_string()));
        assert!(delete.contains(&"foo.log".to_string()));
    }
}
