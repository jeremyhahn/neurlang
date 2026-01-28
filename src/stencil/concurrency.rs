//! Concurrency stencils for spawn, channels, and atomics
//!
//! Implements lightweight task spawning, message-passing channels,
//! and atomic operations.

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Task identifier
pub type TaskId = u64;

/// Channel identifier
pub type ChannelId = u64;

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run
    Ready,
    /// Task is currently running
    Running,
    /// Task is waiting on a channel
    Blocked,
    /// Task has completed
    Completed,
    /// Task has failed
    Failed,
}

/// A lightweight task
pub struct Task {
    pub id: TaskId,
    pub state: AtomicU64, // TaskState encoded
    pub result: Mutex<Option<u64>>,
    handle: Mutex<Option<JoinHandle<u64>>>,
}

impl Task {
    fn new(id: TaskId) -> Self {
        Self {
            id,
            state: AtomicU64::new(TaskState::Ready as u64),
            result: Mutex::new(None),
            handle: Mutex::new(None),
        }
    }

    pub fn get_state(&self) -> TaskState {
        match self.state.load(Ordering::Acquire) {
            0 => TaskState::Ready,
            1 => TaskState::Running,
            2 => TaskState::Blocked,
            3 => TaskState::Completed,
            _ => TaskState::Failed,
        }
    }

    fn set_state(&self, state: TaskState) {
        self.state.store(state as u64, Ordering::Release);
    }
}

/// Channel for inter-task communication
pub struct Channel {
    pub id: ChannelId,
    sender: Sender<u64>,
    receiver: Receiver<u64>,
    closed: AtomicU64,
}

impl Channel {
    fn new(id: ChannelId, capacity: Option<usize>) -> Self {
        let (sender, receiver) = match capacity {
            Some(cap) => bounded(cap),
            None => unbounded(),
        };

        Self {
            id,
            sender,
            receiver,
            closed: AtomicU64::new(0),
        }
    }

    pub fn send(&self, value: u64) -> Result<(), ChannelError> {
        if self.closed.load(Ordering::Acquire) != 0 {
            return Err(ChannelError::Closed);
        }
        self.sender.send(value).map_err(|_| ChannelError::Closed)
    }

    pub fn recv(&self) -> Result<u64, ChannelError> {
        self.receiver.recv().map_err(|_| ChannelError::Closed)
    }

    pub fn try_recv(&self) -> Result<u64, ChannelError> {
        self.receiver.try_recv().map_err(|_| ChannelError::Empty)
    }

    pub fn close(&self) {
        self.closed.store(1, Ordering::Release);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelError {
    Closed,
    Empty,
    Full,
}

/// Concurrency runtime managing tasks and channels
pub struct ConcurrencyRuntime {
    /// Next task ID
    next_task_id: AtomicU64,
    /// Next channel ID
    next_channel_id: AtomicU64,
    /// Active tasks
    tasks: RwLock<HashMap<TaskId, Arc<Task>>>,
    /// Active channels
    channels: RwLock<HashMap<ChannelId, Arc<Channel>>>,
    /// Maximum concurrent tasks
    max_tasks: usize,
    /// Current task count
    active_task_count: AtomicUsize,
}

impl ConcurrencyRuntime {
    pub fn new(max_tasks: usize) -> Self {
        Self {
            next_task_id: AtomicU64::new(1),
            next_channel_id: AtomicU64::new(1),
            tasks: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
            max_tasks,
            active_task_count: AtomicUsize::new(0),
        }
    }

    /// Spawn a new task
    pub fn spawn<F>(&self, f: F) -> Result<TaskId, ConcurrencyError>
    where
        F: FnOnce() -> u64 + Send + 'static,
    {
        // Check task limit
        let current = self.active_task_count.load(Ordering::Acquire);
        if current >= self.max_tasks {
            return Err(ConcurrencyError::TooManyTasks);
        }

        let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        let task = Arc::new(Task::new(task_id));

        // Insert task
        {
            let mut tasks = self.tasks.write();
            tasks.insert(task_id, Arc::clone(&task));
        }

        // Spawn the actual thread
        let task_clone = Arc::clone(&task);
        let handle = thread::spawn(move || {
            task_clone.set_state(TaskState::Running);
            let result = f();
            *task_clone.result.lock() = Some(result);
            task_clone.set_state(TaskState::Completed);
            result
        });

        *task.handle.lock() = Some(handle);
        self.active_task_count.fetch_add(1, Ordering::Release);

        Ok(task_id)
    }

    /// Wait for a task to complete and get its result
    pub fn join(&self, task_id: TaskId) -> Result<u64, ConcurrencyError> {
        let task = {
            let tasks = self.tasks.read();
            tasks.get(&task_id).cloned()
        };

        let task = task.ok_or(ConcurrencyError::TaskNotFound)?;

        // Take the handle and join
        let handle = task.handle.lock().take();
        if let Some(h) = handle {
            let result = h.join().map_err(|_| ConcurrencyError::TaskPanicked)?;

            // Clean up
            self.active_task_count.fetch_sub(1, Ordering::Release);
            {
                let mut tasks = self.tasks.write();
                tasks.remove(&task_id);
            }

            Ok(result)
        } else {
            // Already joined or never started
            task.result.lock().ok_or(ConcurrencyError::TaskNotStarted)
        }
    }

    /// Check if a task is complete
    pub fn is_complete(&self, task_id: TaskId) -> bool {
        let tasks = self.tasks.read();
        tasks
            .get(&task_id)
            .map(|t| t.get_state() == TaskState::Completed)
            .unwrap_or(true)
    }

    /// Create a new channel
    pub fn create_channel(&self, capacity: Option<usize>) -> ChannelId {
        let chan_id = self.next_channel_id.fetch_add(1, Ordering::Relaxed);
        let channel = Arc::new(Channel::new(chan_id, capacity));

        let mut channels = self.channels.write();
        channels.insert(chan_id, channel);

        chan_id
    }

    /// Get a channel by ID
    pub fn get_channel(&self, chan_id: ChannelId) -> Option<Arc<Channel>> {
        let channels = self.channels.read();
        channels.get(&chan_id).cloned()
    }

    /// Send a value on a channel
    pub fn send(&self, chan_id: ChannelId, value: u64) -> Result<(), ChannelError> {
        let channel = self.get_channel(chan_id).ok_or(ChannelError::Closed)?;
        channel.send(value)
    }

    /// Receive a value from a channel
    pub fn recv(&self, chan_id: ChannelId) -> Result<u64, ChannelError> {
        let channel = self.get_channel(chan_id).ok_or(ChannelError::Closed)?;
        channel.recv()
    }

    /// Close a channel
    pub fn close_channel(&self, chan_id: ChannelId) {
        if let Some(channel) = self.get_channel(chan_id) {
            channel.close();
        }
        // Optionally remove from map
        let mut channels = self.channels.write();
        channels.remove(&chan_id);
    }

    /// Yield the current task (cooperative multitasking hint)
    pub fn yield_now(&self) {
        thread::yield_now();
    }
}

impl Default for ConcurrencyRuntime {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyError {
    TooManyTasks,
    TaskNotFound,
    TaskPanicked,
    TaskNotStarted,
    ChannelClosed,
}

/// Atomic operations for lock-free programming
pub mod atomics {
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Compare-and-swap operation
    #[inline]
    pub fn cas(ptr: &AtomicU64, expected: u64, new: u64) -> (bool, u64) {
        match ptr.compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(prev) => (true, prev),
            Err(prev) => (false, prev),
        }
    }

    /// Atomic exchange
    #[inline]
    pub fn xchg(ptr: &AtomicU64, new: u64) -> u64 {
        ptr.swap(new, Ordering::SeqCst)
    }

    /// Atomic add, returns old value
    #[inline]
    pub fn add(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_add(val, Ordering::SeqCst)
    }

    /// Atomic sub, returns old value
    #[inline]
    pub fn sub(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_sub(val, Ordering::SeqCst)
    }

    /// Atomic and, returns old value
    #[inline]
    pub fn and(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_and(val, Ordering::SeqCst)
    }

    /// Atomic or, returns old value
    #[inline]
    pub fn or(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_or(val, Ordering::SeqCst)
    }

    /// Atomic xor, returns old value
    #[inline]
    pub fn xor(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_xor(val, Ordering::SeqCst)
    }

    /// Atomic min (unsigned), returns old value
    #[inline]
    pub fn min(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_min(val, Ordering::SeqCst)
    }

    /// Atomic max (unsigned), returns old value
    #[inline]
    pub fn max(ptr: &AtomicU64, val: u64) -> u64 {
        ptr.fetch_max(val, Ordering::SeqCst)
    }

    /// Memory fence - acquire semantics
    #[inline]
    pub fn fence_acquire() {
        std::sync::atomic::fence(Ordering::Acquire);
    }

    /// Memory fence - release semantics
    #[inline]
    pub fn fence_release() {
        std::sync::atomic::fence(Ordering::Release);
    }

    /// Memory fence - acquire-release semantics
    #[inline]
    pub fn fence_acq_rel() {
        std::sync::atomic::fence(Ordering::AcqRel);
    }

    /// Memory fence - sequentially consistent
    #[inline]
    pub fn fence_seq_cst() {
        std::sync::atomic::fence(Ordering::SeqCst);
    }
}

/// FFI functions for calling from generated code
mod ffi {
    use super::*;

    static RUNTIME: once_cell::sync::Lazy<ConcurrencyRuntime> =
        once_cell::sync::Lazy::new(|| ConcurrencyRuntime::new(1024));

    /// Spawn a task (simplified - takes function pointer)
    #[no_mangle]
    pub extern "C" fn neurlang_spawn(entry: extern "C" fn() -> u64) -> u64 {
        RUNTIME.spawn(move || entry()).unwrap_or_default()
    }

    /// Join a task
    #[no_mangle]
    pub extern "C" fn neurlang_join(task_id: u64) -> u64 {
        RUNTIME.join(task_id).unwrap_or(0)
    }

    /// Create a channel
    #[no_mangle]
    pub extern "C" fn neurlang_chan_create(capacity: u64) -> u64 {
        let cap = if capacity == 0 {
            None
        } else {
            Some(capacity as usize)
        };
        RUNTIME.create_channel(cap)
    }

    /// Send on a channel
    #[no_mangle]
    pub extern "C" fn neurlang_chan_send(chan_id: u64, value: u64) -> u64 {
        match RUNTIME.send(chan_id, value) {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }

    /// Receive from a channel
    #[no_mangle]
    pub extern "C" fn neurlang_chan_recv(chan_id: u64, out_value: *mut u64) -> u64 {
        match RUNTIME.recv(chan_id) {
            Ok(val) => {
                if !out_value.is_null() {
                    unsafe { *out_value = val };
                }
                0
            }
            Err(_) => 1,
        }
    }

    /// Close a channel
    #[no_mangle]
    pub extern "C" fn neurlang_chan_close(chan_id: u64) {
        RUNTIME.close_channel(chan_id);
    }

    /// Yield
    #[no_mangle]
    pub extern "C" fn neurlang_yield() {
        RUNTIME.yield_now();
    }

    /// Memory fences
    #[no_mangle]
    pub extern "C" fn neurlang_fence(mode: u64) {
        match mode {
            0 => atomics::fence_acquire(),
            1 => atomics::fence_release(),
            2 => atomics::fence_acq_rel(),
            _ => atomics::fence_seq_cst(),
        }
    }
}

// Re-export FFI functions
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_join() {
        let rt = ConcurrencyRuntime::new(10);

        let task_id = rt.spawn(|| 42).unwrap();
        let result = rt.join(task_id).unwrap();

        assert_eq!(result, 42);
    }

    #[test]
    fn test_channel() {
        let rt = ConcurrencyRuntime::new(10);

        let chan_id = rt.create_channel(Some(10));

        // Sender task
        let chan = rt.get_channel(chan_id).unwrap();
        chan.send(123).unwrap();

        // Receiver
        let val = rt.recv(chan_id).unwrap();
        assert_eq!(val, 123);
    }

    #[test]
    fn test_atomics() {
        let val = AtomicU64::new(10);

        let old = atomics::add(&val, 5);
        assert_eq!(old, 10);
        assert_eq!(val.load(Ordering::SeqCst), 15);

        let (success, old) = atomics::cas(&val, 15, 20);
        assert!(success);
        assert_eq!(old, 15);
        assert_eq!(val.load(Ordering::SeqCst), 20);
    }

    #[test]
    fn test_multiple_tasks() {
        let rt = ConcurrencyRuntime::new(100);

        let tasks: Vec<_> = (0..10).map(|i| rt.spawn(move || i * 2).unwrap()).collect();

        let results: Vec<_> = tasks.iter().map(|&id| rt.join(id).unwrap()).collect();

        let expected: Vec<u64> = (0..10).map(|i| i * 2).collect();
        assert_eq!(results, expected);
    }
}
