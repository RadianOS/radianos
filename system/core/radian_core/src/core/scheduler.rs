use super::task::{Task, TaskState};

pub trait Scheduler {
    fn select<'a>(&mut self, tasks: &'a mut [Task]) -> Option<&'a mut Task>;
}

pub struct RoundRobinScheduler {
    index: usize,
}

impl RoundRobinScheduler {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl Scheduler for RoundRobinScheduler {
    fn select<'a>(&mut self, tasks: &'a mut [Task]) -> Option<&'a mut Task> {
        let len = tasks.len();
        for _ in 0..len {
            let idx = self.index % len;
            self.index = (self.index + 1) % len;

            if tasks[idx].state == TaskState::Ready {
                return Some(&mut tasks[idx]);
            }
        }
        None
    }
}
