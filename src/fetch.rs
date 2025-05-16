use std::fs::File;
use std::io::copy;
use std::path::Path;

/// Downloads a file from the given URL to the destination path.
pub fn fetch_url(url: &str, dest: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("fetch_url: downloading {} to {}", url, dest);
    }
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }
    let parent = Path::new(dest).parent();
    if let Some(parent) = parent {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let mut content = response;
    copy(&mut content, &mut file)?;
    if verbose {
        println!("fetch_url: download complete");
    }
    Ok(())
}
