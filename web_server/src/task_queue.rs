use async_trait::async_trait;
use log::{error, info};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

#[async_trait]
pub trait TaskQueueProcessor: Display + Hash + Clone + Eq + Send + Sync + 'static {
    type ProcessorError: Error + Send + Sync + 'static;
    async fn execute(&self) -> Result<(), Self::ProcessorError>;
}

#[async_trait]
pub trait TaskScheduler<T>: Send + Sync
where
    T: TaskQueueProcessor,
{
    async fn schedule(&self, task: T) -> Result<bool, TrySendError<T>>;
}

pub struct TaskDedupQueue<T> {
    queue: Arc<Mutex<HashSet<Arc<T>>>>,
    tx: Sender<T>,
}

impl<T> TaskDedupQueue<T>
where
    T: TaskQueueProcessor,
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

                let res = task.execute().await;
                if let Err(e) = res {
                    error!("Scrape task failed: {}", e);
                }

                tasks.lock().await.remove(&task);
            }
        });

        TaskDedupQueue {
            queue: task_set.clone(),
            tx,
        }
    }
}

#[async_trait]
impl<T> TaskScheduler<T> for TaskDedupQueue<T>
where
    T: TaskQueueProcessor + Display + Hash + Clone + Eq + Send + Sync + 'static,
{
    async fn schedule(&self, task: T) -> Result<bool, TrySendError<T>> {
        info!("Scraping task: {}", task);
        let mut guard = self.queue.lock().await;
        let task_set = guard.deref_mut();

        let task_ref = Arc::new(task.clone());
        if task_set.contains(&task_ref) {
            drop(guard);
            return Ok(false);
        } else {
            task_set.insert(task_ref.clone());
            // self.tx.send(task).await?;
            let _sent = self.tx.try_send(task)?;
        }

        info!("Scrape task set size: {}", task_set.len());

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_queue::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::Notify;
    use tokio::time::{self, Duration};

    struct TestTask {
        id: u64,
        delay: Duration,
        executed: Arc<AtomicUsize>,
        start_notify: Option<Arc<Notify>>,
        done_notify: Option<Arc<Notify>>,
    }

    impl Clone for TestTask {
        fn clone(&self) -> Self {
            // For clone, we intentionally drop notifiers to keep clones side effect free
            Self {
                id: self.id,
                delay: self.delay,
                executed: self.executed.clone(),
                start_notify: None,
                done_notify: None,
            }
        }
    }

    impl PartialEq for TestTask {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl Eq for TestTask {}

    impl Hash for TestTask {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state)
        }
    }

    impl Display for TestTask {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "TestTask({})", self.id)
        }
    }

    #[derive(Debug)]
    struct DummyError {}
    impl Error for DummyError {}
    impl Display for DummyError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "DummyError")
        }
    }

    #[async_trait]
    impl TaskQueueProcessor for TestTask {
        type ProcessorError = DummyError;

        async fn execute(&self) -> Result<(), DummyError> {
            self.executed.fetch_add(1, Ordering::SeqCst);
            if let Some(started) = &self.start_notify {
                started.notify_one();
            }
            tokio::time::sleep(self.delay).await;
            if let Some(done) = &self.done_notify {
                done.notify_one();
            }
            Ok(())
        }
    }

    fn make_task(
        id: u64,
        delay: Duration,
        executed: &Arc<AtomicUsize>,
    ) -> (TestTask, Arc<Notify>, Arc<Notify>) {
        let started = Arc::new(Notify::new());
        let done = Arc::new(Notify::new());
        (
            TestTask {
                id,
                delay,
                executed: executed.clone(),
                start_notify: Some(started.clone()),
                done_notify: Some(done.clone()),
            },
            started,
            done,
        )
    }

    #[tokio::test(flavor = "current_thread")]
    async fn different_tasks_are_enqueued_and_both_execute() {
        time::pause();

        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(8);
        let t1_executed = Arc::new(AtomicUsize::new(0));
        let t2_executed = Arc::new(AtomicUsize::new(0));

        let (t1, t1_start, _t1_done) = make_task(1, Duration::from_millis(10), &t1_executed);
        let (t2, t2_start, _t2_done) = make_task(2, Duration::from_millis(10), &t2_executed);

        // schedule two different tasks
        let r1 = processor.schedule(t1).await.expect("send ok");
        let r2 = processor.schedule(t2).await.expect("send ok");
        assert!(r1);
        assert!(r2);

        // both should start executing
        time::advance(Duration::from_millis(1)).await; // let worker pick them up
        let _ = t1_start.notified().await;
        let _ = t2_start.notified().await;

        assert_eq!(t1_executed.load(Ordering::SeqCst), 1);
        assert_eq!(t2_executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn duplicate_while_first_running_second_is_rejected_and_not_executed() {
        time::pause();

        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(8);

        let t1_executed = Arc::new(AtomicUsize::new(0));
        let (t1, t1_start, _t1_done) = make_task(42, Duration::from_secs(10), &t1_executed);
        // schedule first
        let r1 = processor.schedule(t1).await.expect("send ok");
        assert!(r1);

        // let it start
        time::advance(Duration::from_millis(1)).await;
        let _ = t1_start.notified().await;

        // schedule duplicate while first still running
        let t2_executed = Arc::new(AtomicUsize::new(0));
        let (dup, _dup_start, _dup_done) = make_task(42, Duration::from_millis(10), &t2_executed);
        let r2 = processor.schedule(dup).await.expect("send ok");
        assert!(!r2, "duplicate should be rejected");

        // only the first should have executed (started once)
        assert_eq!(t1_executed.load(Ordering::SeqCst), 1);
        assert_eq!(t2_executed.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn identical_task_runs_again_after_first_finished_and_removed() {
        time::pause();

        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(8);
        let t1_executed = Arc::new(AtomicUsize::new(0));

        let (t1, _t1_start, t1_done) = make_task(7, Duration::from_millis(5), &t1_executed);
        let r1 = processor.schedule(t1).await.expect("send ok");
        assert!(r1);

        // allow executing to complete
        time::advance(Duration::from_millis(10)).await;
        let _ = t1_done.notified().await;

        // an internal worker waits extra 1 s before removal; advance exactly that
        // time::advance(Duration::from_secs(1)).await;

        // now schedule an identical task again; should be accepted and executed
        let t2_executed = Arc::new(AtomicUsize::new(0));
        let (t2, t2_start, _t2_done) = make_task(7, Duration::from_millis(5), &t2_executed);
        let r2 = processor.schedule(t2).await.expect("send ok");
        assert!(
            r2,
            "should accept after previous finished and removal delay elapsed"
        );

        time::advance(Duration::from_millis(1)).await;
        let _ = t2_start.notified().await;

        assert_eq!(t1_executed.load(Ordering::SeqCst), 1);
        assert_eq!(t2_executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn buffer_size_exceeded_blocks_additional_scheduling() {
        time::pause();

        // In this test we do not pause time; we use real short delays to observe backpressure.
        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(1);
        let t1_executed = Arc::new(AtomicUsize::new(0));
        let t2_executed = Arc::new(AtomicUsize::new(0));
        let t3_executed = Arc::new(AtomicUsize::new(0));

        let (t1, t1_start, _t1_done) = make_task(100, Duration::from_millis(200), &t1_executed);
        let (t2, _t2_start, _t2_done) = make_task(101, Duration::from_millis(200), &t2_executed);
        let (t3, _t3_start, _t3_done) = make_task(102, Duration::from_millis(10), &t3_executed);

        // schedule the first two tasks; with buffer=1, t1 will be received immediately, t2 will fill the channel buffer
        assert!(processor.schedule(t1).await.expect("send ok"));
        time::advance(Duration::from_millis(1)).await;
        assert!(processor.schedule(t2).await.expect("send ok"));
        // 3rd schedule should return the error immediately.
        let schedule_third = processor.schedule(t3);

        let _ = t1_start.notified().await;

        assert!(schedule_third.await.is_err());
        assert_eq!(t1_executed.load(Ordering::SeqCst), 1);
        assert_eq!(t2_executed.load(Ordering::SeqCst), 0);
        assert_eq!(t3_executed.load(Ordering::SeqCst), 0);
    }

    // TODO: test to check that a failing task does not crash the loop
}
