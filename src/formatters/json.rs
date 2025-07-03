use crate::domain::LineHistory;
use crate::policy::OutputFormatter;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, history: &LineHistory) -> String {
        serde_json::to_string_pretty(history).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChangeType, LineEntry};
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_json_formatter_empty_history() {
        let formatter = JsonFormatter::new();
        let history = LineHistory::new("test.rs".to_string(), 42);

        let result = formatter.format(&history);

        assert!(result.contains("\"file_path\": \"test.rs\""));
        assert!(result.contains("\"line_number\": 42"));
        assert!(result.contains("\"entries\": []"));
    }

    #[test]
    fn test_json_formatter_with_entries() {
        let formatter = JsonFormatter::new();
        let mut history = LineHistory::new("test.rs".to_string(), 42);

        history.add_entry(LineEntry {
            commit_hash: "abc123".to_string(),
            author: "John Doe".to_string(),
            timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
            message: "Initial commit".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            change_type: ChangeType::Created,
        });

        let result = formatter.format(&history);

        assert!(result.contains("\"commit_hash\": \"abc123\""));
        assert!(result.contains("\"author\": \"John Doe\""));
        assert!(result.contains("\"message\": \"Initial commit\""));
        assert!(result.contains("\"Created\""));
    }

    #[test]
    fn test_json_formatter_valid_json() {
        let formatter = JsonFormatter::new();
        let history = LineHistory::new("test.rs".to_string(), 42);

        let result = formatter.format(&history);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["file_path"], "test.rs");
        assert_eq!(parsed["line_number"], 42);
    }
}
