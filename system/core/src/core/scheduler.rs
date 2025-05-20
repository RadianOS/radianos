use crate::core::task::{Task, TaskState};

pub trait Scheduler {
    fn select(&self, tasks: &mut Vec<Task>) -> Option<*mut Task>;
}

pub struct RoundRobinScheduler;

impl RoundRobinScheduler {
    pub fn new() -> Self {
        RoundRobinScheduler
    }
}

impl Scheduler for RoundRobinScheduler {
    fn select(&self, tasks: &mut Vec<Task>) -> Option<*mut Task> {
        for task in tasks.iter_mut() {
            if let TaskState::Ready = task.state {
                return Some(task as *mut Task);
            }
        }
        None
    }
}
