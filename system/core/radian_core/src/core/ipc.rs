use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub payload: Vec<u8>,
}
#[derive(Debug)]
pub struct MessageQueue {
    queue: VecDeque<Message>,
    limit: usize, // max messages in queue
}

impl MessageQueue {
    pub fn new(limit: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            limit,
        }
    }

    pub fn enqueue(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.queue.len() >= self.limit {
            return Err("Message queue full");
        }
        self.queue.push_back(msg);
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<Message> {
        self.queue.pop_front()
    }
}
