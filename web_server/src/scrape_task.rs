use crate::CommentRecord;
use crate::db::CombinedRepository;
use crate::scrape::{CommentScraper, ScrapeError};
use crate::task_queue::ExecutableTask;
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
    scraper: Arc<dyn CommentScraper>,
}

impl std::fmt::Debug for ScrapeTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrapeTask")
            .field("url", &self.url)
            .field("url_id", &self.url_id)
            .field("repo", &"<CombinedRepository>")
            .field("scraper", &"<Scraper>")
            .finish()
    }
}

impl ScrapeTask {
    pub fn new(
        url: String,
        url_id: i64,
        comments_repo: Arc<dyn CombinedRepository>,
        scraper: Arc<dyn CommentScraper>,
    ) -> Self {
        ScrapeTask {
            url,
            url_id,
            repo: comments_repo,
            scraper,
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
impl ExecutableTask for ScrapeTask {
    type ProcessorError = ScrapeTaskError;

    async fn execute(&self) -> Result<(), ScrapeTaskError> {
        info!("Executing Scrape TASK.");

        info!("Started task for scraping for {}", self.url);
        let comments_retrieval = self.scraper.get_comments(&self.url).await;
        match comments_retrieval {
            Ok(scrape_result) => {
                let comments = scrape_result.comments;
                info!("Parsed {} root comments", comments.len());

                let batches: Vec<Vec<CommentRecord>> =
                    comments.chunks(10).map(|chunk| chunk.to_vec()).collect();
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

                // Update metadata and timestamps
                if let Err(e) = self.repo.update_comment_count(self.url_id).await {
                    error!("Failed to update comment count: {}", e);
                }
                if let Err(e) = self
                    .repo
                    .update_thread_metadata(
                        self.url_id,
                        scrape_result.thread_month,
                        scrape_result.thread_year,
                    )
                    .await
                {
                    error!("Failed to update thread metadata: {}", e);
                }
            }
            Err(error) => {
                error!("Scraping failed for {}: {}", self.url, error);
                return Err(ScrapeTaskError::ScrapingError(error));
            }
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
    use crate::scrape::{CommentScraper, ScrapeError, ScrapeResult};
    use async_trait::async_trait;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::sync::{Arc, Mutex};

    struct NoOpScraper;

    #[async_trait]
    impl CommentScraper for NoOpScraper {
        async fn get_comments(&self, _url: &str) -> Result<ScrapeResult, ScrapeError> {
            unimplemented!()
        }
    }

    struct MockRepo {
        update_thread_metadata_called: Mutex<Option<(i64, Option<i32>, Option<i32>)>>,
    }

    #[async_trait]
    impl CommentsRepository for MockRepo {
        async fn count_comments(
            &self,
            _url_id: i64,
            _state: Option<i32>,
        ) -> Result<u32, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
            _url_id: i64,
            _state: Option<i32>,
            _sort_by: Option<crate::SortBy>,
            _sort_order: Option<crate::SortOrder>,
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

        async fn update_comment_state(&self, _id: i64, _state: i32) -> Result<(), sqlx::Error> {
            Ok(())
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

        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_comment_count(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn update_thread_metadata(
            &self,
            url_id: i64,
            month: Option<i32>,
            year: Option<i32>,
        ) -> Result<(), sqlx::Error> {
            let mut called = self.update_thread_metadata_called.lock().unwrap();
            *called = Some((url_id, month, year));
            Ok(())
        }
    }

    fn new_task(url: &str, url_id: i64) -> ScrapeTask {
        let repo: Arc<dyn CombinedRepository> = Arc::new(MockRepo {
            update_thread_metadata_called: Mutex::new(None),
        });
        ScrapeTask::new(url.to_string(), url_id, repo, Arc::new(NoOpScraper))
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

#[cfg(test)]
mod tests_execute {
    use super::{ScrapeTask, ScrapeTaskError};
    use crate::db::CombinedRepository;
    use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use crate::scrape::{CommentScraper, ScrapeError, ScrapeResult};
    use crate::task_queue::ExecutableTask;
    use crate::{CommentRecord, CommentState};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockScraper(Mutex<Option<Result<ScrapeResult, ScrapeError>>>);

    impl MockScraper {
        fn returning(r: Result<ScrapeResult, ScrapeError>) -> Self {
            MockScraper(Mutex::new(Some(r)))
        }
    }

    #[async_trait]
    impl CommentScraper for MockScraper {
        async fn get_comments(&self, _url: &str) -> Result<ScrapeResult, ScrapeError> {
            self.0.lock().unwrap().take().unwrap()
        }
    }

    struct MockRepo {
        upsert_calls: Mutex<Vec<(Vec<CommentRecord>, i64)>>,
        update_comment_count_calls: Mutex<Vec<i64>>,
        update_thread_metadata_called: Mutex<Option<(i64, Option<i32>, Option<i32>)>>,
        upsert_error: Option<String>,
        update_count_error: Option<String>,
        update_metadata_error: Option<String>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                upsert_calls: Mutex::new(vec![]),
                update_comment_count_calls: Mutex::new(vec![]),
                update_thread_metadata_called: Mutex::new(None),
                upsert_error: None,
                update_count_error: None,
                update_metadata_error: None,
            }
        }
    }

    #[async_trait]
    impl CommentsRepository for MockRepo {
        async fn count_comments(
            &self,
            _url_id: i64,
            _state: Option<i32>,
        ) -> Result<u32, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
            _url_id: i64,
            _state: Option<i32>,
            _sort_by: Option<crate::SortBy>,
            _sort_order: Option<crate::SortOrder>,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            Ok(Vec::new())
        }

        async fn upsert_comments(
            &self,
            comments: &[CommentRecord],
            url_id: i64,
        ) -> Result<usize, sqlx::Error> {
            if let Some(msg) = &self.upsert_error {
                return Err(sqlx::Error::Protocol(msg.clone()));
            }
            let n = comments.len();
            self.upsert_calls
                .lock()
                .unwrap()
                .push((comments.to_vec(), url_id));
            Ok(n)
        }

        async fn update_comment_state(&self, _id: i64, _state: i32) -> Result<(), sqlx::Error> {
            Ok(())
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

        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_comment_count(&self, url_id: i64) -> Result<(), sqlx::Error> {
            if let Some(msg) = &self.update_count_error {
                return Err(sqlx::Error::Protocol(msg.clone()));
            }
            self.update_comment_count_calls.lock().unwrap().push(url_id);
            Ok(())
        }

        async fn update_thread_metadata(
            &self,
            url_id: i64,
            month: Option<i32>,
            year: Option<i32>,
        ) -> Result<(), sqlx::Error> {
            if let Some(msg) = &self.update_metadata_error {
                return Err(sqlx::Error::Protocol(msg.clone()));
            }
            let mut called = self.update_thread_metadata_called.lock().unwrap();
            *called = Some((url_id, month, year));
            Ok(())
        }
    }

    fn make_comment(id: i64) -> CommentRecord {
        CommentRecord {
            id,
            author: format!("user{}", id),
            date: "2025-01-01".to_string(),
            text: "some text".to_string(),
            tags: vec![],
            state: CommentState::New,
            subcomment_count: 0,
        }
    }

    fn make_scrape_result(comments: Vec<CommentRecord>) -> ScrapeResult {
        ScrapeResult {
            comments,
            thread_month: Some(1),
            thread_year: Some(2025),
        }
    }

    fn make_task(
        scraper: Arc<dyn CommentScraper>,
        repo: Arc<dyn CombinedRepository>,
    ) -> ScrapeTask {
        ScrapeTask::new("https://example.com/item/1".to_string(), 42, repo, scraper)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_happy_path_returns_ok() {
        let comments = vec![make_comment(1), make_comment(2), make_comment(3)];
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(comments))));
        let repo = Arc::new(MockRepo::new());
        let repo_ref = Arc::clone(&repo);
        let task = make_task(scraper, repo);

        let result = task.execute().await;

        assert!(result.is_ok());
        assert_eq!(repo_ref.upsert_calls.lock().unwrap().len(), 1);
        assert_eq!(repo_ref.update_comment_count_calls.lock().unwrap().len(), 1);
        assert!(
            repo_ref
                .update_thread_metadata_called
                .lock()
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_batches_11_comments_into_two_upsert_calls() {
        let comments: Vec<CommentRecord> = (1..=11).map(make_comment).collect();
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(comments))));
        let repo = Arc::new(MockRepo::new());
        let repo_ref = Arc::clone(&repo);
        let task = make_task(scraper, repo);

        task.execute().await.unwrap();

        let calls = repo_ref.upsert_calls.lock().unwrap();
        assert_eq!(calls.len(), 2, "expected 2 batches (10 + 1)");
        assert_eq!(calls[0].0.len(), 10);
        assert_eq!(calls[1].0.len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_zero_comments_still_updates_count_and_metadata() {
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(vec![]))));
        let repo = Arc::new(MockRepo::new());
        let repo_ref = Arc::clone(&repo);
        let task = make_task(scraper, repo);

        task.execute().await.unwrap();

        assert_eq!(repo_ref.upsert_calls.lock().unwrap().len(), 0);
        assert_eq!(repo_ref.update_comment_count_calls.lock().unwrap().len(), 1);
        assert!(
            repo_ref
                .update_thread_metadata_called
                .lock()
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_scrape_error_returns_err() {
        let scraper = Arc::new(MockScraper::returning(Err(
            ScrapeError::ElementSelectorError(),
        )));
        let repo = Arc::new(MockRepo::new());
        let repo_ref = Arc::clone(&repo);
        let task = make_task(scraper, repo);

        let result = task.execute().await;

        assert!(matches!(result, Err(ScrapeTaskError::ScrapingError(_))));
        assert_eq!(repo_ref.upsert_calls.lock().unwrap().len(), 0);
        assert_eq!(repo_ref.update_comment_count_calls.lock().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_upsert_error_still_returns_ok() {
        let comments = vec![make_comment(1)];
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(comments))));
        let repo = Arc::new(MockRepo {
            upsert_error: Some("db down".to_string()),
            ..MockRepo::new()
        });
        let task = make_task(scraper, repo);

        let result = task.execute().await;

        assert!(result.is_ok(), "upsert errors are soft failures");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_update_count_error_still_returns_ok() {
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(vec![]))));
        let repo = Arc::new(MockRepo {
            update_count_error: Some("db down".to_string()),
            ..MockRepo::new()
        });
        let task = make_task(scraper, repo);

        let result = task.execute().await;

        assert!(
            result.is_ok(),
            "update_comment_count errors are soft failures"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execute_update_metadata_error_still_returns_ok() {
        let scraper = Arc::new(MockScraper::returning(Ok(make_scrape_result(vec![]))));
        let repo = Arc::new(MockRepo {
            update_metadata_error: Some("db down".to_string()),
            ..MockRepo::new()
        });
        let task = make_task(scraper, repo);

        let result = task.execute().await;

        assert!(
            result.is_ok(),
            "update_thread_metadata errors are soft failures"
        );
    }
}

#[cfg(test)]
mod tests_constructor {
    use super::ScrapeTask;
    use crate::db::CombinedRepository;
    use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use crate::scrape::{CommentScraper, ScrapeError, ScrapeResult};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct NoOpScraper;

    #[async_trait]
    impl CommentScraper for NoOpScraper {
        async fn get_comments(&self, _url: &str) -> Result<ScrapeResult, ScrapeError> {
            unimplemented!()
        }
    }

    fn no_op_scraper() -> Arc<dyn CommentScraper> {
        Arc::new(NoOpScraper)
    }

    struct MockRepo;

    #[async_trait]
    impl CommentsRepository for MockRepo {
        async fn count_comments(
            &self,
            _url_id: i64,
            _state: Option<i32>,
        ) -> Result<u32, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
            _url_id: i64,
            _state: Option<i32>,
            _sort_by: Option<crate::SortBy>,
            _sort_order: Option<crate::SortOrder>,
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

        async fn update_comment_state(&self, _id: i64, _state: i32) -> Result<(), sqlx::Error> {
            Ok(())
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

        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(vec![])
        }

        async fn update_comment_count(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn update_thread_metadata(
            &self,
            _url_id: i64,
            _month: Option<i32>,
            _year: Option<i32>,
        ) -> Result<(), sqlx::Error> {
            Ok(())
        }
    }

    fn new_repo() -> Arc<dyn CombinedRepository> {
        Arc::new(MockRepo)
    }

    #[test]
    fn new_stores_url_and_url_id() {
        let task = ScrapeTask::new(
            "https://example.com/item/42".to_string(),
            42,
            new_repo(),
            no_op_scraper(),
        );
        assert_eq!(task.url(), "https://example.com/item/42");
        assert_eq!(task.url_id(), 42);
    }

    #[test]
    fn new_url_id_boundary_zero() {
        let task = ScrapeTask::new(
            "https://example.com/item/0".to_string(),
            0,
            new_repo(),
            no_op_scraper(),
        );
        assert_eq!(task.url_id(), 0);
    }

    #[test]
    fn new_url_id_negative() {
        let task = ScrapeTask::new(
            "https://example.com/item/-1".to_string(),
            -1,
            new_repo(),
            no_op_scraper(),
        );
        assert_eq!(task.url_id(), -1);
    }
}
