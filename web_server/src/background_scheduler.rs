use crate::db::CombinedRepository;
use crate::scrape::CommentScraper;
use crate::scrape_task::ScrapeTask;
use crate::task_queue::TaskScheduler;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct BackgroundScheduler {
    repo: Arc<dyn CombinedRepository>,
    task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    scraper: Arc<dyn CommentScraper>,
    check_interval: Duration,
    cancellation_token: CancellationToken,
}

impl BackgroundScheduler {
    pub fn new(
        repo: Arc<dyn CombinedRepository>,
        task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
        scraper: Arc<dyn CommentScraper>,
        check_interval: Duration,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            repo,
            task_queue,
            scraper,
            check_interval,
            cancellation_token,
        }
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        info!(
            "Background scheduler started with interval: {:?}",
            self.check_interval
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match self.check_and_schedule_due_urls().await {
                        Ok(scheduled_count) => {
                            if scheduled_count > 0 {
                                info!("Scheduled {} URLs for refresh", scheduled_count);
                            }
                        }
                        Err(e) => {
                            error!("Error in background scheduler: {}", e);
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    info!("Background scheduler shutting down");
                    break;
                }
            }
        }
    }

    // REFACTOR: Return a specific error type.
    async fn check_and_schedule_due_urls(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let due_urls = self.repo.get_urls_due_for_refresh().await?;
        let mut scheduled_count = 0;

        for url_row in due_urls {
            let target_url = url_row.url.clone();
            let item_id = url_row.id;

            // Create scrape task
            let scrape_task =
                ScrapeTask::new(target_url, item_id, self.repo.clone(), self.scraper.clone());

            // Schedule the task
            match self.task_queue.schedule(scrape_task).await {
                Ok(true) => {
                    info!("Scheduled refresh for URL ID: {}", item_id);
                    scheduled_count += 1;
                }
                Ok(false) => {
                    info!("Refresh for URL ID {} already scheduled", item_id);
                }
                Err(e) => {
                    error!("Failed to schedule refresh for URL ID {}: {}", item_id, e);
                }
            }
        }

        Ok(scheduled_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::comments_repository::{CommentsRepository, DbCommentRow};
    use crate::db::links_repository::{DbUrlRow, LinksRepository, ScheduledUrl};
    use crate::scrape::{ScrapeError, ScrapeResult};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc::error::TrySendError;

    struct NoOpScraper;

    #[async_trait]
    impl CommentScraper for NoOpScraper {
        async fn get_comments(&self, _url: &str) -> Result<ScrapeResult, ScrapeError> {
            unimplemented!()
        }
    }

    struct MockRepo {
        urls: Mutex<Vec<ScheduledUrl>>,
    }

    impl MockRepo {
        fn new(urls: Vec<ScheduledUrl>) -> Self {
            Self {
                urls: Mutex::new(urls),
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
            Ok(vec![])
        }

        async fn upsert_comments(
            &self,
            _comments: &[crate::CommentRecord],
            _url_id: i64,
            _thread_month: Option<i32>,
            _thread_year: Option<i32>,
        ) -> Result<usize, sqlx::Error> {
            Ok(0)
        }

        async fn update_comment_state(&self, _id: i64, _state: i32) -> Result<(), sqlx::Error> {
            Ok(())
        }

        async fn get_comment(&self, _id: i64) -> Result<Option<DbCommentRow>, sqlx::Error> {
            Ok(None)
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
            Ok(self.urls.lock().unwrap().clone())
        }
        async fn get_url_by_id(&self, _id: i64) -> Result<Option<String>, sqlx::Error> {
            Ok(None)
        }
    }

    struct MockScheduler {
        scheduled: Mutex<Vec<ScrapeTask>>,
    }

    impl MockScheduler {
        fn new() -> Self {
            Self {
                scheduled: Mutex::new(vec![]),
            }
        }

        fn get_scheduled(&self) -> Vec<ScrapeTask> {
            self.scheduled.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl TaskScheduler<ScrapeTask> for MockScheduler {
        async fn schedule(&self, task: ScrapeTask) -> Result<bool, TrySendError<ScrapeTask>> {
            self.scheduled.lock().unwrap().push(task);
            Ok(true)
        }

        async fn shutdown(&self) {}
    }

    #[tokio::test]
    async fn run_exits_on_shutdown_signal() {
        let repo = Arc::new(MockRepo::new(vec![]));
        let scheduler = Arc::new(MockScheduler::new());

        let token = CancellationToken::new();

        let bg_scheduler = BackgroundScheduler::new(
            repo.clone(),
            scheduler.clone(),
            Arc::new(NoOpScraper),
            Duration::from_secs(3600), // 1-hour interval — tick never fires during test
            token.clone(),
        );

        let handle = tokio::spawn(async move { bg_scheduler.run().await });

        // Yield so the spawned task gets a full poll: it processes the initial tick
        // (MockRepo is synchronous, no Pending points) and loops back to select!
        // returning Pending. Only then do we cancel, ensuring cancelled() is the
        // only ready branch in select! — no pseudorandom branch selection.
        tokio::task::yield_now().await;

        token.cancel();

        tokio::time::timeout(std::time::Duration::from_secs(1), handle)
            .await
            .expect("run() must exit promptly after shutdown signal")
            .expect("task must not panic");
    }

    #[tokio::test]
    async fn test_schedules_due_urls() {
        let past_time = Utc::now() - chrono::Duration::hours(25);
        let url_row = ScheduledUrl {
            id: 123,
            url: "https://example.com".to_string(),
            last_scraped: Some(past_time),
            frequency_hours: 24,
            days_limit: 7,
            comment_count: 0,
            picked_comment_count: 0,
            thread_month: None,
            thread_year: None,
        };

        let repo = Arc::new(MockRepo::new(vec![url_row]));
        let scheduler = Arc::new(MockScheduler::new());

        let bg_scheduler = BackgroundScheduler::new(
            repo.clone(),
            scheduler.clone(),
            Arc::new(NoOpScraper),
            Duration::from_secs(60),
            CancellationToken::new(),
        );

        // Run one check cycle
        let scheduled = bg_scheduler.check_and_schedule_due_urls().await.unwrap();
        assert_eq!(scheduled, 1);

        let scheduled_tasks = scheduler.get_scheduled();
        assert_eq!(scheduled_tasks.len(), 1);
        assert_eq!(scheduled_tasks[0].url_id(), 123);
    }
}
