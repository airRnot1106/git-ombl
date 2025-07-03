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
cargo test                               # Run all tests
cargo test <test_name>                   # Run specific test
cargo test -- --nocapture               # Run tests with output
cargo test adapters::git::tests::test_git_adapter_get_line_history -- --nocapture  # Single test with output
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
  - `json.rs` - JSON output
  - `colored.rs` - Terminal colored output

**Dependency Flow:** Details depend on policies, never the reverse.

### Key Implementation Details

**Git Blame Integration:**
- Uses `git2::Blame::get_line()` with **1-based indexing** (not 0-based)
- `blame.get_line(line_number as usize)` where `line_number` is the user-provided line number
- The git2 API returns hunks (groups of consecutive lines from same commit), not individual lines

**Testing Strategy:**
- Follows TDD (Test-Driven Development) with Red-Green-Refactor cycles
- Uses `tempfile` and `git2::Repository::init()` for isolated test repositories
- Tests use `mockall` for mocking in dev-dependencies but rely primarily on test implementations

**CLI Interface:**
- Uses `clap` with derive macros for argument parsing
- Supports `--format` option: `json` or `colored` (default)
- CLI structure: `ombl <file> <line_number> [--format <format>]`

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
    ├── json.rs       # JSON formatter
    └── colored.rs    # Terminal colored formatter
```

## Development Notes

- All data structures implement `serde::Serialize/Deserialize` for JSON compatibility
- Error handling uses `anyhow::Result` throughout
- The project uses Rust edition 2024
- Git operations require the current directory to be a git repository
- Line numbers in user interface are 1-based (matching standard editor conventions)