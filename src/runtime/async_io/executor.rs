//! Task executor for async operations
//!
//! Manages async tasks that are waiting for I/O or timers.

use super::{Interest, Reactor, Token};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl TaskId {
    fn next() -> Self {
        TaskId(TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// State of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run
    Ready,
    /// Task is blocked waiting for I/O
    Blocked { token: Token, interest: Interest },
    /// Task is blocked waiting for a timer
    Timer { token: Token },
    /// Task has completed
    Completed,
}

/// A task that can be scheduled
pub struct Task {
    /// Unique identifier
    pub id: TaskId,
    /// Current state
    pub state: TaskState,
    /// Callback to run when ready
    pub callback: Box<dyn FnMut(&mut TaskContext) -> TaskResult + Send>,
    /// User data
    pub data: Option<Box<dyn std::any::Any + Send>>,
}

/// Result from running a task
pub enum TaskResult {
    /// Task completed successfully
    Done,
    /// Task yielded, should be rescheduled
    Yield,
    /// Task is waiting for I/O
    WaitIo { token: Token, interest: Interest },
    /// Task is waiting for a timer
    WaitTimer { token: Token },
    /// Task encountered an error
    Error(String),
}

/// Context passed to task callbacks
pub struct TaskContext {
    /// Result of the current I/O operation (if any)
    pub io_result: Option<std::io::Result<usize>>,
    /// Whether a timer expired
    pub timer_expired: bool,
}

impl TaskContext {
    fn new() -> Self {
        Self {
            io_result: None,
            timer_expired: false,
        }
    }
}

/// Task executor
pub struct Executor {
    /// All tasks by ID
    tasks: HashMap<TaskId, Task>,
    /// Ready queue
    ready: VecDeque<TaskId>,
    /// Tasks waiting for I/O (token -> task_id)
    waiting_io: HashMap<Token, TaskId>,
    /// Tasks waiting for timers (token -> task_id)
    waiting_timer: HashMap<Token, TaskId>,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            ready: VecDeque::new(),
            waiting_io: HashMap::new(),
            waiting_timer: HashMap::new(),
        }
    }

    /// Spawn a new task
    pub fn spawn(&mut self, mut task: Task) -> TaskId {
        let id = TaskId::next();
        task.id = id;
        task.state = TaskState::Ready;
        self.ready.push_back(id);
        self.tasks.insert(id, task);
        id
    }

    /// Create a task from a callback
    pub fn spawn_fn<F>(&mut self, callback: F) -> TaskId
    where
        F: FnMut(&mut TaskContext) -> TaskResult + Send + 'static,
    {
        self.spawn(Task {
            id: TaskId(0), // Will be set by spawn
            state: TaskState::Ready,
            callback: Box::new(callback),
            data: None,
        })
    }

    /// Check if executor has any pending tasks
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Number of pending tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Wake a task waiting for I/O
    pub fn wake_io(&mut self, token: Token) {
        if let Some(task_id) = self.waiting_io.remove(&token) {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                task.state = TaskState::Ready;
                self.ready.push_back(task_id);
            }
        }
    }

    /// Wake a task waiting for a timer
    pub fn wake_timer(&mut self, token: Token) {
        if let Some(task_id) = self.waiting_timer.remove(&token) {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                task.state = TaskState::Ready;
                self.ready.push_back(task_id);
            }
        }
    }

    /// Run all ready tasks
    pub fn run_ready(&mut self, reactor: &Reactor) {
        // Wake tasks whose I/O is ready
        let ready_tokens: Vec<_> = reactor.ready_tokens().collect();
        for (token, _interest) in ready_tokens {
            self.wake_io(token);
        }

        // Run ready tasks
        while let Some(task_id) = self.ready.pop_front() {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                let mut ctx = TaskContext::new();

                // Check if I/O is ready
                if let TaskState::Blocked { token, interest } = task.state {
                    if reactor.is_ready(token, interest) {
                        ctx.io_result = Some(Ok(0)); // Placeholder
                    }
                }

                // Run the task callback
                let result = (task.callback)(&mut ctx);

                match result {
                    TaskResult::Done => {
                        task.state = TaskState::Completed;
                        self.tasks.remove(&task_id);
                    }
                    TaskResult::Yield => {
                        task.state = TaskState::Ready;
                        self.ready.push_back(task_id);
                    }
                    TaskResult::WaitIo { token, interest } => {
                        task.state = TaskState::Blocked { token, interest };
                        self.waiting_io.insert(token, task_id);
                    }
                    TaskResult::WaitTimer { token } => {
                        task.state = TaskState::Timer { token };
                        self.waiting_timer.insert(token, task_id);
                    }
                    TaskResult::Error(_msg) => {
                        task.state = TaskState::Completed;
                        self.tasks.remove(&task_id);
                    }
                }
            }
        }
    }

    /// Get a task by ID
    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Cancel a task
    pub fn cancel(&mut self, id: TaskId) -> bool {
        if let Some(task) = self.tasks.remove(&id) {
            // Remove from waiting queues
            if let TaskState::Blocked { token, .. } = task.state {
                self.waiting_io.remove(&token);
            }
            if let TaskState::Timer { token } = task.state {
                self.waiting_timer.remove(&token);
            }
            true
        } else {
            false
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_run() {
        let mut executor = Executor::new();
        let reactor = Reactor::new();

        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        executor.spawn_fn(move |_ctx| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            TaskResult::Done
        });

        executor.run_ready(&reactor);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert!(executor.is_empty());
    }

    #[test]
    fn test_yield_and_continue() {
        let mut executor = Executor::new();
        let reactor = Reactor::new();

        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        executor.spawn_fn(move |_ctx| {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                TaskResult::Yield
            } else {
                TaskResult::Done
            }
        });

        // Run until done
        for _ in 0..10 {
            if executor.is_empty() {
                break;
            }
            executor.run_ready(&reactor);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 3);
        assert!(executor.is_empty());
    }
}
