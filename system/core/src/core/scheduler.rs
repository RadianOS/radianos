use crate::core::task::{Task, TaskState};

pub trait Scheduler {
    fn select<'a>(&self, tasks: &'a mut Vec<Task>) -> Option<&'a mut Task>; // thank you andreashgk
    // :333
}

pub struct RoundRobinScheduler;

impl RoundRobinScheduler {
    pub fn new() -> Self {
        RoundRobinScheduler
    }
}

impl Scheduler for RoundRobinScheduler {
    fn select<'a>(&self, tasks: &'a mut Vec<Task>) -> Option<&'a mut Task> {
        for task in tasks.iter_mut() {
            if let TaskState::Ready = task.state {
                return Some(task as &mut Task);
            }
        }
        None
    }
}
