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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_fetch_url_success() {
        let mut server = Server::new();
        let server_address = server.url();
        let _m = server
            .mock("GET", "/testfile.txt")
            .with_status(200)
            .with_body("hello world")
            .create();

        let dir = tempdir().unwrap();
        let dest_path = dir.path().join("testfile.txt");
        let url = format!("{}/testfile.txt", &server_address);

        let result = fetch_url(&url, dest_path.to_str().unwrap(), true);
        assert!(result.is_ok());

        let mut file = fs::File::open(&dest_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "hello world");
    }

    #[test]
    fn test_fetch_url_http_error() {
        let mut server = Server::new();
        let server_address = server.url();
        let _m = server
            .mock("GET", "/notfound.txt")
            .with_status(404)
            .with_body("not found")
            .create();

        let dir = tempdir().unwrap();
        let dest_path = dir.path().join("notfound.txt");
        let url = format!("{}/notfound.txt", &server_address);

        let result = fetch_url(&url, dest_path.to_str().unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_url_creates_parent_dirs() {
        let mut server = Server::new();
        let server_address = server.url();
        let _m = server
            .mock("GET", "/nested/file.txt")
            .with_status(200)
            .with_body("nested content")
            .create();

        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("a/b/c/file.txt");
        let url = format!("{}/nested/file.txt", &server_address);

        let result = fetch_url(&url, nested_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let mut file = fs::File::open(&nested_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "nested content");
    }
}
