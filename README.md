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

[[sources]]
type = "sh"
name = "example-script"
# The script content can be a multiline string
script = """
ls -la
echo \"Hello from a shell script!\"
# You can run any shell commands here
"""
# The destination for 'sh' type can be a relative path within the context folder
# or "." to run in the root of the context folder.
# The script will be executed with `sh -c "<script_content>"` in this directory.
dest = "script_output"
```

## Features

- **Git sparse/shallow clone**: Only fetch what you need
- **Glob file filtering**: Include/exclude files with patterns
- **Copy from local paths**: Add your own files
- **Download raw files via HTTP(S)**
- **Run shell scripts**: Execute custom shell commands and include their output or side-effects.
- **Combine files**: Concatenate multiple files from your context folder into a single string, with options for headers, separators, and clipboard output.
- **Flatten/rename output structure**
- **Summary log or JSON metadata output**
- **Clean command**: Remove files not specified in the configuration

## CLI Usage

- List sources: `copilot-context list`
- Add a source: `copilot-context add --name foo --kind repo --repo <url> --dest <dir>`
  - For `sh` kind: `copilot-context add --name my-script --kind sh --script "echo hello" --dest .`
- Remove a source: `copilot-context remove --name foo`
- Update a source: `copilot-context update --name foo --repo <new-url>`
  - For `sh` kind: `copilot-context update --name my-script --script "echo updated"`
- Initialize a config: `copilot-context init`
- Clean context folder: `copilot-context clean`
- Combine files: `copilot-context combine <patterns...> [options]`
  - Example: `copilot-context combine "src/**/*.rs" "docs/*.md" --output combined.txt --with-headers`
  - Example: `copilot-context combine "lib/**" --clipboard --separator "\n---\n"`
  - Options:
    - `patterns...`: One or more glob patterns or file paths to include (relative to the context directory).
    - `-o, --output <path>`: Write combined content to a file instead of stdout.
    - `-c, --clipboard`: Copy combined content to the clipboard (conflicts with `--output`).
    - `--with-headers`: Add a header comment before each file's content (e.g., `// File: src/main.rs`).
    - `--header-format <format>`: Custom header format. Use `{path}` for the file's relative path (default: `// File: {path}`). Requires `--with-headers`.
    - `--separator <string>`: String to insert between combined files (default: newline).
    - `--sort-files`: Sort files alphabetically before combining (default: true). Use `--no-sort-files` to disable.

  - Example Output:
    - If you have `file1.txt` with content `Hello` and `file2.txt` with content `World` in your context folder:
    - Default: `copilot-context combine "file*.txt"`
      ```
      Hello
      World
      ```
    - With headers: `copilot-context combine "file*.txt" --with-headers`
      ```
      // File: file1.txt
      Hello
      // File: file2.txt
      World
      ```
    - With custom header and separator: `copilot-context combine "file*.txt" --with-headers --header-format "### {path} ###" --separator "\n---\n"`
      ```
      ### file1.txt ###
      Hello
      ---
      ### file2.txt ###
      World
      ```

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
