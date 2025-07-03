use crate::domain::{ChangeType, LineEntry, LineHistory};
use crate::policy::LineHistoryProvider;
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::Repository;
use std::path::Path;

pub struct GitAdapter {
    repository: Repository,
}

impl GitAdapter {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repository = Repository::open(repo_path)?;
        Ok(Self { repository })
    }

    fn extract_full_line_history(
        &self,
        file_path: &str,
        line_number: u32,
    ) -> Result<Vec<LineEntry>> {
        let commits = self.find_commits_affecting_file(file_path, line_number)?;

        // Check if the file exists in the repository at all
        if commits.is_empty() {
            // Try to find the file in the current HEAD to see if it exists
            let head = self.repository.head()?;
            let head_commit = head.peel_to_commit()?;
            let tree = head_commit.tree()?;
            if tree.get_path(Path::new(file_path)).is_err() {
                return Err(anyhow::anyhow!(
                    "File not found in repository: {}",
                    file_path
                ));
            }
        }

        let entries = self.convert_commits_to_entries(commits)?;
        self.sort_entries_chronologically(entries)
    }

    fn find_commits_affecting_file(
        &self,
        file_path: &str,
        line_number: u32,
    ) -> Result<Vec<git2::Commit>> {
        let mut commits = Vec::new();
        let mut revwalk = self.repository.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut seen_commits = std::collections::HashSet::new();

        for commit_oid in revwalk {
            let commit_oid = commit_oid?;

            if seen_commits.contains(&commit_oid) {
                continue;
            }
            seen_commits.insert(commit_oid);

            let commit = self.repository.find_commit(commit_oid)?;

            if self.commit_affects_file(&commit, file_path)?
                && self.commit_changes_line(file_path, line_number, &commit)?
            {
                commits.push(commit);
            }
        }

        Ok(commits)
    }

    fn commit_affects_file(&self, commit: &git2::Commit, file_path: &str) -> Result<bool> {
        if let Some(tree) = commit.tree_id().into() {
            let tree = self.repository.find_tree(tree)?;
            return Ok(tree.get_path(Path::new(file_path)).is_ok());
        }
        Ok(false)
    }

    fn convert_commits_to_entries(&self, commits: Vec<git2::Commit>) -> Result<Vec<LineEntry>> {
        let mut entries = Vec::new();

        for commit in commits {
            let entry = self.create_line_entry_from_commit(&commit, entries.is_empty())?;
            entries.push(entry);
        }

        Ok(entries)
    }

    fn create_line_entry_from_commit(
        &self,
        commit: &git2::Commit,
        is_first_entry: bool,
    ) -> Result<LineEntry> {
        let author = commit.author();
        let timestamp =
            DateTime::from_timestamp(commit.time().seconds(), 0).unwrap_or_else(|| Utc::now());

        Ok(LineEntry {
            commit_hash: commit.id().to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            timestamp,
            message: commit.message().unwrap_or("").to_string(),
            content: "".to_string(),
            change_type: if is_first_entry {
                ChangeType::Created
            } else {
                ChangeType::Modified
            },
        })
    }

    fn sort_entries_chronologically(&self, mut entries: Vec<LineEntry>) -> Result<Vec<LineEntry>> {
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(entries)
    }

    fn commit_changes_line(
        &self,
        file_path: &str,
        _line_number: u32,
        commit: &git2::Commit,
    ) -> Result<bool> {
        // For the first commit (no parents), assume it creates the line
        if commit.parent_count() == 0 {
            return Ok(true);
        }

        // For subsequent commits, check if the line content changed
        // This is a simplified check - we assume if the file was modified in this commit,
        // and the line exists, then it was potentially changed
        let mut found_file_change = false;

        for parent_commit in commit.parents() {
            let diff = self.repository.diff_tree_to_tree(
                Some(&parent_commit.tree()?),
                Some(&commit.tree()?),
                None,
            )?;

            diff.foreach(
                &mut |delta, _progress| {
                    if let Some(file) = delta.new_file().path() {
                        if file == Path::new(file_path) {
                            found_file_change = true;
                        }
                    }
                    true
                },
                None,
                None,
                None,
            )?;

            if found_file_change {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl LineHistoryProvider for GitAdapter {
    fn get_line_history(&self, file_path: &str, line_number: u32) -> Result<LineHistory> {
        // Use full history extraction for multiple commits
        let entries = self.extract_full_line_history(file_path, line_number)?;

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

    fn setup_test_repo_with_multiple_commits() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize git repository
        let repo = Repository::init(repo_path)?;

        // Configure git user
        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;

        // First commit - create file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "original line 1\nline 2\nline 3\n")?;

        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature1 =
            git2::Signature::new("Test User", "test@example.com", &git2::Time::new(1000, 0))?;

        let initial_commit = repo.commit(
            Some("HEAD"),
            &signature1,
            &signature1,
            "Initial commit",
            &tree,
            &[],
        )?;

        // Second commit - modify line 1
        fs::write(
            &file_path,
            "modified line 1 - first change\nline 2\nline 3\n",
        )?;

        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent_commit = repo.find_commit(initial_commit)?;
        let signature2 =
            git2::Signature::new("Test User", "test@example.com", &git2::Time::new(2000, 0))?;

        let second_commit = repo.commit(
            Some("HEAD"),
            &signature2,
            &signature2,
            "Update line 1 - first change",
            &tree,
            &[&parent_commit],
        )?;

        // Third commit - modify line 1 again
        fs::write(
            &file_path,
            "modified line 1 - second change\nline 2\nline 3\n",
        )?;

        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent_commit = repo.find_commit(second_commit)?;
        let signature3 =
            git2::Signature::new("Test User", "test@example.com", &git2::Time::new(3000, 0))?;

        repo.commit(
            Some("HEAD"),
            &signature3,
            &signature3,
            "Update line 1 - second change",
            &tree,
            &[&parent_commit],
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

    #[test]
    fn test_git_adapter_multiple_commit_history() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        let history = adapter.get_line_history("test.txt", 1).unwrap();

        assert_eq!(history.file_path, "test.txt");
        assert_eq!(history.line_number, 1);

        // Debug output
        println!("Found {} entries:", history.entries.len());
        for (i, entry) in history.entries.iter().enumerate() {
            println!(
                "  {}: {} - {}",
                i,
                entry.commit_hash[..8].to_string(),
                entry.message
            );
        }

        // This should fail initially - we expect 3 commits but only get 1
        assert_eq!(history.entries.len(), 3);

        // Verify the entries are in chronological order (oldest first)
        assert_eq!(history.entries[0].message, "Initial commit");
        assert_eq!(history.entries[1].message, "Update line 1 - first change");
        assert_eq!(history.entries[2].message, "Update line 1 - second change");
    }
}
