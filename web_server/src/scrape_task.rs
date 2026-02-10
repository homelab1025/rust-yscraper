use crate::CommentRecord;
use crate::db::CombinedRepository;
use crate::scrape::{ScrapeError, get_comments};
use crate::task_queue::TaskQueueProcessor;
use crate::utils::create_batches;
use async_trait::async_trait;
use log::{error, info};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
pub struct ScrapeTask {
    url: String,
    url_id: i64,
    repo: Arc<dyn CombinedRepository>,
}

impl std::fmt::Debug for ScrapeTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrapeTask")
            .field("url", &self.url)
            .field("url_id", &self.url_id)
            .field("repo", &"<CombinedRepository>")
            .finish()
    }
}

impl ScrapeTask {
    pub fn new(url: String, url_id: i64, comments_repo: Arc<dyn CombinedRepository>) -> Self {
        ScrapeTask {
            url,
            url_id,
            repo: comments_repo.clone(),
        }
    }

    pub fn url_id(&self) -> i64 {
        self.url_id
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Eq for ScrapeTask {}

impl PartialEq for ScrapeTask {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.url_id == other.url_id
    }
}

impl Hash for ScrapeTask {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.url_id.hash(state);
    }
}

impl Display for ScrapeTask {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "\nurl id: {}\nurl: {}", self.url_id, self.url)
    }
}

#[derive(Debug)]
pub enum ScrapeTaskError {
    DatabaseError(sqlx::Error),
    ScrapingError(ScrapeError),
}

impl Error for ScrapeTaskError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            ScrapeTaskError::DatabaseError(e) => Some(e),
            ScrapeTaskError::ScrapingError(e) => Some(e),
        }
    }
}
impl Display for ScrapeTaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeTaskError::DatabaseError(e) => write!(f, "{}", e),
            ScrapeTaskError::ScrapingError(e) => write!(f, "{}", e),
        }
    }
}

#[async_trait]
impl TaskQueueProcessor for ScrapeTask {
    type ProcessorError = ScrapeTaskError;

    async fn execute(&self) -> Result<(), ScrapeTaskError> {
        info!("Executing Scrape TASK.");

        info!("/scrape called; starting scraping for {}", self.url);
        let comments_retrieval = get_comments(&self.url).await;
        match comments_retrieval {
            Ok(comments) => {
                info!("Parsed {} root comments", comments.len());

                let batches: Vec<Vec<CommentRecord>> = create_batches(&comments, 10);
                let mut total_inserted = 0usize;
                for batch in batches.iter() {
                    match self.repo.upsert_comments(batch, self.url_id).await {
                        Ok(n) => {
                            total_inserted += n;
                            info!("Inserted {} comments into the database", n);
                        }
                        Err(e) => error!("Failed to insert comments: {}", e),
                    }
                }

                info!("Scraping complete; {} comments inserted", total_inserted);

                // Update last_scraped timestamp for scheduling purposes
                if let Err(e) = self.repo.update_last_scraped(self.url_id).await {
                    error!("Failed to update last_scraped timestamp: {}", e);
                }
            }
            Err(error) => return Err(ScrapeTaskError::ScrapingError(error)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_task_hashing {
    use super::ScrapeTask;
    use crate::db::CombinedRepository;
    use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;

    struct MockRepo;

    #[async_trait]
    impl CommentsRepository for MockRepo {
        async fn count_comments(&self) -> Result<i64, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            Ok(Vec::new())
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
        ) -> Result<usize, sqlx::Error> {
            Ok(0)
        }
    }

    #[async_trait]
    impl LinksRepository for MockRepo {
        async fn list_links(&self) -> Result<Vec<DbUrlRow>, sqlx::Error> {
            Ok(vec![])
        }

        async fn delete_link(&self, _id: i64) -> Result<u64, sqlx::Error> {
            Ok(0)
        }

        async fn upsert_url_with_scheduling(
            &self,
            _id: i64,
            _url: &str,
            _frequency_hours: u32,
            _days_limit: u32,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_urls_due_for_refresh(
            &self,
        ) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_last_scraped(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }
    }

    fn new_task(url: &str, url_id: i64) -> ScrapeTask {
        let repo: Arc<dyn CombinedRepository> = Arc::new(MockRepo {});
        ScrapeTask::new(url.to_string(), url_id, repo)
    }

    #[test]
    fn hashset_deduplicates_equal_tasks() {
        // Same url_id and url -> tasks considered equal and hash to same value.
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 1);

        let mut set = HashSet::new();
        set.insert(t1);
        set.insert(t2);

        assert_eq!(
            set.len(),
            1,
            "HashSet should deduplicate equal ScrapeTask items"
        );
    }

    #[test]
    fn hashset_distinguishes_same_url_different_ids() {
        // Same URL but different url_id -> should be considered different
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 999); // different id

        let mut set = HashSet::new();
        set.insert(t1);
        set.insert(t2);

        assert_eq!(
            set.len(),
            2,
            "Same URL with different url_id should be distinct"
        );
    }

    #[test]
    fn hashset_distinguishes_different_urls() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/2", 1); // different url

        let mut set = HashSet::new();
        set.insert(t1);
        set.insert(t2);

        assert_eq!(
            set.len(),
            2,
            "Different url should produce distinct set entries"
        );
    }

    #[test]
    fn test_equality_same_url_and_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 1);

        assert_eq!(t1, t2, "Tasks with same URL and url_id should be equal");
    }

    #[test]
    fn test_inequality_same_url_different_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 999);

        assert_ne!(
            t1, t2,
            "Tasks with same URL but different url_id should not be equal"
        );
    }

    #[test]
    fn test_inequality_different_url_same_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/2", 1);

        assert_ne!(
            t1, t2,
            "Tasks with different URL but same url_id should not be equal"
        );
    }

    #[test]
    fn test_inequality_different_url_and_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/2", 999);

        assert_ne!(
            t1, t2,
            "Tasks with different URL and url_id should not be equal"
        );
    }

    #[test]
    fn test_hash_consistency_same_url_and_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 1);

        // Same URL and url_id should have same hash
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        t1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        t2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2, "Same URL and url_id should have same hash");
    }

    #[test]
    fn test_hash_difference_same_url_different_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 999);

        // Same URL but different url_id should have different hashes
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        t1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        t2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_ne!(
            hash1, hash2,
            "Same URL but different url_id should have different hashes"
        );
    }

    #[test]
    fn test_hash_difference_different_url_same_id() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/2", 1);

        // Different URL but same url_id should have different hashes
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        t1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        t2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_ne!(
            hash1, hash2,
            "Different URL but same url_id should have different hashes"
        );
    }
}
