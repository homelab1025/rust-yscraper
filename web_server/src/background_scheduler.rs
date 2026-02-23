use crate::db::CombinedRepository;
use crate::scrape_task::ScrapeTask;
use crate::task_queue::TaskScheduler;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;

pub struct BackgroundScheduler {
    repo: Arc<dyn CombinedRepository>,
    task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
    check_interval: Duration,
}

impl BackgroundScheduler {
    pub fn new(
        repo: Arc<dyn CombinedRepository>,
        task_queue: Arc<dyn TaskScheduler<ScrapeTask>>,
        check_interval: Duration,
    ) -> Self {
        Self {
            repo,
            task_queue,
            check_interval,
        }
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        info!(
            "Background scheduler started with interval: {:?}",
            self.check_interval
        );

        // REFACTOR: Wait on the tasks to finish and then shutdown when receiving the shutdown signal.
        loop {
            interval.tick().await;

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
            let scrape_task = ScrapeTask::new(target_url, item_id, self.repo.clone());

            // Schedule the task
            match self.task_queue.schedule(scrape_task).await {
                Ok(true) => {
                    info!("Scheduled refresh for URL ID: {}", item_id);
                    scheduled_count += 1;

                    // Update last_scraped timestamp to prevent immediate re-scheduling
                    if let Err(e) = self.repo.update_last_scraped(item_id).await {
                        error!(
                            "Failed to update last_scraped for URL ID {}: {}",
                            item_id, e
                        );
                    }
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
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc::error::TrySendError;

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
        async fn count_comments(&self, _url_id: Option<i64>) -> Result<i64, sqlx::Error> {
            Ok(0)
        }

        async fn page_comments(
            &self,
            _offset: i64,
            _count: i64,
            _url_id: Option<i64>,
        ) -> Result<Vec<DbCommentRow>, sqlx::Error> {
            Ok(vec![])
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

        async fn get_urls_due_for_refresh(&self) -> Result<Vec<ScheduledUrl>, sqlx::Error> {
            Ok(self.urls.lock().unwrap().clone())
        }

        async fn update_last_scraped(&self, _url_id: i64) -> Result<(), sqlx::Error> {
            Ok(())
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
        };

        let repo = Arc::new(MockRepo::new(vec![url_row]));
        let scheduler = Arc::new(MockScheduler::new());

        let bg_scheduler =
            BackgroundScheduler::new(repo.clone(), scheduler.clone(), Duration::from_secs(60));

        // Run one check cycle
        let scheduled = bg_scheduler.check_and_schedule_due_urls().await.unwrap();
        assert_eq!(scheduled, 1);

        let scheduled_tasks = scheduler.get_scheduled();
        assert_eq!(scheduled_tasks.len(), 1);
        assert_eq!(scheduled_tasks[0].url_id(), 123);
    }
}
