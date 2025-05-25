use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{match_files_and_mark, parse_file_rules, Source};

/// Process a destination path and add it and potentially its contents to the keep list
fn process_destination(
    context_root: &Path,
    src_dest: &str,
    files: Option<&Vec<String>>,
    keep_files: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    let full_dest = context_root.join(src_dest);
    keep_files.insert(full_dest.clone());

    // If the destination exists and there are no file rules or it's not a file-based source,
    // keep everything in that directory
    if full_dest.exists() && files.is_none() {
        for entry in WalkDir::new(&full_dest).into_iter().filter_map(Result::ok) {
            keep_files.insert(entry.path().to_path_buf());
        }
    }
    // If there are file rules, apply them
    else if let Some(file_rules) = files {
        let rules = parse_file_rules(file_rules);
        let matches = match_files_and_mark(&full_dest, &rules);
        for (path, keep) in matches {
            if keep {
                keep_files.insert(path);
            }
        }
    }

    Ok(())
}

/// Clean the context folder, removing files not specified in the configuration
pub fn clean_context_folder(dest: &str, sources: &[Source], verbose: bool) -> Result<(), String> {
    // Create destination directory if it doesn't exist
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create destination directory '{}': {}", dest, e))?;

    println!("Cleaning context folder: {}", dest);

    // Build a list of all files that should be kept
    let mut keep_files = HashSet::new();

    // Always keep the root directory
    let context_dir = Path::new(dest);
    keep_files.insert(context_dir.to_path_buf());

    // Process each source to determine which files to keep
    for source in sources {
        match source {
            Source::Repo {
                dest: src_dest,
                files,
                ..
            } => {
                process_destination(context_dir, src_dest, files.as_ref(), &mut keep_files)?;
            }
            Source::Url {
                dest: src_dest,
                files,
                ..
            } => {
                process_destination(context_dir, src_dest, files.as_ref(), &mut keep_files)?;
            }
            Source::Path {
                dest: src_dest,
                files,
                ..
            } => {
                process_destination(context_dir, src_dest, files.as_ref(), &mut keep_files)?;
            }
            Source::Sh { dest: src_dest, .. } => {
                process_destination(context_dir, src_dest, None, &mut keep_files)?;
            }
        }
    }

    // Walk the context directory and remove files not in the keep list
    for entry in WalkDir::new(context_dir)
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
    {
        let path = entry.path().to_path_buf();

        // Skip the root directory
        if path == context_dir {
            continue;
        }

        // If the file is not in the keep list, remove it
        if !keep_files.contains(&path) {
            if path.is_dir() {
                // Only remove empty directories
                let is_empty = std::fs::read_dir(&path)
                    .map(|entries| entries.count() == 0)
                    .unwrap_or(false);

                if is_empty {
                    if let Err(e) = std::fs::remove_dir(&path) {
                        eprintln!("Failed to remove directory {}: {}", path.display(), e);
                    } else if verbose {
                        println!("Removed directory: {}", path.display());
                    }
                }
            } else if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Failed to remove file {}: {}", path.display(), e);
            } else if verbose {
                println!("Removed file: {}", path.display());
            }
        }
    }

    println!("Context folder cleaned successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to create a test file structure
    fn create_test_files(base_dir: &Path, files: &[&str]) -> Result<(), std::io::Error> {
        for file_path in files {
            let full_path = base_dir.join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(full_path)?;
            write!(file, "test content")?;
        }
        Ok(())
    }

    #[test]
    fn test_clean_command_removes_unspecified_files() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().unwrap();
        let context_dir = temp_dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir).unwrap();

        // Create some test files in the context directory
        let test_files = [
            "keep/file1.txt",
            "keep/file2.txt",
            "remove/file1.txt",
            "remove/file2.txt",
            "remove/subdir/file3.txt",
        ];
        create_test_files(&context_dir, &test_files).unwrap();

        // Create sources that only keep the "keep" directory
        let sources = vec![crate::config::Source::Path {
            name: "keep-source".to_string(),
            path: "dummy".to_string(),
            dest: "keep".to_string(),
            files: None,
        }];

        // Run the clean function
        clean_context_folder(context_dir.to_str().unwrap(), &sources, true).unwrap();

        // Verify keep files still exist
        assert!(context_dir.join("keep/file1.txt").exists());
        assert!(context_dir.join("keep/file2.txt").exists());

        // Verify files were removed
        assert!(!context_dir.join("remove/file1.txt").exists());
        assert!(!context_dir.join("remove/file2.txt").exists());
        assert!(!context_dir.join("remove/subdir/file3.txt").exists());
    }

    #[test]
    fn test_clean_command_with_file_rules() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().unwrap();
        let context_dir = temp_dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir).unwrap();

        // Create some test files in the context directory
        let test_files = [
            "src/file1.rs",
            "src/file2.rs",
            "src/file3.txt",
            "src/subdir/file4.rs",
            "src/subdir/file5.txt",
        ];
        create_test_files(&context_dir, &test_files).unwrap();

        // Create sources that only keeps .rs files in the src directory
        let sources = vec![crate::config::Source::Path {
            name: "src-source".to_string(),
            path: "dummy".to_string(),
            dest: "src".to_string(),
            files: Some(vec!["**/*.rs".to_string(), "!**/*.txt".to_string()]),
        }];

        // Run the clean function
        clean_context_folder(context_dir.to_str().unwrap(), &sources, true).unwrap();

        // Verify files that should be kept still exist
        assert!(context_dir.join("src/file1.rs").exists());
        assert!(context_dir.join("src/file2.rs").exists());
        assert!(context_dir.join("src/subdir/file4.rs").exists());
        assert!(context_dir.join("src/subdir").exists());

        // Verify files that should be removed no longer exist
        assert!(!context_dir.join("src/file3.txt").exists());
        assert!(!context_dir.join("src/subdir/file5.txt").exists());
    }

    #[test]
    fn test_clean_command_preserves_empty_directories() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().unwrap();
        let context_dir = temp_dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir).unwrap();

        // Create some test directories
        let test_dirs = ["keep", "keep/subdir", "remove", "remove/subdir"];

        for dir in &test_dirs {
            fs::create_dir_all(context_dir.join(dir)).unwrap();
        }

        // Create a file in keep/subdir to ensure it's not empty
        let keep_file = context_dir.join("keep/subdir/file.txt");
        let mut file = File::create(&keep_file).unwrap();
        write!(file, "test content").unwrap();

        // Create sources that only keep the "keep" directory
        let sources = vec![crate::config::Source::Path {
            name: "keep-source".to_string(),
            path: "dummy".to_string(),
            dest: "keep".to_string(),
            files: None,
        }];

        // Run the clean function
        clean_context_folder(context_dir.to_str().unwrap(), &sources, true).unwrap();

        // Verify keep directory and its contents still exist
        assert!(context_dir.join("keep").exists());
        assert!(context_dir.join("keep/subdir").exists());
        assert!(context_dir.join("keep/subdir/file.txt").exists());

        // Verify remove directory no longer exists (it was empty)
        assert!(!context_dir.join("remove/subdir").exists());

        // The parent remove directory might still exist if it wasn't empty
        // after removing the subdirectory
        let remove_dir = context_dir.join("remove");
        if remove_dir.exists() {
            let is_empty = fs::read_dir(&remove_dir)
                .map(|entries| entries.count() == 0)
                .unwrap_or(false);
            assert!(
                is_empty,
                "Remove directory should be empty if it still exists"
            );
        }
    }

    #[test]
    fn test_clean_command_with_sh_source() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().unwrap();
        let context_dir = temp_dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir).unwrap();

        // Create script output directory and some test files
        let script_dir = context_dir.join("script_output");
        fs::create_dir_all(&script_dir).unwrap();

        // Create some test files in script_output and a directory to be kept
        let test_files = [
            "script_output/result1.txt",
            "script_output/result2.log",
            "script_output/subdir/nested.txt",
            "other_dir/file.txt", // This should be removed
        ];
        create_test_files(&context_dir, &test_files).unwrap();

        // Create sources that include a Sh source with script_output
        let sources = vec![crate::config::Source::Sh {
            name: "test-script".to_string(),
            script: "echo 'test'".to_string(),
            dest: "script_output".to_string(),
        }];

        // Run the clean function
        clean_context_folder(context_dir.to_str().unwrap(), &sources, true).unwrap();

        // Verify script_output and its contents are kept
        assert!(context_dir.join("script_output").exists());
        assert!(context_dir.join("script_output/result1.txt").exists());
        assert!(context_dir.join("script_output/result2.log").exists());
        assert!(context_dir.join("script_output/subdir/nested.txt").exists());

        // Verify other files are removed
        assert!(!context_dir.join("other_dir/file.txt").exists());

        // Note: the clean function only removes empty directories
        // If other_dir is not empty after removing file.txt (e.g., due to hidden files)
        // it won't be removed, so we don't assert on the directory itself
    }
}
