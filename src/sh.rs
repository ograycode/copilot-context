use std::fs;
use std::path::Path;
use std::process::Command;

/// Run a shell script in the specified destination directory
///
/// # Arguments
/// * `script` - The shell script content to execute
/// * `dest` - The destination directory, relative to the current directory
/// * `verbose` - Whether to print verbose output
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
pub fn run_script(script: &str, dest: &Path, verbose: bool) -> Result<(), String> {
    if script.trim().is_empty() {
        return Err("Empty script provided".to_string());
    }

    if verbose {
        println!("copilot-context: Running script in '{}'", dest.display());
        println!("copilot-context: Script content:");
        println!("--- SCRIPT START ---");
        println!("{}", script);
        println!("--- SCRIPT END ---");
    }

    // Get the current directory
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Create target directory
    let target_dir = current_dir.join(dest);

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create directory {}: {}", target_dir.display(), e))?;
    }

    // Execute the script
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(&target_dir)
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    // Handle output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if verbose {
        if !stdout.is_empty() {
            println!("copilot-context: Script STDOUT:");
            println!("--- STDOUT START ---");
            println!("{}", stdout);
            println!("--- STDOUT END ---");
        }
        if !stderr.is_empty() {
            println!("copilot-context: Script STDERR:");
            println!("--- STDERR START ---");
            println!("{}", stderr);
            println!("--- STDERR END ---");
        }
    }

    // Check exit status
    if !output.status.success() {
        return Err(format!(
            "Script execution failed with status {}:\nStdout:\n{}\nStderr:\n{}",
            output.status, stdout, stderr
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_run_script_success() {
        let temp_dir = tempdir().unwrap();
        let script = "echo \'Hello, World!\' > test.txt";
        let dest_path = temp_dir.path();

        let result = run_script(script, dest_path, false);
        assert!(result.is_ok());

        let file_path = dest_path.join("test.txt");
        assert!(file_path.exists());

        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents.trim(), "Hello, World!");
    }

    #[test]
    fn test_run_script_creates_directories() {
        let temp_dir = tempdir().unwrap();
        let nested_dir = "nested/directory";
        let nested_path_buf = temp_dir.path().join(nested_dir);
        let script = "echo \'Hello from nested directory\' > test.txt";

        let result = run_script(script, &nested_path_buf, false);
        assert!(result.is_ok());

        let file_path = nested_path_buf.join("test.txt");
        assert!(file_path.exists());
    }

    #[test]
    fn test_run_script_error() {
        let temp_dir = tempdir().unwrap();
        // Command that should fail on most systems
        let script = "exit 1";
        let dest_path = temp_dir.path();

        let result = run_script(script, dest_path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_script_empty() {
        let temp_dir = tempdir().unwrap();
        let script = "";
        let dest_path = temp_dir.path();

        let result = run_script(script, dest_path, false);
        assert!(result.is_err());
    }
}
