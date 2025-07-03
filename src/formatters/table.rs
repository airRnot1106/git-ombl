use crate::core::formatting::OutputFormatter;
use crate::core::line_history::LineHistory;
use tabled::{Table, Tabled};

pub struct TableFormatter;

#[derive(Tabled)]
struct TableEntry {
    #[tabled(rename = "Commit")]
    commit_hash: String,
    #[tabled(rename = "Author")]
    author: String,
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Message")]
    message: String,
    #[tabled(rename = "Change Type")]
    change_type: String,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for TableFormatter {
    fn format(&self, history: &LineHistory) -> String {
        let header = format!(
            "File: {}\nLine: {}\n\n",
            history.file_path, history.line_number
        );

        if history.entries.is_empty() {
            return format!("{}No history entries", header);
        }

        let table_entries: Vec<TableEntry> = history
            .entries
            .iter()
            .map(|entry| {
                TableEntry {
                    commit_hash: entry.commit_hash.chars().take(8).collect(), // Truncate commit hash
                    author: entry.author.clone(),
                    timestamp: entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    message: entry.message.clone(),
                    change_type: entry.change_type.to_string(),
                }
            })
            .collect();

        let table = Table::new(table_entries).to_string();
        format!("{}{}", header, table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChangeType, LineEntry};
    use chrono::{DateTime, Utc};

    #[test]
    fn test_table_formatter_empty_history() {
        let formatter = TableFormatter::new();
        let history = LineHistory::new("test.rs".to_string(), 42);

        let output = formatter.format(&history);

        // Should contain basic table structure
        assert!(output.contains("File: test.rs"));
        assert!(output.contains("Line: 42"));
        assert!(output.contains("No history entries"));
    }

    #[test]
    fn test_table_formatter_with_entries() {
        let formatter = TableFormatter::new();
        let mut history = LineHistory::new("test.rs".to_string(), 42);

        let entry = LineEntry {
            commit_hash: "abc123".to_string(),
            author: "Test Author".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "Test commit".to_string(),
            content: "test content".to_string(),
            change_type: ChangeType::Created,
        };

        history.add_entry(entry);

        let output = formatter.format(&history);

        // Should contain table headers and data
        assert!(output.contains("File: test.rs"));
        assert!(output.contains("Line: 42"));
        assert!(output.contains("Commit"));
        assert!(output.contains("Author"));
        assert!(output.contains("Message"));
        assert!(output.contains("Change Type"));
        assert!(output.contains("abc123"));
        assert!(output.contains("Test Author"));
        assert!(output.contains("Test commit"));
        assert!(output.contains("Created"));
    }

    #[test]
    fn test_table_formatter_multiple_entries() {
        let formatter = TableFormatter::new();
        let mut history = LineHistory::new("test.rs".to_string(), 42);

        let entry1 = LineEntry {
            commit_hash: "abc123".to_string(),
            author: "Test Author 1".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "First commit".to_string(),
            content: "test content 1".to_string(),
            change_type: ChangeType::Created,
        };

        let entry2 = LineEntry {
            commit_hash: "def456".to_string(),
            author: "Test Author 2".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2023-01-02T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            message: "Second commit".to_string(),
            content: "test content 2".to_string(),
            change_type: ChangeType::Modified,
        };

        history.add_entry(entry1);
        history.add_entry(entry2);

        let output = formatter.format(&history);

        // Should contain both entries
        assert!(output.contains("abc123"));
        assert!(output.contains("def456"));
        assert!(output.contains("Test Author 1"));
        assert!(output.contains("Test Author 2"));
        assert!(output.contains("First commit"));
        assert!(output.contains("Second commit"));
        assert!(output.contains("Created"));
        assert!(output.contains("Modified"));
    }
}
