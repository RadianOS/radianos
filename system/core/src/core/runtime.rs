use super::scheduler::Scheduler;
use super::task::{Task, TaskManager, TaskState};

pub struct Runtime {
    pub scheduler: Box<dyn Scheduler>,
    pub task_manager: TaskManager,
}

impl Runtime {
    pub fn new(scheduler: Box<dyn Scheduler>, task_manager: TaskManager) -> Self {
        Self {
            scheduler,
            task_manager,
        }
    }

    pub fn run(&mut self) {
        println!("--- Radian Core Runtime Starting ---");

        loop {
            let task_id = {
                let tasks = &mut self.task_manager.tasks;
                self.scheduler.select(tasks).map(|task| task.id)
            };

            let maybe_task = match task_id {
                Some(id) => {
                    let task_ptr: *mut Task = self
                        .task_manager
                        .tasks
                        .iter_mut()
                        .find(|t| t.id == id)
                        .map(|t| t as *mut Task)
                        .unwrap();
                    Some(unsafe { &mut *task_ptr })
                }
                None => None,
            };

            match maybe_task {
                Some(task) => {
                    println!("→ Running task: {} (id: {})", task.name, task.id);
                    self.simulate_task(task);
                    if task.state == TaskState::Terminated {
                        println!("✖ Task {} has terminated.", task.name);
                    }
                }
                None => {
                    println!("⏸ No ready tasks. System idle.");
                    break;
                }
            }
        }

        println!("--- Radian Core Runtime Halted ---");
    }

    fn simulate_task(&mut self, task: &mut Task) {
        println!("...Task {} is doing work...", task.name);
        task.state = TaskState::Ready;
    }
}
