use anyhow::Result;
use clap::{Parser, ValueEnum};
use git_ombl::{
    ColoredFormatter, GitAdapter, JsonFormatter, LineHistoryUseCase, OutputFormatter, SortOrder,
    TableFormatter, YamlFormatter,
};
use std::env;

#[derive(Parser)]
#[command(name = "git-ombl")]
#[command(about = "Ultrathink git blame - trace complete line history")]
#[command(version)]
struct Cli {
    /// File path to analyze
    file: String,

    /// Line number to analyze
    line: u32,

    /// Output format
    #[arg(short, long, default_value = "colored")]
    format: Format,

    /// Maximum number of commits to traverse
    #[arg(short, long)]
    limit: Option<usize>,

    /// Sort order for commit history
    #[arg(short, long, default_value = "asc")]
    sort: SortOrder,

    /// Ignore changes made by the specified revision(s)
    #[arg(long = "ignore-rev")]
    ignore_revs: Vec<String>,

    /// Show commits more recent than a specific date (e.g., "2023-01-01", "2023-01-01T12:00:00Z")
    #[arg(long)]
    since: Option<String>,

    /// Show commits older than a specific date (e.g., "2023-12-31", "2023-12-31T23:59:59Z")
    #[arg(long)]
    until: Option<String>,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum Format {
    Colored,
    Json,
    Table,
    Yaml,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get current directory as repository root
    let current_dir = env::current_dir()?;

    // Create git adapter
    let git_adapter = GitAdapter::new(&current_dir)?;

    // Create use case
    let use_case = LineHistoryUseCase::new(git_adapter);

    // Get line history
    let history = use_case.get_line_history(
        &cli.file,
        cli.line,
        cli.sort,
        &cli.ignore_revs,
        cli.since.as_deref(),
        cli.until.as_deref(),
    )?;

    // Create formatter based on format choice
    let formatter: Box<dyn OutputFormatter> = match cli.format {
        Format::Colored => Box::new(ColoredFormatter::new()),
        Format::Json => Box::new(JsonFormatter::new()),
        Format::Table => Box::new(TableFormatter::new()),
        Format::Yaml => Box::new(YamlFormatter::new()),
    };

    // Format and output
    let output = formatter.format(&history);
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_ombl::LineHistory;

    #[test]
    fn test_format_enum_parsing() {
        use clap::ValueEnum;

        assert_eq!(Format::from_str("colored", true).unwrap(), Format::Colored);
        assert_eq!(Format::from_str("json", true).unwrap(), Format::Json);
        assert_eq!(Format::from_str("table", true).unwrap(), Format::Table);
        assert_eq!(Format::from_str("yaml", true).unwrap(), Format::Yaml);
    }

    #[test]
    fn test_sort_order_enum_parsing() {
        use clap::ValueEnum;

        assert_eq!(SortOrder::from_str("asc", true).unwrap(), SortOrder::Asc);
        assert_eq!(SortOrder::from_str("desc", true).unwrap(), SortOrder::Desc);
    }

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--format", "json"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.format, Format::Json));
    }

    #[test]
    fn test_cli_parsing_with_sort_desc() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--sort", "desc"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.sort, SortOrder::Desc));
    }

    #[test]
    fn test_cli_parsing_with_sort_asc() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--sort", "asc"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.sort, SortOrder::Asc));
    }

    #[test]
    fn test_cli_parsing_default_sort() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.sort, SortOrder::Asc));
    }

    #[test]
    fn test_cli_parsing_with_single_ignore_rev() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--ignore-rev", "abc123def"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.ignore_revs.len(), 1);
        assert_eq!(cli.ignore_revs[0], "abc123def");
    }

    #[test]
    fn test_cli_parsing_with_multiple_ignore_revs() {
        let cli = Cli::parse_from(&[
            "git-ombl",
            "test.rs",
            "42",
            "--ignore-rev",
            "abc123def",
            "--ignore-rev",
            "def456ghi",
        ]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.ignore_revs.len(), 2);
        assert_eq!(cli.ignore_revs[0], "abc123def");
        assert_eq!(cli.ignore_revs[1], "def456ghi");
    }

    #[test]
    fn test_cli_parsing_with_no_ignore_revs() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(cli.ignore_revs.is_empty());
    }

    #[test]
    fn test_formatter_selection() {
        colored::control::set_override(true);
        let colored_formatter: Box<dyn OutputFormatter> = Box::new(ColoredFormatter::new());
        let json_formatter: Box<dyn OutputFormatter> = Box::new(JsonFormatter::new());
        let table_formatter: Box<dyn OutputFormatter> = Box::new(TableFormatter::new());
        let yaml_formatter: Box<dyn OutputFormatter> = Box::new(YamlFormatter::new());

        let history = LineHistory::new("test.rs".to_string(), 42);

        let colored_output = colored_formatter.format(&history);
        let json_output = json_formatter.format(&history);
        let table_output = table_formatter.format(&history);
        let yaml_output = yaml_formatter.format(&history);

        // Strip ANSI codes for colored output testing
        let stripped = strip_ansi_escapes::strip(&colored_output);
        let stripped_str = String::from_utf8(stripped).unwrap();

        assert!(stripped_str.contains("test.rs:42"));
        assert!(json_output.contains("\"file_path\""));
        assert!(table_output.contains("File: test.rs"));
        assert!(yaml_output.contains("file_path: test.rs"));
    }

    #[test]
    fn test_cli_parsing_with_since_option() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--since", "2023-01-01"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, Some("2023-01-01".to_string()));
        assert_eq!(cli.until, None);
    }

    #[test]
    fn test_cli_parsing_with_until_option() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--until", "2023-12-31"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, None);
        assert_eq!(cli.until, Some("2023-12-31".to_string()));
    }

    #[test]
    fn test_cli_parsing_with_both_since_and_until() {
        let cli = Cli::parse_from(&[
            "git-ombl",
            "test.rs",
            "42",
            "--since",
            "2023-01-01T00:00:00Z",
            "--until",
            "2023-12-31T23:59:59Z",
        ]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, Some("2023-01-01T00:00:00Z".to_string()));
        assert_eq!(cli.until, Some("2023-12-31T23:59:59Z".to_string()));
    }

    #[test]
    fn test_cli_parsing_with_since_rfc2822_format() {
        let cli = Cli::parse_from(&[
            "git-ombl",
            "test.rs",
            "42",
            "--since",
            "Mon, 01 Jan 2023 00:00:00 GMT",
        ]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, Some("Mon, 01 Jan 2023 00:00:00 GMT".to_string()));
    }

    #[test]
    fn test_cli_parsing_with_since_and_ignore_rev_combined() {
        let cli = Cli::parse_from(&[
            "git-ombl",
            "test.rs",
            "42",
            "--since",
            "2023-01-01",
            "--ignore-rev",
            "abc123def",
            "--sort",
            "desc",
        ]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, Some("2023-01-01".to_string()));
        assert_eq!(cli.ignore_revs.len(), 1);
        assert_eq!(cli.ignore_revs[0], "abc123def");
        assert!(matches!(cli.sort, SortOrder::Desc));
    }

    #[test]
    fn test_cli_parsing_without_date_filters() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert_eq!(cli.since, None);
        assert_eq!(cli.until, None);
    }
}
