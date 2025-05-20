use crate::core::memory::MemoryRegion;
use core::runtime::Runtime;
use core::scheduler::RoundRobinScheduler;
mod core;

use core::cap::*;
use core::ipc::*;
use core::memory::*;
use core::policy::*;
use core::runtime::*;
use core::scheduler::*;
use core::task::*;

fn main() {
    let mut mem = MemoryRegion::new(1024);

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

    // Initialize policy engine and add rules
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

    // Test the memory write operation near bounds
    match mem.write(1000, &[1, 2, 3, 4]) {
        Ok(_) => println!("Memory write success."),
        Err(e) => println!("Memory write failed: {}", e),
    }

    let mut tm = TaskManager::new();

    tm.spawn("worker_1", &caps, &policy)
        .expect("Failed to spawn worker_1");
    tm.spawn("worker_2", &caps, &policy)
        .expect("Failed to spawn worker_2");

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

    // runtime loop
    let scheduler = Box::new(RoundRobinScheduler::new());
    let mut runtime = Runtime::new(scheduler, tm);
    runtime.run();
}
