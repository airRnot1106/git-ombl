use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineHistory {
    pub file_path: String,
    pub line_number: u32,
    pub entries: Vec<LineEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineEntry {
    pub commit_hash: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub content: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Created => write!(f, "Created"),
            ChangeType::Modified => write!(f, "Modified"),
            ChangeType::Deleted => write!(f, "Deleted"),
        }
    }
}

impl LineHistory {
    pub fn new(file_path: String, line_number: u32) -> Self {
        Self {
            file_path,
            line_number,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: LineEntry) {
        self.entries.push(entry);
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_line_history_creation() {
        let history = LineHistory::new("test.rs".to_string(), 42);

        assert_eq!(history.file_path, "test.rs");
        assert_eq!(history.line_number, 42);
        assert_eq!(history.entry_count(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut history = LineHistory::new("test.rs".to_string(), 42);
        let entry = LineEntry {
            commit_hash: "abc123".to_string(),
            author: "John Doe".to_string(),
            timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
            message: "Initial commit".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            change_type: ChangeType::Created,
        };

        history.add_entry(entry.clone());

        assert_eq!(history.entry_count(), 1);
        assert_eq!(history.entries[0], entry);
    }

    #[test]
    fn test_line_entry_serialization() {
        let entry = LineEntry {
            commit_hash: "abc123".to_string(),
            author: "John Doe".to_string(),
            timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
            message: "Initial commit".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            change_type: ChangeType::Created,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LineEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry, deserialized);
    }
}
