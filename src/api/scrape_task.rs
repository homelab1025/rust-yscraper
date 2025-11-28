use crate::scrape::scrape::{get_comments, ScrapeError};
use crate::task_queue::TaskQueueProcessor;
use crate::utils::create_batches;
use crate::{db, CommentRecord};
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
    repo: Arc<dyn db::CommentsRepository>,
}

impl ScrapeTask {
    pub fn new(url: String, url_id: i64, comments_repo: Arc<dyn db::CommentsRepository>) -> Self {
        ScrapeTask {
            url,
            url_id,
            repo: comments_repo.clone(),
        }
    }
}

impl Eq for ScrapeTask {}

impl PartialEq for ScrapeTask {
    fn eq(&self, other: &Self) -> bool {
        self.url_id == other.url_id && self.url == other.url
    }
}

impl Hash for ScrapeTask {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url_id.hash(state);
        self.url.hash(state);
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

        // Ensure the URL is recorded in the urls table and get (or confirm) its id
        if let Err(e) = self.repo.upsert_url(self.url_id, &self.url).await {
            error!("Failed to upsert url: {}", e);
            return Err(ScrapeTaskError::DatabaseError(e));
        }

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
            }
            Err(error) => return Err(ScrapeTaskError::ScrapingError(error)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_task_hashing {
    use super::ScrapeTask;
    use crate::db::{CommentsRepository, DbCommentRow};
    use async_trait::async_trait;
    use std::collections::HashSet;
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

        async fn upsert_url(&self, _id: i64, _url: &str) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
        ) -> Result<usize, sqlx::Error> {
            Ok(0)
        }
    }

    fn new_task(url: &str, url_id: i64) -> ScrapeTask {
        let repo: Arc<dyn CommentsRepository> = Arc::new(MockRepo {});
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
    fn hashset_distinguishes_different_url_ids() {
        let t1 = new_task("https://example.com/item/1", 1);
        let t2 = new_task("https://example.com/item/1", 2); // different id

        let mut set = HashSet::new();
        set.insert(t1);
        set.insert(t2);

        assert_eq!(
            set.len(),
            2,
            "Different url_id should produce distinct set entries"
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
}
