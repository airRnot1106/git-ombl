use crate::core::line_history::domain::LineHistory;
use crate::core::types::SortOrder;
use anyhow::Result;

pub trait LineHistoryProvider {
    fn get_line_history(
        &self,
        file_path: &str,
        line_number: u32,
        sort_order: SortOrder,
    ) -> Result<LineHistory>;
}
