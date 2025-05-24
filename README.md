# copilot-context

A reproducible, declarative context folder generator for GitHub Copilot and LLM tools.

## Overview

`copilot-context` lets you curate a local folder containing only the files you want from multiple sources—Git repositories, URLs, or local paths—using a single TOML config. This is ideal for building reproducible, minimal context folders for LLMs, code assistants, or documentation tools.

- **Fetch from Git repos, URLs, or local paths**
- **Filter files with glob patterns** (include/exclude)
- **Flatten or rename output structure**
- **Declarative config: `context.toml`**
- **Fast, shallow Git clones**
- **Verbose logging and dry-run support**

## Quick Start

1. Edit `context.toml` to declare your sources and file rules.
2. Run:

```sh
copilot-context
```

This generates a `.copilot-context/` folder with your curated files.

## Example `context.toml`

```toml
version = 1
dest = ".copilot-context"

[[sources]]
type = "repo"
name = "cargo-lib"
repo = "https://github.com/rust-lang/cargo.git"
branch = "gh-pages"
dest = "vendor/cargo-gh-pages"
files = ["!CNAME"]

[[sources]]
type = "url"
name = "api-specs"
url = "https://raw.githubusercontent.com/softprops/action-gh-release/refs/heads/master/README.md"
dest = "softprops/action-gh-release/README.md"

[[sources]]
type = "path"
name = "local-notes"
path = "README.md"
dest = "vendor/notes/README.md"
```

## Features

- **Git sparse/shallow clone**: Only fetch what you need
- **Glob file filtering**: Include/exclude files with patterns
- **Copy from local paths**: Add your own files
- **Download raw files via HTTP(S)**
- **Flatten/rename output structure**
- **Summary log or JSON metadata output**

## CLI Usage

- List sources: `copilot-context list`
- Add a source: `copilot-context add --name foo --kind repo --repo <url> --dest <dir>`
- Remove a source: `copilot-context remove --name foo`
- Update a source: `copilot-context update --name foo --repo <new-url>`
- Initialize a config: `copilot-context init`

See `copilot-context --help` for all options.

## Install

### Download the Latest Release

1. Go to the [Releases page](https://github.com/ograycode/copilot-context/releases) and download the latest binary for your platform (macOS, Linux, or Windows).
2. Unpack the archive if necessary.
3. Move the binary to a directory in your `$PATH`, for example:
   ```sh
   mv copilot-context /usr/local/bin/
   chmod +x /usr/local/bin/copilot-context
   ```
4. Verify installation:
   ```sh
   copilot-context --help
   ```

### Build from Source

1. Ensure you have [Rust](https://rustup.rs/) installed (edition 2021).
2. Clone the repository:
   ```sh
   git clone https://github.com/ograycode/copilot-context.git
   cd copilot-context
   ```
3. Build the binary:
   ```sh
   cargo build --release
   ```
4. Move the binary to a directory in your `$PATH` (optional):
   ```sh
   cp target/release/copilot-context /usr/local/bin/
   ```
5. Verify installation:
   ```sh
   copilot-context --help
   ```

## Requirements

- Rust (edition 2021)
- macOS, Linux, or Windows

## License

MIT
