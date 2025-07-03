use anyhow::Result;
use clap::{Parser, ValueEnum};
use ombl::{ColoredFormatter, GitAdapter, JsonFormatter, LineHistoryUseCase, OutputFormatter};
use std::env;

#[derive(Parser)]
#[command(name = "ombl")]
#[command(about = "Ultrathink git blame - trace complete line history")]
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
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum Format {
    Json,
    Colored,
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
    let history = use_case.get_line_history(&cli.file, cli.line)?;

    // Create formatter based on format choice
    let formatter: Box<dyn OutputFormatter> = match cli.format {
        Format::Json => Box::new(JsonFormatter::new()),
        Format::Colored => Box::new(ColoredFormatter::new()),
    };

    // Format and output
    let output = formatter.format(&history);
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ombl::LineHistory;

    #[test]
    fn test_format_enum_parsing() {
        use clap::ValueEnum;

        assert_eq!(Format::from_str("json", true).unwrap(), Format::Json);
        assert_eq!(Format::from_str("colored", true).unwrap(), Format::Colored);
    }

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(&["ombl", "test.rs", "42", "--format", "json"]);

        assert_eq!(cli.file, "test.rs");
        assert_eq!(cli.line, 42);
        assert!(matches!(cli.format, Format::Json));
    }

    #[test]
    fn test_formatter_selection() {
        let json_formatter: Box<dyn OutputFormatter> = Box::new(JsonFormatter::new());
        let colored_formatter: Box<dyn OutputFormatter> = Box::new(ColoredFormatter::new());

        let history = LineHistory::new("test.rs".to_string(), 42);

        let json_output = json_formatter.format(&history);
        let colored_output = colored_formatter.format(&history);

        assert!(json_output.contains("\"file_path\""));
        assert!(colored_output.contains("test.rs:42"));
    }
}
