use glob::Pattern;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextConfig {
    pub version: u8,
    pub dest: Option<String>,
    pub sources: Vec<Source>,
}

impl ContextConfig {
    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }
    pub fn remove_source(&mut self, name: &str) -> bool {
        let orig_len = self.sources.len();
        self.sources.retain(|src| src.name() != name);
        self.sources.len() < orig_len
    }
    pub fn update_source(&mut self, name: &str, update: SourceUpdate) -> bool {
        if let Some(src) = self.sources.iter_mut().find(|s| s.name() == name) {
            src.apply_update(update);
            true
        } else {
            false
        }
    }
}

pub struct SourceUpdate {
    pub repo: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub dest: Option<String>,
    pub branch: Option<String>,
    pub files: Option<Vec<String>>,
    pub script: Option<String>,
}

impl SourceUpdate {
    pub fn from_args(
        repo: Option<String>,
        url: Option<String>,
        path: Option<String>,
        dest: Option<String>,
        branch: Option<String>,
        files: Option<Vec<String>>,
        script: Option<String>,
    ) -> Self {
        Self {
            repo,
            url,
            path,
            dest,
            branch,
            files,
            script,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn make_source(
    kind: &str,
    name: String,
    repo: Option<String>,
    url: Option<String>,
    path: Option<String>,
    dest: String,
    branch: Option<String>,
    files: Option<Vec<String>>,
    script: Option<String>,
) -> Source {
    match kind {
        "repo" => Source::Repo {
            name,
            repo: repo.expect("--repo required for repo kind"),
            branch,
            dest,
            files,
        },
        "url" => Source::Url {
            name,
            url: url.expect("--url required for url kind"),
            dest,
            files,
        },
        "path" => Source::Path {
            name,
            path: path.expect("--path required for path kind"),
            dest,
            files,
        },
        "sh" => Source::Sh {
            name,
            script: script.expect("--script required for sh kind"),
            dest,
        },
        _ => panic!("Unknown kind: {}", kind),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Sh {
        name: String,
        script: String,
        dest: String,
    },
}

impl Source {
    pub fn name(&self) -> &str {
        match self {
            Source::Repo { name, .. } => name,
            Source::Url { name, .. } => name,
            Source::Path { name, .. } => name,
            Source::Sh { name, .. } => name,
        }
    }
    pub fn apply_update(&mut self, update: SourceUpdate) {
        match self {
            Source::Repo {
                repo,
                branch,
                dest,
                files,
                ..
            } => {
                if let Some(r) = update.repo {
                    *repo = r;
                }
                if let Some(b) = update.branch {
                    *branch = Some(b);
                }
                if let Some(d) = update.dest {
                    *dest = d;
                }
                if let Some(f) = update.files {
                    *files = Some(f);
                }
            }
            Source::Url {
                url, dest, files, ..
            } => {
                if let Some(u) = update.url {
                    *url = u;
                }
                if let Some(d) = update.dest {
                    *dest = d;
                }
                if let Some(f) = update.files {
                    *files = Some(f);
                }
            }
            Source::Path {
                path, dest, files, ..
            } => {
                if let Some(p) = update.path {
                    *path = p;
                }
                if let Some(d) = update.dest {
                    *dest = d;
                }
                if let Some(f) = update.files {
                    *files = Some(f);
                }
            }
            Source::Sh { script, dest, .. } => {
                if let Some(s) = update.script {
                    *script = s;
                }
                if let Some(d) = update.dest {
                    *dest = d;
                }
            }
        }
    }
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

    if rules.is_empty() {
        return results;
    }

    let keep_patterns: Vec<&Pattern> = rules
        .iter()
        .filter_map(|r| match r {
            FileRule::Keep(p) => Some(p),
            _ => None,
        })
        .collect();

    let delete_patterns: Vec<&Pattern> = rules
        .iter()
        .filter_map(|r| match r {
            FileRule::Delete(p) => Some(p),
            _ => None,
        })
        .collect();

    for entry_result in WalkDir::new(root).min_depth(1).into_iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let rel_str = rel_path.to_string_lossy();

        let mut should_be_kept: bool;

        if !keep_patterns.is_empty() {
            should_be_kept = keep_patterns.iter().any(|p| p.matches(&rel_str));
        } else {
            should_be_kept = true;
        }

        if should_be_kept
            && !delete_patterns.is_empty()
            && delete_patterns.iter().any(|p| p.matches(&rel_str))
        {
            should_be_kept = false;
        }

        results.push((path.to_path_buf(), should_be_kept));
    }
    results
}

pub fn load_config(path: &str) -> Result<ContextConfig, Box<dyn std::error::Error>> {
    let f = std::fs::read_to_string(path)?;
    let config: ContextConfig = toml::from_str(&f)?;
    Ok(config)
}

pub fn save_config(path: &str, config: &ContextConfig) -> Result<(), Box<dyn std::error::Error>> {
    let toml = toml::to_string_pretty(config)?;
    std::fs::write(path, toml)?;
    Ok(())
}

pub fn write_default_config_if_missing(path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::path::Path;
    if Path::new(path).exists() {
        return Ok(false);
    }
    let default = ContextConfig {
        version: 1,
        dest: Some(".copilot-context".to_string()),
        sources: vec![
            Source::Repo {
                name: "example-repo".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                branch: Some("main".to_string()),
                dest: "vendor/example-repo".to_string(),
                files: Some(vec!["*".to_string()]),
            },
            Source::Url {
                name: "example-url".to_string(),
                url: "https://example.com/file.txt".to_string(),
                dest: "example/file.txt".to_string(),
                files: None,
            },
            Source::Path {
                name: "local-notes".to_string(),
                path: "README.md".to_string(),
                dest: "vendor/notes/README.md".to_string(),
                files: None,
            },
            Source::Sh {
                name: "example-script".to_string(),
                script: "echo \'Hello from example script!\'\necho \'Current directory: $(pwd)\'"
                    .to_string(),
                dest: ".".to_string(),
            },
        ],
    };
    save_config(path, &default)?;
    Ok(true)
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

        // Test with both keep and delete patterns
        let rules = parse_file_rules(&[
            "!foo.log".to_string(),
            "*.txt".to_string(),
            "*.md".to_string(),
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

        // Test with only keep patterns - only matching files should be kept
        let rules = parse_file_rules(&["*.txt".to_string()]);
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
        assert!(!keep.contains(&"baz.md".to_string()));
        assert!(!keep.contains(&"foo.log".to_string()));

        // Test with only delete patterns - all non-matching files should be kept
        let rules = parse_file_rules(&["!*.txt".to_string()]);
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

        assert!(!keep.contains(&"bar.txt".to_string()));
        assert!(keep.contains(&"baz.md".to_string()));
        assert!(keep.contains(&"foo.log".to_string()));
    }

    #[test]
    fn test_source_serialization_roundtrip() {
        let sources = vec![
            Source::Repo {
                name: "repo1".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                branch: Some("main".to_string()),
                dest: "vendor/repo1".to_string(),
                files: Some(vec!["*".to_string()]),
            },
            Source::Url {
                name: "url1".to_string(),
                url: "https://example.com/file.txt".to_string(),
                dest: "file.txt".to_string(),
                files: None,
            },
            Source::Path {
                name: "path1".to_string(),
                path: "README.md".to_string(),
                dest: "notes/README.md".to_string(),
                files: None,
            },
            Source::Sh {
                name: "script1".to_string(),
                script: "echo \"hello world\"".to_string(),
                dest: "scripts".to_string(),
            },
        ];
        let config = ContextConfig {
            version: 1,
            dest: Some(".copilot-context".to_string()),
            sources: sources.clone(),
        };
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: ContextConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.dest, Some(".copilot-context".to_string()));
        assert_eq!(parsed.sources.len(), 4);
        match &parsed.sources[0] {
            Source::Repo {
                name,
                repo,
                branch,
                dest,
                files,
            } => {
                assert_eq!(name, "repo1");
                assert_eq!(repo, "https://github.com/example/repo.git");
                assert_eq!(branch, &Some("main".to_string()));
                assert_eq!(dest, "vendor/repo1");
                assert_eq!(files.as_ref().unwrap()[0], "*");
            }
            _ => panic!("Expected repo source"),
        }
        match &parsed.sources[1] {
            Source::Url {
                name,
                url,
                dest,
                files,
            } => {
                assert_eq!(name, "url1");
                assert_eq!(url, "https://example.com/file.txt");
                assert_eq!(dest, "file.txt");
                assert!(files.is_none());
            }
            _ => panic!("Expected url source"),
        }
        match &parsed.sources[2] {
            Source::Path {
                name,
                path,
                dest,
                files,
            } => {
                assert_eq!(name, "path1");
                assert_eq!(path, "README.md");
                assert_eq!(dest, "notes/README.md");
                assert!(files.is_none());
            }
            _ => panic!("Expected path source"),
        }
        match &parsed.sources[3] {
            Source::Sh { name, script, dest } => {
                assert_eq!(name, "script1");
                assert_eq!(script, "echo \"hello world\"");
                assert_eq!(dest, "scripts");
            }
            _ => panic!("Expected sh source"),
        }
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("context.toml");
        let config = ContextConfig {
            version: 1,
            dest: Some(".copilot-context".to_string()),
            sources: vec![Source::Repo {
                name: "repo1".to_string(),
                repo: "https://github.com/example/repo.git".to_string(),
                branch: None,
                dest: "vendor/repo1".to_string(),
                files: None,
            }],
        };
        save_config(file_path.to_str().unwrap(), &config).unwrap();
        let loaded = load_config(file_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.dest, Some(".copilot-context".to_string()));
        assert_eq!(loaded.sources.len(), 1);
        match &loaded.sources[0] {
            Source::Repo { name, repo, .. } => {
                assert_eq!(name, "repo1");
                assert_eq!(repo, "https://github.com/example/repo.git");
            }
            _ => panic!("Expected repo source"),
        }
    }

    #[test]
    fn test_file_rule_patterns() {
        let rules = parse_file_rules(&["foo/*.rs".to_string(), "!foo/bar.rs".to_string()]);
        match &rules[0] {
            FileRule::Keep(pat) => assert!(pat.matches("foo/main.rs")),
            _ => panic!("Expected Keep pattern"),
        }
        match &rules[1] {
            FileRule::Delete(pat) => assert!(pat.matches("foo/bar.rs")),
            _ => panic!("Expected Delete pattern"),
        }

        // Test with only keep patterns
        let keep_rules = parse_file_rules(&["*.rs".to_string()]);
        let has_keep_patterns = keep_rules
            .iter()
            .any(|rule| matches!(rule, FileRule::Keep(_)));
        assert!(has_keep_patterns);

        // Test with only delete patterns
        let delete_rules = parse_file_rules(&["!*.rs".to_string()]);
        let has_keep_patterns = delete_rules
            .iter()
            .any(|rule| matches!(rule, FileRule::Keep(_)));
        assert!(!has_keep_patterns);
    }

    #[test]
    fn test_match_files_and_mark_default_keep() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("foo.txt");
        fs::write(&file1, "test").unwrap();

        // Empty rules case
        let rules = parse_file_rules(&[]); // No rules
        let results = match_files_and_mark(dir.path(), &rules);
        assert!(results.is_empty()); // With no rules, no files should be processed

        // Only delete rules case - files not matching delete pattern should be kept
        let rules = parse_file_rules(&["!bar.txt".to_string()]); // Only delete rule
        let results = match_files_and_mark(dir.path(), &rules);
        let mut found = false;
        for (path, keep) in results {
            if path.file_name().map(|f| f == "foo.txt").unwrap_or(false) {
                assert!(keep); // foo.txt should be kept since it doesn't match !bar.txt
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_load_config_error() {
        let result = load_config("/nonexistent/path/to/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_config_error() {
        let config = ContextConfig {
            version: 1,
            dest: Some(".copilot-context".to_string()),
            sources: vec![],
        };
        // Try to save to a directory path, which should fail
        let result = save_config("/", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_default_config_if_missing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("context.toml");
        // Ensure the file does not exist
        assert!(!file_path.exists());
        // Write default config
        let result = write_default_config_if_missing(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
        // Ensure the file was created
        assert!(file_path.exists());
        // Check the contents
        let config = load_config(file_path.to_str().unwrap()).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.dest, Some(".copilot-context".to_string()));
        assert_eq!(config.sources.len(), 4);
    }

    #[test]
    fn test_write_default_config_if_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("context.toml");
        // Create the file first
        fs::write(&file_path, "version = 1\n").unwrap();
        // Try to write default config
        let result = write_default_config_if_missing(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should not overwrite, so returns false
    }

    #[test]
    fn test_empty_rules() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("test.txt");
        fs::write(&file1, "test").unwrap();

        // Empty rules should result in no files being kept
        let rules = parse_file_rules(&[]);
        let results = match_files_and_mark(dir.path(), &rules);

        // Should be empty because we return early with empty results
        assert!(results.is_empty());
    }
}
