# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`git-ombl` (Ultrathink Git Blame) is a Rust CLI tool that extends git blame functionality to trace the complete history of individual lines across all commits, not just the most recent change. The tool is built using Clean Architecture principles with clear separation between domain logic, use cases, and adapters.

## Development Commands

### Build and Run
```bash
# Standard Rust development
cargo build                              # Build the project
cargo run -- <file> <line_number>       # Run with arguments
./target/debug/git-ombl <file> <line_number> # Run built binary directly

# Nix development (recommended)
nix develop                              # Enter development shell
nix build                               # Build with Nix (skips integration tests)
nix build .#git-ombl-linux-x86_64      # Cross-compile for specific platform
```

### Testing
```bash
cargo test                               # Run all tests
cargo test <test_name>                   # Run specific test
cargo test -- --nocapture               # Run tests with output
cargo test adapters::git::tests::test_git_adapter_get_line_history -- --nocapture  # Single test with output

# Test specific categories
cargo test formatters::colored          # Test colored formatter
cargo test integration_tests            # Run integration tests (requires git repo)
```

### Formatting and Linting
- The project uses Nix flakes with treefmt for formatting
- Pre-commit hooks automatically format Rust code with rustfmt
- Formatting is enforced via git hooks (treefmt configuration in `nix/treefmt/default.nix`)
- Nix integration tests are skipped during `nix build` due to missing git repository

### Release Process
```bash
# Create and push version tag to trigger GitHub Actions release
git tag v1.0.0
git push origin v1.0.0
# Automatically builds for: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
```

## Architecture

### Clean Architecture Implementation
The codebase follows Clean Architecture with dependency inversion using a "Package by Feature + Core" structure:

**Core Layer (Stable/Abstract):**
- `src/core/line_history/` - Line history domain and abstractions
  - `domain.rs` - Core domain types (`LineHistory`, `LineEntry`, `ChangeType`)
  - `provider.rs` - `LineHistoryProvider` trait for data sources
  - `use_case.rs` - `LineHistoryUseCase<P: LineHistoryProvider>` business logic
- `src/core/formatting/` - Output formatting abstractions
  - `formatter.rs` - `OutputFormatter` trait

**Adapter Layer (Concrete/Variable):**
- `src/adapters/git.rs` - `GitAdapter` implements `LineHistoryProvider` using git2
- `src/formatters/` - Multiple formatters implement `OutputFormatter`
  - `json.rs` - JSON output using serde_json
  - `colored.rs` - Terminal colored output with ANSI colors
  - `yaml.rs` - YAML output using serde_yaml
  - `table.rs` - Tabular output using tabled crate

**Dependency Flow:** Adapters depend on core abstractions, never the reverse.

### Key Implementation Details

**Git History Traversal:**
- **Complete History**: Traces ALL commits that affected a line, not just the most recent
- Uses `repository.revwalk()` with `git2::Sort::TIME` to traverse commit history chronologically
- Implements `commit_affects_file()` and `commit_changes_line()` to filter relevant commits
- **1-based indexing**: Line numbers match standard editor conventions (not 0-based)
- **Filtering Support**: Multiple filtering methods integrated into commit traversal:
  - Hash-based filtering: supports full and abbreviated commit hashes (`--ignore-rev`)
  - Date-based filtering: ISO 8601, simple dates, and datetime formats (`--since`/`--until`)
  - Sort ordering: chronological (asc) or reverse-chronological (desc) (`--sort`)
- All filters work together and maintain performance with large repositories

**Testing Strategy:**
- Follows TDD (Test-Driven Development) with Red-Green-Refactor cycles
- Uses `tempfile` and `git2::Repository::init()` for isolated test repositories
- Tests use `mockall` for mocking in dev-dependencies but rely primarily on test implementations

**CLI Interface:**
- Uses `clap` with derive macros for argument parsing
- Supports `--format` option: `json`, `colored` (default), `yaml`, or `table`
- CLI structure: `git-ombl <file> <line_number> [OPTIONS]`
- **Filtering Options:**
  - `--sort asc|desc` - Sort order for commit history (default: asc)
  - `--ignore-rev <commit>` - Ignore changes made by specified revision(s), supports multiple instances
  - `--since <date>` - Show commits more recent than specified date
  - `--until <date>` - Show commits older than specified date
  - `--limit <number>` - Maximum number of commits to traverse

## Module Structure

```
src/
├── main.rs           # CLI entry point and argument parsing
├── lib.rs            # Module exports and re-exports
├── core/             # Core business logic and abstractions
│   ├── line_history/ # Line history domain and use cases
│   │   ├── domain.rs # Core domain types (LineHistory, LineEntry, ChangeType)
│   │   ├── provider.rs # LineHistoryProvider trait
│   │   ├── use_case.rs # LineHistoryUseCase business logic
│   │   └── mod.rs    # Module exports
│   ├── formatting/   # Output formatting abstractions
│   │   ├── formatter.rs # OutputFormatter trait
│   │   └── mod.rs    # Module exports
│   ├── types.rs      # Shared types (SortOrder enum)
│   └── mod.rs        # Core module exports
├── adapters/         # External system integrations
│   ├── git.rs        # Git integration via git2
│   └── mod.rs        # Adapter exports
└── formatters/       # Output format implementations
    ├── mod.rs        # Module exports
    ├── json.rs       # JSON formatter
    ├── colored.rs    # Terminal colored formatter
    ├── yaml.rs       # YAML formatter
    └── table.rs      # Table formatter
```

## Development Notes

### TDD Methodology
- **Strictly follows TDD**: Red-Green-Refactor cycles as prescribed by t_wada
- **Martin Fowler Refactoring**: Apply Extract Method, Remove Dead Code, and other techniques
- **Test Structure**: Each formatter and major feature has comprehensive test coverage
- **Architecture Evolution**: Refactored from flat structure to "Package by Feature + Core" for better separation of concerns

### Technical Details
- All domain types implement `serde::Serialize/Deserialize` for multi-format output
- Error handling uses `anyhow::Result` throughout for ergonomic error propagation
- Uses Rust edition 2024 with modern language features
- Git operations require the current directory to be a git repository
- **Lifetime Management**: `git2::Commit<'_>` requires explicit lifetime annotations
- **Dependencies**: serde_yaml, tabled, colored, chrono for various output formats

### Cross-Platform Build System
- **Nix Flakes**: Primary build system with reproducible builds
- **Multi-platform Support**: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
- **GitHub Actions**: Automated release pipeline triggered by version tags
- **Build Strategy**: Separate runners for optimal cross-compilation (Ubuntu for Linux/Windows, macOS for Darwin)

### Key Gotchas
- Line numbers are **1-based** in all user interfaces (not 0-based)
- `git2::Commit` objects borrow from the Repository, requiring careful lifetime management
- Pre-commit hooks auto-format with treefmt, may modify files during commit
- Test repositories use explicit timestamps for deterministic chronological ordering
- **Colored Tests**: Use `strip-ansi-escapes` dev-dependency to handle ANSI codes in non-TTY environments
- **Nix Build Tests**: Integration tests are skipped in Nix builds due to missing git repository context
