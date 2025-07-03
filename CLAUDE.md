# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ombl` (Ultrathink Git Blame) is a Rust CLI tool that extends git blame functionality to trace the complete history of individual lines across all commits, not just the most recent change. The tool is built using Clean Architecture principles with clear separation between domain logic, use cases, and adapters.

## Development Commands

### Build and Run
```bash
cargo build                              # Build the project
cargo run -- <file> <line_number>       # Run with arguments
./target/debug/ombl <file> <line_number> # Run built binary directly
```

### Testing
```bash
cargo test                               # Run all tests (unit + integration)
cargo test --test integration_tests     # Run only integration tests
cargo test <test_name>                   # Run specific test by name
cargo test -- --nocapture               # Run tests with stdout output
cargo test adapters::git::tests::test_git_adapter_get_line_history -- --nocapture  # Single test with output

# Integration test examples
cargo test test_sample_file_line_history_integration
cargo test test_sample_file_with_all_formatters
```

### Formatting and Linting
- The project uses Nix flakes with treefmt for formatting
- Pre-commit hooks automatically format Rust code with rustfmt
- Formatting is enforced via git hooks (treefmt configuration in `nix/treefmt/default.nix`)

## Architecture

### Clean Architecture Implementation
The codebase follows Clean Architecture with dependency inversion:

**Policy Layer (Stable/Abstract):**
- `src/policy.rs` - Contains traits `LineHistoryProvider` and `OutputFormatter`
- `src/domain.rs` - Core domain types (`LineHistory`, `LineEntry`, `ChangeType`)
- Use case: `LineHistoryUseCase<P: LineHistoryProvider>`

**Detail Layer (Concrete/Variable):**
- `src/adapters/git.rs` - `GitAdapter` implements `LineHistoryProvider` using git2
- `src/formatters/` - Multiple formatters implement `OutputFormatter`
  - `colored.rs` - Terminal colored output with ANSI colors
  - `json.rs` - JSON output using serde_json
  - `table.rs` - Tabular output using tabled crate
  - `yaml.rs` - YAML output using serde_yaml

**Dependency Flow:** Details depend on policies, never the reverse.

### Key Implementation Details

**Git History Traversal:**
- **Complete History**: Traces ALL commits that affected a line, not just the most recent
- Uses `repository.revwalk()` with `git2::Sort::TIME` to traverse commit history chronologically
- Implements `commit_affects_file()` and `commit_changes_line()` to filter relevant commits
- **1-based indexing**: Line numbers match standard editor conventions (not 0-based)
- Sorts entries chronologically (oldest first) using commit timestamps

**Testing Strategy:**
- Follows TDD (Test-Driven Development) with Red-Green-Refactor cycles
- **Unit Tests**: Uses `tempfile` and `git2::Repository::init()` for isolated test repositories
- **Integration Tests**: Real-world testing with `test_sample.rs` (has 3-commit history for line 1)
- Tests use `mockall` for mocking in dev-dependencies but rely primarily on test implementations
- Integration tests verify all formatters with actual git repository data

**CLI Interface:**
- Uses `clap` with derive macros for argument parsing
- Supports `--format` option: `colored` (default), `json`, `table`, or `yaml`
- CLI structure: `ombl <file> <line_number> [--format <format>] [--limit <number>]`
- Additional `--limit` option for constraining number of commits traversed

## Module Structure

```
src/
├── main.rs           # CLI entry point and argument parsing
├── lib.rs            # Module exports and re-exports
├── domain.rs         # Core domain types (LineHistory, LineEntry, ChangeType)
├── policy.rs         # Traits and use case (LineHistoryProvider, OutputFormatter)
├── adapters/         # External system integrations
│   └── git.rs        # Git integration via git2
└── formatters/       # Output format implementations
    ├── mod.rs        # Module exports
    ├── colored.rs    # Terminal colored formatter
    ├── json.rs       # JSON formatter
    ├── table.rs      # Table formatter
    └── yaml.rs       # YAML formatter

tests/
└── integration_tests.rs # Integration tests with real git data

test_sample.rs        # Test file with 3-commit history for integration testing
```

## Development Notes

### TDD Methodology
- **Strictly follows TDD**: Red-Green-Refactor cycles as prescribed by t_wada
- **Martin Fowler Refactoring**: Apply Extract Method, Remove Dead Code, and other techniques
- **Test Structure**: Each formatter and major feature has comprehensive test coverage

### Technical Details
- All domain types implement `serde::Serialize/Deserialize` for multi-format output
- Error handling uses `anyhow::Result` throughout for ergonomic error propagation
- Uses Rust edition 2024 with modern language features
- Git operations require the current directory to be a git repository
- **Lifetime Management**: `git2::Commit<'_>` requires explicit lifetime annotations
- **Dependencies**: serde_yaml, tabled, colored, chrono for various output formats

### Key Gotchas
- Line numbers are **1-based** in all user interfaces (not 0-based)
- `git2::Commit` objects borrow from the Repository, requiring careful lifetime management
- Pre-commit hooks auto-format with treefmt, may modify files during commit
- Test repositories use explicit timestamps for deterministic chronological ordering

### Integration Testing
- `test_sample.rs` is a real file in the repository with git history for testing
- Line 1 has been modified across 3 commits to test complete history traversal
- Integration tests verify all formatters (Colored, JSON, Table, YAML) work with real data
- Tests check chronological ordering, change types, and commit message validation
- Use `cargo run -- test_sample.rs 1 --format <format>` to manually test output formats