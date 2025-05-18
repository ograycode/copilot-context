use std::fs;
use std::io;
use std::path::Path;

pub fn copy_local(src: &str, dest: &str, verbose: bool) -> io::Result<()> {
    let src_path = Path::new(src);
    let dest_path = Path::new(dest);
    eprintln!("🔍 current dir = {}", std::env::current_dir()?.display());
    if !src_path.exists() {
        println!(
            "copilot-context: source path '{}' does not exist",
            src_path.display()
        );
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path '{}' does not exist", src),
        ));
    }
    if src_path.is_file() {
        fs::create_dir_all(dest_path.parent().unwrap())?;
        println!("copilot-context: copying file {} -> {}", src, dest);
        println!("dest_path.exists() = {}", dest_path.exists());
        println!(
            "dest_path.parent().unwrap() = {}",
            dest_path.parent().unwrap().display()
        );
        fs::copy(&src_path, &dest_path).expect("Failed to copy file");
    } else if src_path.is_dir() {
        copy_dir_all(src_path, dest_path, verbose)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dest: &Path, verbose: bool) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_entry = entry.path();
        let dest_entry = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&src_entry, &dest_entry, verbose)?;
        } else {
            if verbose {
                println!(
                    "copilot-context: copying file {} -> {}",
                    src_entry.display(),
                    dest_entry.display()
                );
            }
            fs::copy(&src_entry, &dest_entry)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_copy_file_success() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("source.txt");
        let dest_path = dir.path().join("dest.txt");
        let mut file = File::create(&src_path).unwrap();
        writeln!(file, "hello world").unwrap();
        copy_local(
            src_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
            false,
        )
        .unwrap();
        let content = fs::read_to_string(dest_path).unwrap();
        assert!(content.contains("hello world"));
    }

    #[test]
    fn test_copy_file_not_found() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("does_not_exist.txt");
        let dest_path = dir.path().join("dest.txt");
        let result = copy_local(
            src_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_directory_success() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src_dir");
        let dest_dir = dir.path().join("dest_dir");
        fs::create_dir(&src_dir).unwrap();
        let file1 = src_dir.join("a.txt");
        let file2 = src_dir.join("b.txt");
        File::create(&file1).unwrap().write_all(b"A").unwrap();
        File::create(&file2).unwrap().write_all(b"B").unwrap();
        copy_local(src_dir.to_str().unwrap(), dest_dir.to_str().unwrap(), true).unwrap();
        assert!(dest_dir.join("a.txt").exists());
        assert!(dest_dir.join("b.txt").exists());
        let a = fs::read_to_string(dest_dir.join("a.txt")).unwrap();
        let b = fs::read_to_string(dest_dir.join("b.txt")).unwrap();
        assert_eq!(a, "A");
        assert_eq!(b, "B");
    }
}
