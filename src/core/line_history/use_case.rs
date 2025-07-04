use crate::core::line_history::{LineHistory, LineHistoryProvider};
use crate::core::types::SortOrder;
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
        sort_order: SortOrder,
        ignore_revs: &[String],
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<LineHistory> {
        self.provider.get_line_history(
            file_path,
            line_number,
            sort_order,
            ignore_revs,
            since,
            until,
        )
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
            _sort_order: SortOrder,
            _ignore_revs: &[String],
            _since: Option<&str>,
            _until: Option<&str>,
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
            _sort_order: SortOrder,
            _ignore_revs: &[String],
            _since: Option<&str>,
            _until: Option<&str>,
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
        let result = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 0);
    }

    #[test]
    fn test_use_case_with_populated_history() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].commit_hash, "abc123");
    }

    #[test]
    fn test_use_case_with_sort_order_parameter() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result_asc = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &[], None, None)
            .unwrap();
        let result_desc = use_case
            .get_line_history("test.rs", 42, SortOrder::Desc, &[], None, None)
            .unwrap();

        assert_eq!(result_asc.file_path, "test.rs");
        assert_eq!(result_desc.file_path, "test.rs");
        assert_eq!(result_asc.line_number, 42);
        assert_eq!(result_desc.line_number, 42);
    }

    #[test]
    fn test_use_case_with_ignore_revs_parameter() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let ignore_revs = vec!["abc123".to_string()];
        let result = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &ignore_revs, None, None)
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        // Note: PopulatedProvider doesn't actually filter, this just tests the parameter passing
    }

    #[test]
    fn test_use_case_with_since_parameter() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &[], Some("2023-01-01"), None)
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        // Note: PopulatedProvider doesn't actually filter, this just tests the parameter passing
    }

    #[test]
    fn test_use_case_with_until_parameter() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case
            .get_line_history("test.rs", 42, SortOrder::Asc, &[], None, Some("2023-12-31"))
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        // Note: PopulatedProvider doesn't actually filter, this just tests the parameter passing
    }

    #[test]
    fn test_use_case_with_both_since_and_until_parameters() {
        let provider = PopulatedProvider;
        let use_case = LineHistoryUseCase::new(provider);
        let result = use_case
            .get_line_history(
                "test.rs",
                42,
                SortOrder::Asc,
                &[],
                Some("2023-01-01"),
                Some("2023-12-31"),
            )
            .unwrap();

        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_number, 42);
        // Note: PopulatedProvider doesn't actually filter, this just tests the parameter passing
    }
}
