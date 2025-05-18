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
