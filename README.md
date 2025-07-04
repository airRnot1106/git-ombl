# git-ombl

**git-omni-blame** - Git blame wrapper to display the full commit history for a line.

Unlike `git blame` which shows only the most recent change, git-ombl traces the entire history of a line through all commits that affected it.

## Installation

```bash
# Using Cargo
cargo install --git https://github.com/airRnot1106/git-ombl

# Using Nix
nix build github:airRnot1106/git-ombl
```

## Usage

```bash
git-ombl <file> <line_number> [OPTIONS]
```

### Options

- `-f, --format <FORMAT>`: Output format [default: colored]
  - `colored`: Terminal output with colors
  - `json`: JSON format
  - `yaml`: YAML format
  - `table`: Tabular format
- `-l, --limit <LIMIT>`: Maximum number of commits to traverse
- `-h, --help`: Print help
- `-V, --version`: Print version

### Examples

```bash
# Show complete history of line 42 in main.rs
git-ombl src/main.rs 42

# Output as JSON
git-ombl src/main.rs 42 --format json

# Limit to last 10 commits
git-ombl src/main.rs 42 --limit 10
```

### Sample Output

```bash
$ git-ombl test_sample.rs 1
test_sample.rs:1

abc1234 John Doe    2024-01-15 14:30:22 UTC  Initial commit
  // This is a test file for ombl

def5678 Jane Smith  2024-01-16 09:15:30 UTC  Update comment
  // This is a test file for ombl

ghi9012 Bob Wilson  2024-01-17 16:45:10 UTC  Final version
  // This is a test file for ombl - FINAL VERSION
```

## Development

### Build

```bash
# Standard build
cargo build

# Run tests
cargo test

# With Nix (recommended)
nix develop  # Enter development shell
nix build    # Build with Nix
```

### Development Environment

```bash
# Clone repository
git clone https://github.com/airRnot1106/git-ombl
cd git-ombl

# Enter Nix development shell (includes Rust, formatting tools)
nix develop

# Or use standard Cargo workflow
cargo build
cargo test
```

## License

MIT
