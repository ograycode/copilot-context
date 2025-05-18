# copilot-context

Curated, reproducible local context folder generator for Copilot and LLM tools.

## Usage

1. Edit `context.toml` to declare your sources.
2. Run `cargo run` to generate `.copilot_context/`.

## Features

- Git sparse clone (selective paths, shallow clone)
- File filtering (glob matching, ignore dotfiles)
- Copy from local paths
- Download raw files via HTTP(S)
- Flatten folder structure / rename output dirs
- Output summary log or JSON metadata for each run

## Issues

- sparse option doesn't work yet.
- path option where copying a directory isn't tested yet.
