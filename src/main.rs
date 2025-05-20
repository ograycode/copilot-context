use clap::{Parser, Subcommand};

mod config;
mod copy;
mod fetch;
mod git;

use config::{match_files_and_mark, parse_file_rules};

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all sources
    #[clap(about = "List all sources in the context configuration")]
    List,
    /// Add a new source
    #[clap(about = "Add a new source to the context configuration")]
    Add {
        #[clap(long, help = "Name of the source")]
        name: String,
        #[clap(long, help = "Kind of the source: repo, url, or path")]
        kind: String,
        #[clap(long, help = "Git repository URL (for kind=repo)")]
        repo: Option<String>,
        #[clap(long, help = "URL to fetch (for kind=url)")]
        url: Option<String>,
        #[clap(long, help = "Local path to copy (for kind=path)")]
        path: Option<String>,
        #[clap(long, help = "Destination directory inside context folder")]
        dest: String,
        #[clap(long, help = "Branch to use (for kind=repo)")]
        branch: Option<String>,
        #[clap(long, help = "File rules to include/exclude (glob patterns)")]
        files: Option<Vec<String>>,
    },
    /// Remove a source by name
    #[clap(about = "Remove a source from the context configuration by name")]
    Remove {
        #[clap(long, help = "Name of the source to remove")]
        name: String,
    },
    /// Update a source by name
    #[clap(about = "Update an existing source in the context configuration by name")]
    Update {
        #[clap(long, help = "Name of the source to update")]
        name: String,
        #[clap(long, help = "New git repository URL (for kind=repo)")]
        repo: Option<String>,
        #[clap(long, help = "New URL to fetch (for kind=url)")]
        url: Option<String>,
        #[clap(long, help = "New local path to copy (for kind=path)")]
        path: Option<String>,
        #[clap(long, help = "New destination directory inside context folder")]
        dest: Option<String>,
        #[clap(long, help = "New branch to use (for kind=repo)")]
        branch: Option<String>,
        #[clap(long, help = "New file rules to include/exclude (glob patterns)")]
        files: Option<Vec<String>>,
    },
}

#[derive(Parser, Debug)]
#[clap(
    name = "copilot-context",
    version = "0.1.0",
    author = "Jason OGray",
    about = "A tool to manage context folders for copilot"
)]
struct Cli {
    /// Path to the context.toml file
    #[clap(short, long, default_value = "context.toml")]
    config: String,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

fn main() {
    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        use config::{load_config, make_source, save_config, SourceUpdate};
        let mut config = load_config(&cli.config).expect("Failed to load config");
        match cmd {
            Commands::List => {
                for src in &config.sources {
                    println!("{:?}", src);
                }
                return;
            }
            Commands::Add {
                name,
                kind,
                repo,
                url,
                path,
                dest,
                branch,
                files,
            } => {
                let new_source = make_source(
                    kind,
                    name.clone(),
                    repo.clone(),
                    url.clone(),
                    path.clone(),
                    dest.clone(),
                    branch.clone(),
                    files.clone(),
                );
                config.add_source(new_source);
                save_config(&cli.config, &config).expect("Failed to save config");
                println!("Source added.");
                return;
            }
            Commands::Remove { name } => {
                if config.remove_source(name) {
                    save_config(&cli.config, &config).expect("Failed to save config");
                    println!("Source removed.");
                } else {
                    println!("No source found with name: {}", name);
                }
                return;
            }
            Commands::Update {
                name,
                repo,
                url,
                path,
                dest,
                branch,
                files,
            } => {
                let update = SourceUpdate::from_args(
                    repo.clone(),
                    url.clone(),
                    path.clone(),
                    dest.clone(),
                    branch.clone(),
                    files.clone(),
                );
                if config.update_source(name, update) {
                    save_config(&cli.config, &config).expect("Failed to save config");
                    println!("Source updated.");
                } else {
                    println!("No source found with name: {}", name);
                }
                return;
            }
        }
    }

    if cli.verbose {
        println!("copilot-context: verbose mode enabled");
    }
    let config_path = cli.config;
    if cli.verbose {
        println!("copilot-context: loading config from {}", config_path);
    }
    let mut config = config::load_config(&config_path).expect("Failed to load config");
    if cli.verbose {
        println!("copilot-context: loaded config: {:?}", config);
    }

    if config.dest.is_none() {
        config.dest = Some(".copilot-context".to_string());
    }

    if cli.verbose {
        println!("copilot-context: destination directory: {:?}", config.dest);
    }

    let dest = config.dest.as_ref().unwrap();

    std::fs::create_dir_all(dest).expect("Failed to create destination directory");
    std::env::set_current_dir(dest).expect("Failed to change working directory");

    // Update root to the new current directory after changing into .copilot-context
    let root = std::env::current_dir().expect("Failed to get current directory");

    println!("copilot-context: initializing context folder...");
    for source in config.sources {
        match source {
            config::Source::Repo {
                name,
                repo,
                branch,
                dest,
                files,
            } => {
                if cli.verbose {
                    println!("copilot-context: processing repo source: {}", name);
                }
                if let Err(e) = git::fetch_repo(&repo, &dest, branch.as_deref(), cli.verbose) {
                    eprintln!("copilot-context: error fetching repo {}: {}", name, e);
                }
                if let Some(files) = files {
                    if let Err(e) = files_func(&root.join(dest), files, cli.verbose) {
                        eprintln!("copilot-context: error applying files rules: {}", e);
                    }
                }
            }
            config::Source::Url {
                name,
                url,
                dest,
                files,
            } => {
                if cli.verbose {
                    println!("copilot-context: processing URL source: {}", name);
                }
                if let Err(e) = fetch::fetch_url(&url, &dest, cli.verbose) {
                    eprintln!("copilot-context: error fetching url {}: {}", name, e);
                }
                if let Some(files) = files {
                    if let Err(e) = files_func(&root, files, cli.verbose) {
                        eprintln!("copilot-context: error applying files rules: {}", e);
                    }
                }
            }
            config::Source::Path {
                name,
                path,
                dest,
                files,
            } => {
                if cli.verbose {
                    println!("copilot-context: processing path source: {}", name);
                }
                let project_root = std::env::current_dir()
                    .expect("Failed to get current directory")
                    .parent()
                    .unwrap()
                    .to_path_buf();
                let abs_source = project_root.join(path);
                let abs_source_str = abs_source
                    .as_path()
                    .to_str()
                    .expect("Failed to convert path to string");
                println!("copilot-context: absolute source path: {}", abs_source_str);
                if let Err(e) = copy::copy_local(abs_source_str, &dest, cli.verbose) {
                    eprintln!("copilot-context: error copying path {}: {}", name, e);
                }
                if let Some(files) = files {
                    if let Err(e) = files_func(&root, files, cli.verbose) {
                        eprintln!("copilot-context: error applying files rules: {}", e);
                    }
                }
            }
        }
    }
}

fn files_func(root: &std::path::Path, files: Vec<String>, verbose: bool) -> Result<(), String> {
    let rules = parse_file_rules(&files);
    let matches = match_files_and_mark(root, &rules);
    for (path, keep) in matches {
        if !keep {
            if path.exists() {
                let metadata = std::fs::metadata(&path).map_err(|e| {
                    format!("failed to get metadata for '{}': {}", path.display(), e)
                })?;
                if metadata.is_dir() {
                    std::fs::remove_dir_all(&path).map_err(|e| {
                        format!("failed to remove directory '{}': {}", path.display(), e)
                    })?;
                    if verbose {
                        println!("copilot-context: removed directory: {}", path.display());
                    }
                } else {
                    std::fs::remove_file(&path).map_err(|e| {
                        format!("failed to remove file '{}': {}", path.display(), e)
                    })?;
                    if verbose {
                        println!("copilot-context: removed file: {}", path.display());
                    }
                }
            } else if verbose {
                println!(
                    "copilot-context: path '{}' does not exist, skipping",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
