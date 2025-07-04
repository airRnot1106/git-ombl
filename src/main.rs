use anyhow::Result;
use clap::{Parser, ValueEnum};
use git_ombl::{
    ColoredFormatter, GitAdapter, JsonFormatter, LineHistoryUseCase, OutputFormatter,
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

    /// Reverse sort order (newest first instead of oldest first)
    #[arg(short, long)]
    reverse: bool,
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
    let history = use_case.get_line_history(&cli.file, cli.line, cli.reverse)?;

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
    fn test_cli_parsing() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--format", "json"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.format, Format::Json));
    }

    #[test]
    fn test_cli_parsing_with_reverse_option() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42", "--reverse"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(cli.reverse);
    }

    #[test]
    fn test_cli_parsing_without_reverse_option() {
        let cli = Cli::parse_from(&["git-ombl", "test.rs", "42"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(!cli.reverse);
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
}
