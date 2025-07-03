use crate::core::line_history::LineHistory;

pub trait OutputFormatter {
    fn format(&self, history: &LineHistory) -> String;
}
