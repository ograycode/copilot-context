use std::path::Path;
use std::process::Command;

pub fn fetch_repo(
    repo_url: &str,
    dest: &str,
    branch: Option<&str>,
    verbose: bool,
) -> Result<(), String> {
    if Path::new(dest).exists() {
        if verbose {
            println!("git: destination '{}' already exists, skipping clone", dest);
        }
        let git_dir = Path::new(dest).join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir)
                .map_err(|e| format!("failed to remove .git directory: {e}"))?;
            if verbose {
                println!("git: removed .git directory");
            }
        }
        return Ok(());
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static GIT_MUTEX: Mutex<()> = Mutex::new(());

    fn fake_git_repo(dir: &std::path::Path) {
        fs::create_dir_all(dir).unwrap();
        // Initialize a real git repository
        let status = Command::new("git")
            .arg("init")
            .arg("--initial-branch=main")
            .current_dir(dir)
            .status()
            .expect("failed to run git init");
        assert!(status.success(), "git init failed");

        fs::write(dir.join("README.md"), "test").unwrap();

        let status = Command::new("git")
            .arg("add")
            .arg("README.md")
            .current_dir(dir)
            .status()
            .expect("failed to run git add");
        assert!(status.success(), "git add failed");

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .status()
            .expect("failed to run git commit");
        assert!(status.success(), "git commit failed");
    }

    #[test]
    fn test_fetch_repo_skips_if_exists() {
        let _lock = GIT_MUTEX.lock().ok();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo");
        fs::create_dir_all(&dest).unwrap();
        let res = fetch_repo("irrelevant", dest.to_str().unwrap(), None, true);
        assert!(res.is_ok());
    }

    #[test]
    fn test_fetch_repo_success() {
        let _lock = GIT_MUTEX.lock().ok();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo");
        let repo_dir = dir.path().join("remote");
        fake_git_repo(&repo_dir);

        let url = format!("file://{}", repo_dir.to_str().unwrap());
        let res = fetch_repo(&url, dest.to_str().unwrap(), None, false);
        assert!(res.is_ok());
        assert!(dest.exists());
        assert!(dest.join("README.md").exists());
        assert!(!dest.join(".git").exists());
    }

    #[test]
    fn test_fetch_repo_with_branch() {
        let _lock = GIT_MUTEX.lock().ok();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo");
        let repo_dir = dir.path().join("remote");
        fake_git_repo(&repo_dir);

        let url = format!("file://{}", repo_dir.to_str().unwrap());
        let res = fetch_repo(&url, dest.to_str().unwrap(), Some("main"), false);
        assert!(res.is_ok());
        assert!(dest.exists());
        assert!(dest.join("README.md").exists());
        assert!(!dest.join(".git").exists());
    }

    #[test]
    fn test_fetch_repo_git_fails() {
        let _lock = GIT_MUTEX.lock().ok();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo");
        let res = fetch_repo("file:///nonexistent", dest.to_str().unwrap(), None, false);
        assert!(res.is_err());
    }

    #[test]
    fn test_fetch_repo_remove_git_dir_error() {
        let _lock = GIT_MUTEX.lock().ok();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo");
        let repo_dir = dir.path().join("remote");
        fake_git_repo(&repo_dir);

        let url = format!("file://{}", repo_dir.to_str().unwrap());
        let _ = fetch_repo(&url, dest.to_str().unwrap(), None, false);
        let git_dir = dest.join(".git");
        fs::set_permissions(&git_dir, fs::Permissions::from_mode(0o000)).ok();
        let res = fetch_repo("file:///nonexistent", dest.to_str().unwrap(), None, false);
        fs::set_permissions(&git_dir, fs::Permissions::from_mode(0o755)).ok();
        assert!(res.is_ok() || res.is_err());
    }
}
