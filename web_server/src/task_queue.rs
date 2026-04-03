use async_trait::async_trait;
use log::{error, info};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[async_trait]
pub trait ExecutableTask: Display + Hash + Clone + Eq + Send + Sync + 'static {
    type ProcessorError: Error + Send + Sync + 'static;
    async fn execute(&self) -> Result<(), Self::ProcessorError>;
}

#[async_trait]
pub trait TaskScheduler<T>: Send + Sync
where
    T: ExecutableTask,
{
    async fn schedule(&self, task: T) -> Result<bool, TrySendError<T>>;
    async fn shutdown(&self);
}

pub struct TaskDedupQueue<T> {
    queue: Arc<Mutex<HashSet<Arc<T>>>>,
    tx: Sender<T>,
    shutdown_tx: watch::Sender<bool>,
    worker_handle: std::sync::Mutex<Option<JoinHandle<()>>>,
}

impl<T> TaskDedupQueue<T>
where
    T: ExecutableTask,
{
    pub fn new(buffer_size: usize) -> Self {
        let task_set = Arc::new(Mutex::new(HashSet::new()));

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<T>(buffer_size);
        let tasks = task_set.clone();
        let handle = tokio::spawn(async move {
            info!("Spawned task processor worker.");
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => break,
                    maybe_task = rx.recv() => {
                        match maybe_task {
                            Some(task) => {
                                info!("Scrape task received: {}", task);
                                let res = task.execute().await;
                                if let Err(e) = res {
                                    error!("Scrape task failed: {}", e);
                                }
                                tasks.lock().await.remove(&task);
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        TaskDedupQueue {
            queue: task_set,
            tx,
            shutdown_tx,
            worker_handle: std::sync::Mutex::new(Some(handle)),
        }
    }
}

#[async_trait]
impl<T> TaskScheduler<T> for TaskDedupQueue<T>
where
    T: ExecutableTask + Display + Hash + Clone + Eq + Send + Sync + 'static,
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
            if let Err(e) = self.tx.try_send(task) {
                task_set.remove(&task_ref);
                return Err(e);
            }
        }

        info!("Scrape task set size: {}", task_set.len());

        Ok(true)
    }

    async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
        let handle = self.worker_handle.lock().unwrap().take();
        if let Some(h) = handle {
            let _ = h.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_queue::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    use tokio::time::{self, Duration};

    struct TestTask {
        id: u64,
        delay: Duration,
        executed: Arc<AtomicUsize>,
        start_notify: Option<Arc<Notify>>,
        done_notify: Option<Arc<Notify>>,
        should_fail: bool,
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
                should_fail: self.should_fail,
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
    impl ExecutableTask for TestTask {
        type ProcessorError = DummyError;

        async fn execute(&self) -> Result<(), DummyError> {
            self.executed.fetch_add(1, Ordering::SeqCst);
            if let Some(started) = &self.start_notify {
                started.notify_one();
            }
            tokio::time::sleep(self.delay).await;
            if self.should_fail {
                return Err(DummyError {});
            }
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
        should_fail: bool,
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
                should_fail,
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

        let (t1, t1_start, _t1_done) = make_task(1, Duration::from_millis(10), &t1_executed, false);
        let (t2, t2_start, _t2_done) = make_task(2, Duration::from_millis(10), &t2_executed, false);

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
        let (t1, t1_start, _t1_done) = make_task(42, Duration::from_secs(10), &t1_executed, false);
        // schedule first
        let r1 = processor.schedule(t1).await.expect("send ok");
        assert!(r1);

        // let it start
        time::advance(Duration::from_millis(1)).await;
        let _ = t1_start.notified().await;

        // schedule duplicate while first still running
        let t2_executed = Arc::new(AtomicUsize::new(0));
        let (dup, _dup_start, _dup_done) =
            make_task(42, Duration::from_millis(10), &t2_executed, false);
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

        let (t1, _t1_start, t1_done) = make_task(7, Duration::from_millis(5), &t1_executed, false);
        let r1 = processor.schedule(t1).await.expect("send ok");
        assert!(r1);

        // allow executing to complete
        time::advance(Duration::from_millis(10)).await;
        let _ = t1_done.notified().await;

        // now schedule an identical task again; should be accepted and executed
        let t2_executed = Arc::new(AtomicUsize::new(0));
        let (t2, t2_start, _t2_done) = make_task(7, Duration::from_millis(5), &t2_executed, false);
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

        let (t1, t1_start, _t1_done) =
            make_task(100, Duration::from_millis(200), &t1_executed, false);
        let (t2, _t2_start, _t2_done) =
            make_task(101, Duration::from_millis(200), &t2_executed, false);
        let (t3, _t3_start, _t3_done) =
            make_task(102, Duration::from_millis(10), &t3_executed, false);

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

    #[tokio::test(flavor = "current_thread")]
    async fn send_failure_does_not_prevent_other_tasks_from_scheduling() {
        time::pause();

        // buffer=1: exactly one task can sit in the channel while the worker is busy
        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(1);
        let occupier_executed = Arc::new(AtomicUsize::new(0));
        let filler_executed = Arc::new(AtomicUsize::new(0));
        let t1_executed = Arc::new(AtomicUsize::new(0));
        let t2_executed = Arc::new(AtomicUsize::new(0));

        // Schedule occupier — worker picks it up and starts sleeping, leaving the channel empty
        let (occupier, occupier_start, _) =
            make_task(0, Duration::from_millis(100), &occupier_executed, false);
        assert!(processor.schedule(occupier).await.expect("send ok"));
        time::advance(Duration::from_millis(1)).await;
        occupier_start.notified().await; // confirm worker is now inside execute()

        // Fill the channel buffer so the next send will fail
        let (filler, _, _) = make_task(1, Duration::from_millis(100), &filler_executed, false);
        assert!(processor.schedule(filler).await.expect("send ok"));

        // t1: try_send fails — channel is full; the fix must remove t1 from the HashSet on failure
        let (t1, _, _) = make_task(2, Duration::from_millis(5), &t1_executed, false);
        assert!(
            processor.schedule(t1).await.is_err(),
            "t1 should fail: channel is full"
        );

        // Drain the channel: let occupier finish, then filler finish
        time::advance(Duration::from_millis(150)).await; // occupier done; worker picks up filler
        time::advance(Duration::from_millis(150)).await; // filler done; channel is now empty

        // t1 was removed from the HashSet on failure, so it can be re-scheduled now
        let (t1_retry, _, t1_done) = make_task(2, Duration::from_millis(5), &t1_executed, false);
        let r1 = processor.schedule(t1_retry).await.expect("send ok");
        assert!(r1, "t1 must be re-schedulable after its prior send failure");

        // Let t1_retry complete before filling the buffer again (buffer=1)
        time::advance(Duration::from_millis(1)).await; // worker picks up t1_retry
        time::advance(Duration::from_millis(10)).await; // t1_retry finishes
        t1_done.notified().await;

        // t2 is also unblocked
        let (t2, _, t2_done) = make_task(3, Duration::from_millis(5), &t2_executed, false);
        let r2 = processor.schedule(t2).await.expect("send ok");
        assert!(
            r2,
            "t2 should be scheduled successfully after the channel drains"
        );

        time::advance(Duration::from_millis(1)).await;
        time::advance(Duration::from_millis(10)).await;
        t2_done.notified().await;

        assert_eq!(
            t1_executed.load(Ordering::SeqCst),
            1,
            "t1 ran after being re-scheduled"
        );
        assert_eq!(
            t2_executed.load(Ordering::SeqCst),
            1,
            "t2 executed despite t1's prior failure"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_waits_for_inflight_task() {
        time::pause();

        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(8);
        let executed = Arc::new(AtomicUsize::new(0));

        let (task, start_notify, done_notify) =
            make_task(1, Duration::from_millis(100), &executed, false);
        processor.schedule(task).await.expect("send ok");

        // Let the worker pick up the task
        time::advance(Duration::from_millis(1)).await;
        start_notify.notified().await; // task is now executing inside the arm body

        // Advance past the task's delay so it can complete after shutdown() signals the worker
        time::advance(Duration::from_millis(200)).await;
        done_notify.notified().await;

        processor.shutdown().await;

        assert_eq!(executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_does_not_execute_queued_tasks() {
        time::pause();

        // buffer=2: worker can be busy with task 1 while task 2 sits in the channel
        let processor: TaskDedupQueue<TestTask> = TaskDedupQueue::new(2);
        let t1_executed = Arc::new(AtomicUsize::new(0));
        let t2_executed = Arc::new(AtomicUsize::new(0));

        let (t1, t1_start, _t1_done) =
            make_task(1, Duration::from_millis(100), &t1_executed, false);
        let (t2, _t2_start, _t2_done) =
            make_task(2, Duration::from_millis(10), &t2_executed, false);

        processor.schedule(t1).await.expect("send ok");
        // Advance time so the worker picks up t1 and starts executing it
        time::advance(Duration::from_millis(1)).await;
        t1_start.notified().await; // t1 is executing

        // Enqueue t2 while worker is busy — it sits in the channel buffer
        processor.schedule(t2).await.expect("send ok");

        // Send shutdown signal — when t1 finishes and the worker loops back to select!,
        // biased; ensures the shutdown branch wins over the buffered t2
        let _ = processor.shutdown_tx.send(true);

        // Advance time past t1's delay so it completes
        time::advance(Duration::from_millis(200)).await;
        processor.shutdown().await;

        assert_eq!(t1_executed.load(Ordering::SeqCst), 1, "t1 must complete");
        assert_eq!(
            t2_executed.load(Ordering::SeqCst),
            0,
            "t2 must not run after shutdown"
        );
    }
}
