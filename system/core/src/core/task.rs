use super::cap::{CapSet, Capability};
use super::ipc::{Message, MessageQueue};
use super::policy::{Action, PolicyEngine};
use super::scheduler::{RoundRobinScheduler, Scheduler};

#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub state: TaskState,
    pub msg_queue: MessageQueue,
}

impl Task {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            state: TaskState::Ready,
            msg_queue: MessageQueue::new(10),
        }
    }

    pub fn send_message(&mut self, msg: Message) -> Result<(), &'static str> {
        self.msg_queue.enqueue(msg)
    }

    pub fn receive_message(&mut self) -> Option<Message> {
        self.msg_queue.dequeue()
    }
}

pub struct TaskManager {
    next_id: usize,
    pub tasks: Vec<Task>,
    scheduler: Box<dyn Scheduler>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            tasks: Vec::new(),
            scheduler: Box::new(RoundRobinScheduler::new()),
        }
    }

    pub fn spawn(
        &mut self,
        name: &str,
        caps: &CapSet,
        policy: &PolicyEngine,
    ) -> Result<&Task, &'static str> {
        if !caps.has(&Capability::SpawnTask) {
            return Err("Insufficient capabilities to spawn task");
        }

        if !policy.check(name, &Action::StartTask) {
            return Err("Policy denies spawning this task");
        }

        let task = Task::new(self.next_id, name.to_string());
        self.tasks.push(task);
        self.next_id += 1;

        Ok(self.tasks.last().unwrap())
    }

    pub fn run_next(&mut self) -> Option<&Task> {
        let selected = self.scheduler.select(&mut self.tasks)?;
        selected.state = TaskState::Running;
        Some(selected)
    }

    pub fn send_message(
        &mut self,
        sender: &str,
        receiver: &str,
        payload: Vec<u8>,
        caps: &CapSet,
        policy: &PolicyEngine,
    ) -> Result<(), &'static str> {
        if !caps.has(&Capability::SendMessage) {
            return Err("Missing SendMessage capability");
        }

        if !policy.check(sender, &Action::SendMessageTo(receiver.to_string())) {
            return Err("Policy denies sending message");
        }

        let sender_exists = self.tasks.iter().any(|t| t.name == sender);
        if !sender_exists {
            return Err("Sender task not found");
        }

        if let Some(receiver_task) = self.tasks.iter_mut().find(|t| t.name == receiver) {
            let msg = Message {
                sender: sender.to_string(),
                payload,
            };
            receiver_task.send_message(msg)
        } else {
            Err("Receiver task not found")
        }
    }
}
