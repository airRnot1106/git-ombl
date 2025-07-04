use crate::core::line_history::{ChangeType, LineEntry, LineHistory, LineHistoryProvider};
use crate::core::types::SortOrder;
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
        sort_order: SortOrder,
        ignore_revs: &[String],
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<LineEntry>> {
        let commits =
            self.find_commits_affecting_file(file_path, line_number, ignore_revs, since, until)?;

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
        self.sort_entries_chronologically(entries, sort_order)
    }

    fn find_commits_affecting_file(
        &self,
        file_path: &str,
        line_number: u32,
        ignore_revs: &[String],
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<git2::Commit<'_>>> {
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

            // Check if this commit should be ignored
            if self.should_ignore_commit(&commit, ignore_revs) {
                continue;
            }

            // Check if this commit should be filtered by date
            if !self.should_filter_by_date(&commit, since, until)? {
                continue;
            }

            if self.commit_affects_file(&commit, file_path)?
                && self.commit_changes_line(file_path, line_number, &commit)?
            {
                commits.push(commit);
            }
        }

        Ok(commits)
    }

    fn should_ignore_commit(&self, commit: &git2::Commit, ignore_revs: &[String]) -> bool {
        let commit_hash = commit.id().to_string();

        for ignore_rev in ignore_revs {
            // Support both full hashes and abbreviated hashes
            if commit_hash == *ignore_rev || commit_hash.starts_with(ignore_rev) {
                return true;
            }
        }

        false
    }

    fn parse_git_date(&self, date_str: &str) -> Result<DateTime<Utc>> {
        use chrono::TimeZone;

        // Try ISO 8601 format first (most precise)
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try RFC 2822 format
        if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try custom RFC-like format that git sometimes uses
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S %Z") {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try simple date format (YYYY-MM-DD)
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }

        // Try datetime format (YYYY-MM-DD HH:MM:SS)
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }

        // If all else fails, return an error
        Err(anyhow::anyhow!(
            "Unable to parse date '{}'. Supported formats: ISO 8601 (YYYY-MM-DDTHH:MM:SSZ), RFC 2822, YYYY-MM-DD, YYYY-MM-DD HH:MM:SS",
            date_str
        ))
    }

    fn should_filter_by_date(
        &self,
        commit: &git2::Commit,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<bool> {
        let commit_time = commit.time();
        let commit_timestamp =
            DateTime::from_timestamp(commit_time.seconds(), 0).unwrap_or_else(|| Utc::now());

        // Check since filter
        if let Some(since_str) = since {
            let since_date = self.parse_git_date(since_str)?;
            if commit_timestamp < since_date {
                return Ok(false);
            }
        }

        // Check until filter
        if let Some(until_str) = until {
            let until_date = self.parse_git_date(until_str)?;
            if commit_timestamp > until_date {
                return Ok(false);
            }
        }

        Ok(true)
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

    fn sort_entries_chronologically(
        &self,
        mut entries: Vec<LineEntry>,
        sort_order: SortOrder,
    ) -> Result<Vec<LineEntry>> {
        match sort_order {
            SortOrder::Desc => {
                entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Newest first
            }
            SortOrder::Asc => {
                entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)); // Oldest first
            }
        }
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
    fn get_line_history(
        &self,
        file_path: &str,
        line_number: u32,
        sort_order: SortOrder,
        ignore_revs: &[String],
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<LineHistory> {
        // Use full history extraction for multiple commits
        let entries = self.extract_full_line_history(
            file_path,
            line_number,
            sort_order,
            ignore_revs,
            since,
            until,
        )?;

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

        let history = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

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

        let result =
            adapter.get_line_history("nonexistent.txt", 1, SortOrder::Asc, &[], None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_adapter_multiple_commit_history() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        let history = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

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

    #[test]
    fn test_git_adapter_sort_order() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Test ascending order (oldest first)
        let history_asc = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        // Test descending order (newest first)
        let history_desc = adapter
            .get_line_history("test.txt", 1, SortOrder::Desc, &[], None, None)
            .unwrap();

        // Both should have the same number of entries
        assert_eq!(history_asc.entries.len(), history_desc.entries.len());
        assert_eq!(history_asc.entries.len(), 3);

        // Ascending order: oldest first
        assert_eq!(history_asc.entries[0].message, "Initial commit");
        assert_eq!(
            history_asc.entries[1].message,
            "Update line 1 - first change"
        );
        assert_eq!(
            history_asc.entries[2].message,
            "Update line 1 - second change"
        );

        // Descending order: newest first
        assert_eq!(
            history_desc.entries[0].message,
            "Update line 1 - second change"
        );
        assert_eq!(
            history_desc.entries[1].message,
            "Update line 1 - first change"
        );
        assert_eq!(history_desc.entries[2].message, "Initial commit");

        // Verify descending order has later timestamps first
        assert!(history_desc.entries[0].timestamp >= history_desc.entries[1].timestamp);
        assert!(history_desc.entries[1].timestamp >= history_desc.entries[2].timestamp);
    }

    #[test]
    fn test_git_adapter_ignore_single_revision() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // First get all commits to find one to ignore
        let history_all = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(history_all.entries.len(), 3);

        // Get the hash of the second commit to ignore
        let ignore_hash = &history_all.entries[1].commit_hash[..8]; // Use abbreviated hash
        let ignore_revs = vec![ignore_hash.to_string()];

        // Test with ignored revision
        let history_filtered = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &ignore_revs, None, None)
            .unwrap();

        // Should have one less commit
        assert_eq!(history_filtered.entries.len(), 2);

        // Verify the ignored commit is not present
        for entry in &history_filtered.entries {
            assert!(!entry.commit_hash.starts_with(ignore_hash));
        }
    }

    #[test]
    fn test_git_adapter_ignore_multiple_revisions() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // First get all commits
        let history_all = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(history_all.entries.len(), 3);

        // Ignore first and third commits
        let ignore_revs = vec![
            history_all.entries[0].commit_hash[..8].to_string(),
            history_all.entries[2].commit_hash[..8].to_string(),
        ];

        let history_filtered = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &ignore_revs, None, None)
            .unwrap();

        // Should have only one commit remaining
        assert_eq!(history_filtered.entries.len(), 1);
        assert_eq!(
            history_filtered.entries[0].message,
            "Update line 1 - first change"
        );
    }

    #[test]
    fn test_git_adapter_ignore_nonexistent_revision() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Use a fake hash that doesn't exist
        let ignore_revs = vec!["fakehash123".to_string()];

        let history_normal = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        let history_with_fake_ignore = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &ignore_revs, None, None)
            .unwrap();

        // Should have the same number of commits since fake hash doesn't match anything
        assert_eq!(
            history_normal.entries.len(),
            history_with_fake_ignore.entries.len()
        );
    }

    #[test]
    fn test_git_adapter_parse_date_iso8601() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Test parsing various ISO 8601 formats
        let iso_date = "2023-01-01T00:00:00Z";
        let parsed = adapter.parse_git_date(iso_date).unwrap();
        assert_eq!(parsed.timestamp(), 1672531200); // 2023-01-01 UTC

        let iso_with_tz = "2023-01-01T09:00:00+09:00";
        let parsed_tz = adapter.parse_git_date(iso_with_tz).unwrap();
        assert_eq!(parsed_tz.timestamp(), 1672531200); // Same UTC time
    }

    #[test]
    fn test_git_adapter_parse_date_simple_formats() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Test simple date format
        let simple_date = "2023-01-01";
        let parsed = adapter.parse_git_date(simple_date).unwrap();
        assert_eq!(parsed.format("%Y-%m-%d").to_string(), "2023-01-01");

        // Test datetime format
        let datetime = "2023-01-01 12:00:00";
        let parsed_dt = adapter.parse_git_date(datetime).unwrap();
        assert_eq!(parsed_dt.format("%H").to_string(), "12");
    }

    #[test]
    fn test_git_adapter_parse_date_formats() {
        let temp_dir = setup_test_repo().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Test additional formats
        let iso_local = "2023-01-01T00:00:00";
        let parsed = adapter.parse_git_date(iso_local);
        assert!(parsed.is_err()); // Should fail without timezone

        // Test error case
        let invalid_date = "not-a-date";
        let result = adapter.parse_git_date(invalid_date);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_adapter_filter_by_since_date() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Get all commits first
        let history_all = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(history_all.entries.len(), 3);

        // Filter to only show commits from a specific date onwards
        // Use a timestamp between the first and second commit
        let since_date = "1970-01-01T00:25:00Z"; // 1500 seconds epoch
        let history_filtered = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], Some(since_date), None)
            .unwrap();

        // Should have fewer commits (only those after the since date)
        assert!(history_filtered.entries.len() <= history_all.entries.len());
        assert!(history_filtered.entries.len() >= 2); // Should have at least 2 commits
    }

    #[test]
    fn test_git_adapter_filter_by_until_date() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Get all commits first
        let history_all = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(history_all.entries.len(), 3);

        // Filter to only show commits up to a specific date
        // Use a timestamp between the second and third commit
        let until_date = "1970-01-01T00:35:00Z"; // 2100 seconds epoch
        let history_filtered = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, Some(until_date))
            .unwrap();

        // Should have fewer commits (only those before the until date)
        assert!(history_filtered.entries.len() <= history_all.entries.len());
        assert!(history_filtered.entries.len() >= 1); // Should have at least 1 commit
    }

    #[test]
    fn test_git_adapter_filter_by_date_range() {
        let temp_dir = setup_test_repo_with_multiple_commits().unwrap();
        let adapter = GitAdapter::new(temp_dir.path()).unwrap();

        // Get all commits first
        let history_all = adapter
            .get_line_history("test.txt", 1, SortOrder::Asc, &[], None, None)
            .unwrap();

        assert_eq!(history_all.entries.len(), 3);

        // Filter to a specific date range that should include only the middle commit
        let since_date = "1970-01-01T00:25:00Z"; // 1500 seconds
        let until_date = "1970-01-01T00:35:00Z"; // 2100 seconds
        let history_filtered = adapter
            .get_line_history(
                "test.txt",
                1,
                SortOrder::Asc,
                &[],
                Some(since_date),
                Some(until_date),
            )
            .unwrap();

        // Should have exactly 1 commit (the middle one)
        assert_eq!(history_filtered.entries.len(), 1);
        assert_eq!(
            history_filtered.entries[0].message,
            "Update line 1 - first change"
        );
    }
}
