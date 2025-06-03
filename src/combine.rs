use anyhow::{Context, Result};
use clap::Parser;
use glob::glob;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::ContextConfig;

#[derive(Parser, Debug)]
pub struct CombineArgs {
    /// Glob patterns or specific paths of files to combine, relative to the context directory.
    #[clap(required = true, num_args = 1..)]
    pub patterns: Vec<String>,

    /// Whether to include headers for each file
    #[clap(long)]
    pub with_headers: bool,

    /// Custom format for the header. Use {path} as a placeholder for the file path.
    #[clap(long, default_value = "// File: {path}", requires = "with_headers")]
    pub header_format: String,

    /// Separator to insert between combined files.
    #[clap(long, default_value = "\n")]
    pub separator: String,

    /// Whether to copy the combined content to clipboard instead of writing to file
    #[clap(long)]
    pub clipboard: bool,

    /// Output file path. If not specified, writes to stdout
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Whether to sort files alphabetically before combining
    #[clap(long)]
    pub sort_files: bool,
}

pub fn handle_combine_action(
    args: &CombineArgs,
    config: &ContextConfig,
    verbose: bool,
) -> Result<()> {
    let context_dir_name = config.dest.as_deref().unwrap_or(".copilot-context");
    let base_path = PathBuf::from(context_dir_name);

    if verbose {
        println!("Combine: Context directory: {:?}", base_path);
    }

    let mut files_to_combine: Vec<PathBuf> = Vec::new();
    for pattern in &args.patterns {
        let full_pattern = base_path.join(pattern);
        let glob_pattern = full_pattern.to_str().context("Invalid pattern")?;
        if verbose {
            println!("Combine: Processing glob pattern: {}", glob_pattern);
        }
        for entry in glob(glob_pattern)? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        if verbose {
                            println!("Combine: Found file: {:?}", path);
                        }
                        files_to_combine.push(path);
                    }
                }
                Err(e) => eprintln!("Combine: Error matching glob pattern: {}", e),
            }
        }
    }

    if files_to_combine.is_empty() {
        println!("Combine: No files found matching the patterns.");
        return Ok(());
    }

    if args.sort_files {
        if verbose {
            println!(
                "Combine: Sorting {} files alphabetically.",
                files_to_combine.len()
            );
        }
        files_to_combine.sort();
    }

    let mut combined_content = String::new();
    for (index, file_path) in files_to_combine.iter().enumerate() {
        if verbose {
            println!("Combine: Reading file {:?}", file_path);
        }
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file {:?}", file_path))?;

        if args.with_headers {
            // Get relative path for header
            let relative_path = file_path.strip_prefix(&base_path).unwrap_or(file_path);
            let header = args
                .header_format
                .replace("{path}", relative_path.to_string_lossy().as_ref());
            combined_content.push_str(&header);
            combined_content.push('\n'); // Add a newline after the header
        }

        combined_content.push_str(&content);

        if index < files_to_combine.len() - 1 {
            if !content.ends_with('\n') && !args.separator.starts_with('\n') {
                combined_content.push('\n');
            }
            combined_content.push_str(&args.separator);
        }
    }

    if args.clipboard {
        if verbose {
            println!(
                "Combine: Copying to clipboard ({} bytes)...",
                combined_content.len()
            );
        }
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard
                    .set_text(combined_content.clone())
                    .with_context(|| "Failed to copy to clipboard")?;
                println!("Combined content copied to clipboard.");
            }
            Err(e) => eprintln!("Failed to access clipboard: {}", e),
        }
    } else if let Some(output_path) = &args.output {
        if verbose {
            println!(
                "Combine: Writing to output file {:?} ({} bytes)...",
                output_path,
                combined_content.len()
            );
        }
        fs::write(output_path, combined_content)
            .with_context(|| format!("Failed to write to output file {:?}", output_path))?;
        println!("Combined content written to {:?}", output_path);
    } else {
        if verbose {
            println!(
                "Combine: Printing to stdout ({} bytes)...",
                combined_content.len()
            );
        }
        io::stdout().write_all(combined_content.as_bytes())?;
        // Add a newline if stdout is a tty, to ensure prompt is on next line
        if atty::is(atty::Stream::Stdout) {
            println!();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_dummy_config(dest_path: &Path) -> ContextConfig {
        ContextConfig {
            version: 1,
            dest: Some(dest_path.to_string_lossy().into_owned()),
            sources: vec![],
        }
    }

    #[test]
    fn test_combine_simple() -> Result<()> {
        let dir = tempdir()?;
        let context_dir = dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir)?;

        let file1_path = context_dir.join("a.txt");
        let file2_path = context_dir.join("b.txt");
        fs::write(&file1_path, "Hello")?;
        fs::write(&file2_path, "World")?;

        let config = create_dummy_config(&context_dir);
        let output_file_path = dir.path().join("output.txt");
        let args = CombineArgs {
            patterns: vec!["a.txt".to_string(), "b.txt".to_string()],
            with_headers: false,
            header_format: String::new(),
            separator: "\n\n".to_string(),
            clipboard: false,
            output: Some(output_file_path.clone()),
            sort_files: true,
        };

        handle_combine_action(&args, &config, false)?;

        let mut combined_content = String::new();
        File::open(output_file_path)?.read_to_string(&mut combined_content)?;

        assert_eq!(combined_content, "Hello\n\nWorld");
        Ok(())
    }

    #[test]
    fn test_combine_with_headers_and_separator() -> Result<()> {
        let dir = tempdir()?;
        let context_dir = dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir)?;

        let file1_path = context_dir.join("a.rs");
        let file2_path = context_dir.join("b.rs");
        fs::write(&file1_path, "struct A;")?;
        fs::write(&file2_path, "struct B;")?;

        let config = create_dummy_config(&context_dir);
        let output_file_path = dir.path().join("output.txt");
        let args = CombineArgs {
            patterns: vec!["a.rs".to_string(), "b.rs".to_string()],
            with_headers: true,
            header_format: "// Path: {path}".to_string(),
            separator: "---\n".to_string(),
            clipboard: false,
            output: Some(output_file_path.clone()),
            sort_files: true,
        };

        handle_combine_action(&args, &config, false)?;

        let mut combined_content = String::new();
        File::open(output_file_path)?.read_to_string(&mut combined_content)?;

        // Since files are sorted, a.rs comes before b.rs
        // Relative paths are used in headers
        let expected_content = "// Path: a.rs\nstruct A;\n---\n// Path: b.rs\nstruct B;";
        assert_eq!(combined_content, expected_content);
        Ok(())
    }

    #[test]
    fn test_combine_sorting_disabled() -> Result<()> {
        let dir = tempdir()?;
        let context_dir = dir.path().join(".copilot-context");
        fs::create_dir_all(&context_dir)?;

        let config = create_dummy_config(&context_dir);
        let output_file_path = dir.path().join("output.txt");
        let args = CombineArgs {
            patterns: vec!["a.txt".to_string(), "b.txt".to_string()],
            with_headers: true,
            header_format: "// Path: {path}".to_string(),
            separator: "---\n".to_string(),
            clipboard: false,
            output: Some(output_file_path.clone()),
            sort_files: false,
        };

        let file_b_path = context_dir.join("b.txt");
        let file_a_path = context_dir.join("a.txt");
        fs::write(&file_b_path, "Content B")?;
        fs::write(&file_a_path, "Content A")?;

        handle_combine_action(&args, &config, false)?;

        let mut combined_content = String::new();
        File::open(output_file_path)?.read_to_string(&mut combined_content)?;

        // Order should depend on how glob returns them, then how they are pushed.
        // WalkDir, which glob uses internally usually yields sorted results by default on some OS,
        // but this is not guaranteed across all platforms.
        assert!(combined_content.contains("Content A"));
        assert!(combined_content.contains("Content B"));
        Ok(())
    }
}
