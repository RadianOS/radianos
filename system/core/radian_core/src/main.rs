mod cap;
mod ipc;
mod memory;
mod policy;
mod task;

use cap::*;
use ipc::*;
use policy::*;
use task::*;

fn main() {
    // Initialize memory region (1 KB)
    let mut mem = memory::MemoryRegion::new(1024);

    // Define capabilities including messaging and spawning
    let caps = CapSet::new(vec![
        Capability::WriteLog,
        Capability::SpawnTask,
        Capability::SendMessage,
        Capability::ReceiveMessage,
    ]);

    // Log startup message
    if let Err(e) = log_message(&caps, "Radian Core Starting...\n") {
        eprintln!("Log failed: {}", e);
    }

    // Initialize policy engine and add rules for task start and messaging
    let mut policy = PolicyEngine::new();
    policy.add_rule(PolicyRule {
        subject: "worker_1".into(),
        action: Action::StartTask,
        allow: true,
    });
    policy.add_rule(PolicyRule {
        subject: "worker_2".into(),
        action: Action::StartTask,
        allow: true,
    });
    policy.add_rule(PolicyRule {
        subject: "worker_1".into(),
        action: Action::SendMessageTo("worker_2".into()),
        allow: true,
    });
    policy.add_rule(PolicyRule {
        subject: "worker_2".into(),
        action: Action::ReceiveMessage,
        allow: true,
    });

    // Test memory write operation near bounds
    match mem.write(1000, &[1, 2, 3, 4]) {
        Ok(_) => println!("Memory write success."),
        Err(e) => println!("Memory write failed: {}", e),
    }

    // Create task manager and spawn two workers
    let mut tm = TaskManager::new();

    tm.spawn("worker_1", &caps, &policy)
        .expect("Failed to spawn worker_1");
    tm.spawn("worker_2", &caps, &policy)
        .expect("Failed to spawn worker_2");

    // Run next ready task
    if let Some(task) = tm.run_next() {
        println!("Running task: {} (id: {})", task.name, task.id);
    }

    // Send message from worker_1 to worker_2
    match tm.send_message(
        "worker_1",
        "worker_2",
        b"Hello from worker_1".to_vec(),
        &caps,
        &policy,
    ) {
        Ok(_) => println!("Message sent successfully!"),
        Err(e) => println!("Failed to send message: {}", e),
    }

    // Simulate worker_2 receiving the message
    if let Some(worker_2) = tm.tasks.iter_mut().find(|t| t.name == "worker_2") {
        if let Some(msg) = worker_2.receive_message() {
            println!(
                "worker_2 received message from {}: {}",
                msg.sender,
                String::from_utf8_lossy(&msg.payload)
            );
        } else {
            println!("worker_2 has no messages.");
        }
    }
}
