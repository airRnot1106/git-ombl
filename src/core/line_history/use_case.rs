use crate::core::line_history::{LineHistory, LineHistoryProvider};
use anyhow::Result;

pub struct LineHistoryUseCase<P: LineHistoryProvider> {
    provider: P,
}

impl<P: LineHistoryProvider> LineHistoryUseCase<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn get_line_history(
        &self,
        file_path: &str,
        line_number: u32,
        reverse: bool,
    ) -> Result<LineHistory> {
        self.provider
            .get_line_history(file_path, line_number, reverse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::line_history::{ChangeType, LineEntry};
    use chrono::{TimeZone, Utc};

    struct EmptyProvider;

    impl LineHistoryProvider for EmptyProvider {
        fn get_line_history(
            &self,
            _file_path: &str,
            _line_number: u32,
            _reverse: bool,
        ) -> Result<LineHistory> {
            Ok(LineHistory::new("test.rs".to_string(), 42))
        }
    }

    struct PopulatedProvider;

    impl LineHistoryProvider for PopulatedProvider {
        fn get_line_history(
            &self,
            _file_path: &str,
            _line_number: u32,
            _reverse: bool,
        ) -> Result<LineHistory> {
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
        let result = use_case.get_line_history("test.rs", 42, false).unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 0);
    }

    #[test]
    fn test_use_case_with_populated_history() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case.get_line_history("test.rs", 42, false).unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].commit_hash, "abc123");
    }

    #[test]
    fn test_use_case_with_reverse_parameter() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result_normal = use_case.get_line_history("test.rs", 42, false).unwrap();
        let result_reverse = use_case.get_line_history("test.rs", 42, true).unwrap();

        assert_eq!(result_normal.file_path, "test.rs");
        assert_eq!(result_reverse.file_path, "test.rs");
        assert_eq!(result_normal.line_number, 42);
        assert_eq!(result_reverse.line_number, 42);
    }
}
