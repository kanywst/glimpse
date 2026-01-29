# Glim

> Code review at the speed of thought.
> A paradigm shift from Line-Oriented to Intent-Oriented diffs.

[![CI](https://github.com/glim-rs/glim/actions/workflows/ci.yml/badge.svg)](https://github.com/glim-rs/glim/actions)
[![Crates.io](https://img.shields.io/crates/v/glim.svg)](https://crates.io/crates/glim)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

Glim is a next-generation Git CLI tool designed for modern development workflows. It moves beyond the traditional patch format to provide a semantic, visual, and context-aware understanding of code changes.

![Glim Demo](./assets/demo.gif)

## Concept

Software complexity has outpaced our tools. `git diff` shows you *what* lines changed, but Glim tells you *why* and *where* it matters.

*   **Galaxy View (Zoom L1)**: Heatmap of changes. Identifies impact zones instantly.
*   **Structure View (Zoom L2)**: Reads functions, not just files. Displays a tree of modified symbols powered by Tree-sitter. Supports interactive staging.
*   **Logic View (Zoom L3)**: Context-aware diffs with syntax highlighting and noise reduction.
*   **GitHub PR Mode**: Reviews Pull Requests directly in the terminal.

## Installation

### Cargo

```bash
cargo install glim
```

### From Source

```bash
git clone https://github.com/glim-rs/glim.git
cd glim
make build
cargo install --path .
```

## Usage

### Local Development

View working tree changes in the current directory:

```bash
glim .
```

Target a specific repository:

```bash
glim ~/dev/my-project
```

### GitHub Review

Review a Pull Request by URL or ID (Requires `gh` CLI):

```bash
glim owner/repo#123
```

## Controls

| Key | Action |
| --- | --- |
| `j` / `k` | Navigate items |
| `Enter` | Zoom In (Galaxy -> Structure -> Logic) |
| `Backspace` | Zoom Out |
| `Space` | Stage / Unstage File |
| `q` | Quit |

## Technology Stack

Built on the cutting edge of the Rust ecosystem (2026 Standard).

*   **Core**: Rust 2024 Edition
*   **TUI**: Ratatui v0.30
*   **Parsing**: Tree-sitter v0.26
*   **Git**: git2 (libgit2)
*   **Async**: Tokio v1.49

## Contributing

We enforce strict quality standards via our DevSecOps pipeline.

```bash
# Setup environment
make setup

# Run full CI check
make check
```

## License

MIT © [Glim Contributors](https://github.com/glim-rs/glim/graphs/contributors)