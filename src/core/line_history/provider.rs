use crate::core::line_history::domain::LineHistory;
use anyhow::Result;

pub trait LineHistoryProvider {
    fn get_line_history(
        &self,
        file_path: &str,
        line_number: u32,
        reverse: bool,
    ) -> Result<LineHistory>;
}
