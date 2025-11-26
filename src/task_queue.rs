use crate::scrape::scrape::ScrapeTask;
use async_trait::async_trait;
use log::info;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

#[async_trait]
pub trait TaskQueueProcessor {
    async fn execute(&self) -> Result<(), Box<dyn Error>>;
}

pub struct TaskDedupQueueProcessor<T: TaskQueueProcessor> {
    queue: Arc<Mutex<HashSet<Arc<T>>>>,
    tx: Sender<T>,
}

impl<T> TaskDedupQueueProcessor<T>
where
    T: TaskQueueProcessor + Display + Hash + Clone + Eq + Send + Sync + 'static,
{
    pub fn new(buffer_size: usize) -> Self {
        let task_set = Arc::new(Mutex::new(HashSet::new()));

        // start channel
        let (tx, mut rx) = tokio::sync::mpsc::channel::<T>(buffer_size);
        let tasks = task_set.clone();
        tokio::spawn(async move {
            info!("Spawned task processor worker.");
            while let Some(task) = rx.recv().await {
                info!("Scrape task received: {}", task);

                let _ = task.execute().await;

                tokio::time::sleep(Duration::from_secs(1)).await;
                tasks.lock().await.remove(&task);
            }
        });

        TaskDedupQueueProcessor {
            queue: task_set.clone(),
            tx,
        }
    }

    pub(crate) async fn schedule(&self, task: T) -> Result<bool, SendError<T>> {
        info!("Scraping task: {}", task);
        let mut guard = self.queue.lock().await;
        let task_set = guard.deref_mut();

        let task_arc = Arc::new(task.clone());
        if task_set.contains(&task_arc) {
            drop(guard);
            return Ok(false);
        } else {
            // deque.push_front(task_arc.clone());
            task_set.insert(task_arc.clone());
            self.tx.send(task).await?;
        }

        info!("Scrape task set size: {}", task_set.len());

        Ok(true)
    }
}
