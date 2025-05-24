use super::task::{Task, TaskManager, TaskState};
use crate::core::scheduler::Scheduler;
use crate::core::task;

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

        let mut tick: u32 = 0;

        loop {
            tick += 1;
            println!("\nTick {}", tick);

            for task in self.task_manager.tasks.iter_mut() {
                if let TaskState::Sleeping(ref mut ticks) = task.state {
                    if *ticks > 0 {
                        *ticks -= 1;
                        if *ticks == 0 {
                            task.state    pub fn run(&mut self) {
        println!("--- Radian Core Runtime Starting ---");

        let mut tick: u32 = 0;

        loop {
            tick += 1;
            println!("\nTick {}", tick);

            // Handle sleeping tasks
            for task in self.task_manager.tasks.iter_mut() {
                if let TaskState::Sleeping(ref mut ticks) = task.state {
                    if *ticks > 0 {
                        *ticks -= 1;
                        if *ticks == 0 {
                            task.state = TaskState::Ready;
                            println!("↻ Waking up task {}", task.name);
                        }
                    }
                }
            }

            // Select a task — scope ends here, so borrow ends
            let selected_task_ptr = self.scheduler.select(&mut self.task_manager.tasks);

            // SAFETY: We trust that select() returns a valid mutable pointer from the same vec
            let task = unsafe { selected_task_ptr.map(|ptr| &mut *ptr) };

            match task {
                Some(task) => {
                    println!("→ Running task: {} (id: {})", task.name, task.id);
                    task.state = TaskState::Running;
                    self.simulate_task(task);

                    if let TaskState::Terminated = task.state {
                        println!("✖ Task {} has terminated.", task.name);
                    }
                }
                None => {
                    println!("⏸ No ready tasks. System idle.");
                    break;
                }
            }
        }

        println!("\n--- Radian Core Runtime Halted ---");
    } = TaskState::Ready;
                            println!("↻ Waking up task {}", task.name);
                        }
                    }
                }
            }

            let task_ptr = {
                let tasks = &mut self.task_manager.tasks;
                self.scheduler.select(tasks)
            };

            if let Some(ptr) = task_ptr {
                let task = unsafe { &mut *ptr };
                println!("→ Running task: {} (id: {})", task.name, task.id);
                task.state = TaskState::Running;
                self.simulate_task(task);
                if let TaskState::Terminated = task.state {
                    println!("✖ Task {} has terminated.", task.name);
                }
            } else {
                println!("⏸ No ready tasks. System idle.");
                break;
            }
        }

        println!("\n--- Radian Core Runtime Halted ---");
    }

    fn simulate_task(&mut self, task: &mut task::Task) {
        println!("...Task {} is doing work...", task.name);
        if task.name.contains("1") {
            println!("↺ Task {} is sleeping for 2 ticks", task.name);
            task.state = TaskState::Sleeping(2);
        } else {
            println!("✔ Task {} completed", task.name);
            task.state = TaskState::Terminated;
        }
    }
}
