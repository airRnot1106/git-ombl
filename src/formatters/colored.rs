use crate::core::formatting::OutputFormatter;
use crate::core::line_history::LineHistory;
use colored::Colorize;

pub struct ColoredFormatter;

impl ColoredFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for ColoredFormatter {
    fn format(&self, history: &LineHistory) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "{}:{}\n",
            history.file_path.cyan(),
            history.line_number.to_string().yellow()
        ));

        if history.entries.is_empty() {
            output.push_str(&"No history found".dimmed().to_string());
            return output;
        }

        for (i, entry) in history.entries.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            let short_hash = if entry.commit_hash.len() >= 8 {
                &entry.commit_hash[..8]
            } else {
                &entry.commit_hash
            };

            output.push_str(&format!(
                "{} {} {} {}\n{}",
                short_hash.bright_green(),
                entry.author.blue(),
                entry
                    .timestamp
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .white(),
                format!("({})", entry.change_type).purple(),
                entry.message.white()
            ));

            if !entry.content.is_empty() {
                output.push_str(&format!("\n  {}", entry.content.bright_white()));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::line_history::{ChangeType, LineEntry};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_colored_formatter_empty_history() {
        let formatter = ColoredFormatter::new();
        let history = LineHistory::new("test.rs".to_string(), 42);

        let result = formatter.format(&history);

        assert!(result.contains("test.rs:42"));
        assert!(result.contains("No history found"));
    }

    #[test]
    fn test_colored_formatter_with_entries() {
        let formatter = ColoredFormatter::new();
        let mut history = LineHistory::new("test.rs".to_string(), 42);

        history.add_entry(LineEntry {
            commit_hash: "abc123456789".to_string(),
            author: "John Doe".to_string(),
            timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
            message: "Initial commit".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            change_type: ChangeType::Created,
        });

        let result = formatter.format(&history);

        // Test that essential information is present (colors are ANSI escaped)
        assert!(result.contains("test.rs:42"));
        assert!(result.contains("abc12345")); // First 8 chars of commit hash
        assert!(result.contains("John Doe"));
        assert!(result.contains("Initial commit"));
        assert!(result.contains("Created"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_colored_formatter_multiple_entries() {
        let formatter = ColoredFormatter::new();
        let mut history = LineHistory::new("test.rs".to_string(), 42);

        history.add_entry(LineEntry {
            commit_hash: "abc123".to_string(),
            author: "John Doe".to_string(),
            timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
            message: "Initial commit".to_string(),
            content: "old content".to_string(),
            change_type: ChangeType::Created,
        });

        history.add_entry(LineEntry {
            commit_hash: "def456".to_string(),
            author: "Jane Smith".to_string(),
            timestamp: Utc.timestamp_opt(1234567900, 0).unwrap(),
            message: "Update line".to_string(),
            content: "new content".to_string(),
            change_type: ChangeType::Modified,
        });

        let result = formatter.format(&history);

        assert!(result.contains("John Doe"));
        assert!(result.contains("Jane Smith"));
        assert!(result.contains("Initial commit"));
        assert!(result.contains("Update line"));
        assert!(result.contains("old content"));
        assert!(result.contains("new content"));
    }
}
