use std::path::Path;
use std::process::Command;

pub fn fetch_repo(
    repo_url: &str,
    dest: &str,
    branch: Option<&str>,
    sparse: Option<&[String]>,
    verbose: bool,
) -> Result<(), String> {
    if Path::new(dest).exists() {
        if verbose {
            println!("git: destination '{}' already exists, skipping clone", dest);
        }
        return Ok(());
    }

    // Clone repo (shallow by default)
    let mut clone_args = vec!["clone", "--depth=1"];
    if let Some(branch) = branch {
        clone_args.push("--branch");
        clone_args.push(branch);
    }
    clone_args.push(repo_url);
    clone_args.push(dest);

    if verbose {
        println!("git: running git {:?}", clone_args);
    }
    let status = Command::new("git")
        .args(&clone_args)
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !status.success() {
        return Err(format!("git clone failed for {repo_url}"));
    }

    // rm .git directory
    let git_dir = Path::new(dest).join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir)
            .map_err(|e| format!("failed to remove .git directory: {e}"))?;
        if verbose {
            println!("git: removed .git directory");
        }
    }

    Ok(())
}
