use crate::domain::LineHistory;
use anyhow::Result;

pub trait LineHistoryProvider {
    fn get_line_history(&self, file_path: &str, line_number: u32) -> Result<LineHistory>;
}

pub trait OutputFormatter {
    fn format(&self, history: &LineHistory) -> String;
}

pub struct LineHistoryUseCase<P: LineHistoryProvider> {
    provider: P,
}

impl<P: LineHistoryProvider> LineHistoryUseCase<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn get_line_history(&self, file_path: &str, line_number: u32) -> Result<LineHistory> {
        self.provider.get_line_history(file_path, line_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChangeType, LineEntry};
    use chrono::{TimeZone, Utc};

    struct EmptyProvider;

    impl LineHistoryProvider for EmptyProvider {
        fn get_line_history(&self, _file_path: &str, _line_number: u32) -> Result<LineHistory> {
            Ok(LineHistory::new("test.rs".to_string(), 42))
        }
    }

    struct PopulatedProvider;

    impl LineHistoryProvider for PopulatedProvider {
        fn get_line_history(&self, _file_path: &str, _line_number: u32) -> Result<LineHistory> {
            let mut history = LineHistory::new("test.rs".to_string(), 42);
            history.add_entry(LineEntry {
                commit_hash: "abc123".to_string(),
                author: "John Doe".to_string(),
                timestamp: Utc.timestamp_opt(1234567890, 0).unwrap(),
                message: "Initial commit".to_string(),
                content: "println!(\"Hello, world!\");".to_string(),
                change_type: ChangeType::Created,
            });
            Ok(history)
        }
    }

    #[test]
    fn test_use_case_creation() {
        let provider = EmptyProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case.get_line_history("test.rs", 42).unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 0);
    }

    #[test]
    fn test_use_case_with_populated_history() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case.get_line_history("test.rs", 42).unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].commit_hash, "abc123");
    }
}
