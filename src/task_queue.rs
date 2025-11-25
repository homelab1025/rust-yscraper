use crate::scrape::scrape::ScrapeTask;
use log::info;
use std::collections::HashSet;
use std::error::Error;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::SendError;

pub struct ScrapeTaskDedupProcessor {
    // queue: Mutex<(VecDeque<Arc<ScrapeTask>>, HashSet<Arc<ScrapeTask>>)>,
    queue: Arc<Mutex<HashSet<Arc<ScrapeTask>>>>,
    tx: Sender<ScrapeTask>,
}

/// A tokio thread-safe task queue that ignores tasks that are duplicated. If a task is already present in the queue, it won't be added anymore.
/// It returns the number of added queued tasks or an error.
///If the task already exists it returns 0.
impl ScrapeTaskDedupProcessor {
    pub fn new(buffer_size: usize) -> Self {
        let task_set = Arc::new(Mutex::new(HashSet::new()));

        // start channel
        let (tx, mut rx) = tokio::sync::mpsc::channel::<ScrapeTask>(buffer_size);
        let tasks = task_set.clone();
        tokio::spawn(async move {
            info!("Spawned task processor worker.");
            while let Some(task) = rx.recv().await {
                info!("Scrape task received: {}", task);

                tokio::time::sleep(Duration::from_secs(1)).await;
                tasks.lock().await.remove(&task);
            }
        });

        ScrapeTaskDedupProcessor {
            // queue: Mutex::new((VecDeque::new(), HashSet::new())),
            queue: task_set.clone(),
            tx,
        }
    }

    pub(crate) async fn schedule(&self, task: ScrapeTask) -> Result<bool, SendError<ScrapeTask>> {
        info!("Scraping task: {}", task);
        // before pushing the message check if the task is already in the set
        // once the task is complete it should be removed from the set
        let mut guard = self.queue.lock().await;
        // let (deque, task_set) = &mut guard.deref_mut();
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
