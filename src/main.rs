use clap::Parser;

mod config;
mod copy;
mod fetch;
mod git;

#[derive(Parser, Debug)]
#[clap(
    name = "copilot-context",
    version = "0.1.0",
    author = "Jason OGray",
    about = "A tool to manage context folders for copilot"
)]
struct Cli {
    /// Path to the context.yaml file
    #[clap(short, long, default_value = "context.yaml")]
    config: String,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    // TODO: CLI parsing, load config, dispatch sources
    let cli = Cli::parse();
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

    if !config.dest.is_some() {
        config.dest = Some(".copilot-context".to_string());
    }

    if cli.verbose {
        println!("copilot-context: destination directory: {:?}", config.dest);
    }

    let root = std::env::current_dir().expect("Failed to get current directory");
    let dest = config.dest.as_ref().unwrap();

    std::fs::create_dir_all(dest).expect("Failed to create destination directory");
    std::env::set_current_dir(dest).expect("Failed to change working directory");

    println!("copilot-context: initializing context folder...");
    for source in config.sources {
        match source {
            config::Source::Repo {
                name,
                repo,
                sparse,
                branch,
                dest,
            } => {
                if cli.verbose {
                    println!("copilot-context: processing repo source: {}", name);
                }
                if let Err(e) = git::fetch_repo(
                    &repo,
                    &dest,
                    branch.as_deref(),
                    sparse.as_deref(),
                    cli.verbose,
                ) {
                    eprintln!("copilot-context: error fetching repo {}: {}", name, e);
                }
            }
            config::Source::Url { name, url, dest } => {
                if cli.verbose {
                    println!("copilot-context: processing URL source: {}", name);
                }
                if let Err(e) = fetch::fetch_url(&url, &dest, cli.verbose) {
                    eprintln!("copilot-context: error fetching url {}: {}", name, e);
                }
            }
            config::Source::Path { name, path, dest } => {
                if cli.verbose {
                    println!("copilot-context: processing path source: {}", name);
                }
                let abs_source = root.join(path);
                let abs_source_str = abs_source
                    .as_path()
                    .to_str()
                    .expect("Failed to convert path to string");
                println!("copilot-context: absolute source path: {}", abs_source_str);
                if let Err(e) = copy::copy_local(&abs_source_str, &dest, cli.verbose) {
                    eprintln!("copilot-context: error copying path {}: {}", name, e);
                }
            }
        }
    }
}
