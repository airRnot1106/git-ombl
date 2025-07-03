use crate::domain::{ChangeType, LineEntry, LineHistory};
use crate::policy::LineHistoryProvider;
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::{Blame, BlameOptions, Repository};
use std::path::Path;

pub struct GitAdapter {
    repository: Repository,
}

impl GitAdapter {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repository = Repository::open(repo_path)?;
        Ok(Self { repository })
    }

    fn get_blame_for_file(&self, file_path: &str, _line_number: u32) -> Result<Blame> {
        let mut blame_options = BlameOptions::new();
        blame_options.track_copies_same_file(true);

        let blame = self
            .repository
            .blame_file(Path::new(file_path), Some(&mut blame_options))?;
        Ok(blame)
    }

    fn extract_line_entries(&self, blame: &Blame, line_number: u32) -> Result<Vec<LineEntry>> {
        let mut entries = Vec::new();

        // Check if line number is valid
        if line_number == 0 {
            return Ok(entries);
        }

        // Use git2's get_line method to get the hunk for the specific line
        // Note: get_line uses 1-based indexing (same as user input)
        if let Some(hunk) = blame.get_line(line_number as usize) {
            let commit_id = hunk.final_commit_id();
            let commit = self.repository.find_commit(commit_id)?;

            let author = commit.author();
            let timestamp =
                DateTime::from_timestamp(commit.time().seconds(), 0).unwrap_or_else(|| Utc::now());

            let entry = LineEntry {
                commit_hash: commit_id.to_string(),
                author: author.name().unwrap_or("Unknown").to_string(),
                timestamp,
                message: commit.message().unwrap_or("").to_string(),
                content: "".to_string(), // TODO: Extract actual line content
                change_type: ChangeType::Created, // TODO: Determine if created/modified
            };

            entries.push(entry);
        }

        Ok(entries)
    }
}

impl LineHistoryProvider for GitAdapter {
    fn get_line_history(&self, file_path: &str, line_number: u32) -> Result<LineHistory> {
        let blame = self.get_blame_for_file(file_path, line_number)?;
        let entries = self.extract_line_entries(&blame, line_number)?;

        let mut history = LineHistory::new(file_path.to_string(), line_number);
        for entry in entries {
            history.add_entry(entry);
        }

        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize git repository
        let repo = Repository::init(repo_path)?;

        // Configure git user
        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;

        // Create initial file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "line 1\nline 2\nline 3\n")?;

        // Add and commit
        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = git2::Signature::now("Test User", "test@example.com")?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;

        Ok(temp_dir)
    }

    #[test]
    fn test_git_adapter_creation() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Test that adapter was created successfully
        assert!(adapter.repository.path().exists());
    }

    #[test]
    fn test_git_adapter_get_line_history() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        let history = adapter.get_line_history("test.txt", 1).unwrap();

        assert_eq!(history.file_path, "test.txt");
        assert_eq!(history.line_number, 1);
        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.entries[0].author, "Test User");
        assert_eq!(history.entries[0].message, "Initial commit");
    }

    #[test]
    fn test_git_adapter_nonexistent_file() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        let result = adapter.get_line_history("nonexistent.txt", 1);
        assert!(result.is_err());
    }
}
