use crate::core::formatting::OutputFormatter;
use crate::core::line_history::LineHistory;

pub struct YamlFormatter;

impl YamlFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for YamlFormatter {
    fn format(&self, history: &LineHistory) -> String {
        serde_yaml::to_string(history).unwrap_or_else(|_| "Error formatting YAML".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChangeType, LineEntry};
    use chrono::{DateTime, Utc};

    #[test]
    fn test_yaml_formatter_empty_history() {
        let formatter = YamlFormatter::new();
        let history = LineHistory::new("test.rs".to_string(), 42);

        let output = formatter.format(&history);

        // Should contain basic YAML structure
        assert!(output.contains("file_path: test.rs"));
        assert!(output.contains("line_number: 42"));
        assert!(output.contains("entries: []"));
    }

    #[test]
    fn test_yaml_formatter_with_entries() {
        let formatter = YamlFormatter::new();
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

        // Should contain YAML structure with entry data
        assert!(output.contains("file_path: test.rs"));
        assert!(output.contains("line_number: 42"));
        assert!(output.contains("commit_hash: abc123"));
        assert!(output.contains("author: Test Author"));
        assert!(output.contains("message: Test commit"));
        assert!(output.contains("change_type: Created"));
    }

    #[test]
    fn test_yaml_formatter_valid_yaml() {
        let formatter = YamlFormatter::new();
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

        // Should be valid YAML that can be parsed back
        let parsed: serde_yaml::Value = serde_yaml::from_str(&output).unwrap();
        assert!(parsed.get("file_path").is_some());
        assert!(parsed.get("line_number").is_some());
        assert!(parsed.get("entries").is_some());
    }
}
