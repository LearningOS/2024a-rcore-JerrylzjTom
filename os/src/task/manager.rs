//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use crate::task::BIGSTRIDE;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let pid = task.pid.0;
        debug!("add Task pid: {}", pid);
        {
            let mut inner = task.inner_exclusive_access();
            inner.stride += inner.pass;
            // if inner.stride >= isize::MAX as usize {
            //     inner.stride = 0;
            // }
        }
        self.ready_queue.push_back(task);
        // self.ready_queue
        //     .make_contiguous()
        //     .sort_by(|a, b| a.inner_exclusive_access().stride.cmp(&b.inner_exclusive_access().stride));
        // debug!("manager add");
        // Sort by pass with overflow-safe comparison
        self.ready_queue.make_contiguous().sort_by(|a, b| {
            let a_pass = a.inner_exclusive_access().pass;
            let b_pass = b.inner_exclusive_access().pass;
            let diff = (a_pass - b_pass).abs();

            if diff > BIGSTRIDE {
                if a_pass < b_pass {
                    core::cmp::Ordering::Greater
                } else {
                    core::cmp::Ordering::Less
                }
            } else {
                a_pass.cmp(&b_pass)
            }
        });
        debug!("Task Queue: {:?}", self.ready_queue);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
