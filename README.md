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
cargo run
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

- List sources: `cargo run -- list`
- Add a source: `cargo run -- add --name foo --kind repo --repo <url> --dest <dir>`
- Remove a source: `cargo run -- remove --name foo`
- Update a source: `cargo run -- update --name foo --repo <new-url>`

See `cargo run -- --help` for all options.

## Requirements

- Rust (edition 2021)
- macOS, Linux, or Windows

## License

MIT
