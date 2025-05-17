# `radian_core`: Minimal RadianOS-Compliant Runtime Framework (Rust)

This document defines a minimal runtime framework in Rust that complies with the RadianOS philosophy and directory structure. It includes memory safety, capability-based security, and embedded policy layers.

---

## Directory Layout (Radian-Compliant)

```
/system/
├── core/
│   ├── radian_core/
│   │   ├── main.rs
│   │   ├── cap.rs
│   │   ├── memory.rs
│   │   └── policy.rs
/mutable/logs/radian_core.log
/system/run/radian_core.sock
```

---

## 1. Memory Safety (`memory.rs`)

```rust
// Represents a safe, bounded memory region.
pub struct MemoryRegion {
    data: Vec<u8>,   // Internal byte storage
    limit: usize,    // Max allowable size (bounds checking)
}

impl MemoryRegion {
    // Constructs a new MemoryRegion of a given size, zero-filled.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            limit: size,
        }
    }

    // Attempts to write `input` bytes into the memory at `offset`.
    // Returns error if the write would exceed memory bounds.
    pub fn write(&mut self, offset: usize, input: &[u8]) -> Result<(), &'static str> {
        if offset + input.len() > self.limit {
            return Err("Memory access out of bounds");
        }
        self.data[offset..offset + input.len()].copy_from_slice(input);
        Ok(())
    }

    // Reads `len` bytes from memory starting at `offset`.
    // Returns error if the read would exceed bounds.
    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], &'static str> {
        if offset + len > self.limit {
            return Err("Memory access out of bounds");
        }
        Ok(&self.data[offset..offset + len])
    }
}
```

---

## 2. Capability-Based Security (`cap.rs`)

```rust
// Define system capabilities that can be granted to components or processes.
#[derive(Clone, Debug)]
pub enum Capability {
    ReadFilesystem,
    WriteLog,
    SpawnTask,
    NetworkAccess,
}

// Holds a set of allowed capabilities.
pub struct CapSet {
    allowed: Vec<Capability>, // Permissions granted to a process or module
}

impl CapSet {
    // Create a new CapSet with an initial list of capabilities.
    pub fn new(allowed: Vec<Capability>) -> Self {
        Self { allowed }
    }

    // Checks whether the CapSet includes a particular capability.
    pub fn has(&self, cap: &Capability) -> bool {
        self.allowed.contains(cap)
    }
}

// Logs a message if the provided CapSet has WriteLog permission.
pub fn log_message(capset: &CapSet, msg: &str) -> Result<(), &'static str> {
    if !capset.has(&Capability::WriteLog) {
        return Err("Permission denied: WriteLog");
    }
    std::fs::write("/mutable/logs/radian_core.log", msg)
        .map_err(|_| "IO error")
}
```

---

## 3. Embedded Policy Layer (`policy.rs`)

```rust
// Define actions that subjects might want to perform.
#[derive(Debug)]
pub enum Action {
    StartTask,
    AccessDevice(String), // Accessing a named device
    WriteTo(String),      // Writing to a specific target
}

// Defines a rule mapping subject to allowed/denied action.
#[derive(Debug)]
pub struct PolicyRule {
    subject: String,  // Actor or process name
    action: Action,   // Attempted action
    allow: bool,      // Whether the action is permitted
}

// Policy engine that holds all rules and evaluates them.
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    // Constructs a new engine with no initial rules.
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    // Adds a rule to the policy engine.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    // Evaluates whether a subject can perform a specific action.
    // Defaults to deny if no explicit rule matches.
    pub fn check(&self, subject: &str, action: &Action) -> bool {
        for rule in &self.rules {
            if &rule.subject == subject && &rule.action == action {
                return rule.allow;
            }
        }
        false // default deny
    }
}
```

---

## Example `main.rs` Execution

```rust
// Load internal modules
mod memory;
mod cap;
mod policy;

use cap::*;
use policy::*;

fn main() {
    // Initialize a 1KB memory region for safe access
    let mut mem = memory::MemoryRegion::new(1024);

    // Create a CapSet allowing write access to logs
    let caps = CapSet::new(vec![Capability::WriteLog]);

    // Attempt to log the runtime start message
    log_message(&caps, "Starting Radian Core Runtime...\n").unwrap();

    // Initialize policy engine and add a rule allowing 'worker_1' to start a task
    let mut policy = PolicyEngine::new();
    policy.add_rule(PolicyRule {
        subject: "worker_1".into(),
        action: Action::StartTask,
        allow: true,
    });

    // Perform a policy check for worker_1 starting a task
    let can_start = policy.check("worker_1", &Action::StartTask);
    println!("Policy check: Can start task? {}", can_start);

    // Try writing beyond the valid memory limit (intentionally close to bounds)
    let result = mem.write(1000, &[1, 2, 3, 4]);
    println!("Memory write result: {:?}", result);
}
```

---

## radian.yml

```yaml
name: "radian_core"
version: "0.1"
radian_version: "1.0"
kernel_entry: "system/core/radian_core/main.rs"
api_version: "1.0"
structure:
  - /system/core/radian_core
  - /mutable/logs
features:
  driver_model: unified
  ipc: message-based
  oskit_bootstrap: false
custom:
  runtime: "cap-runtime"
  security: "capabilities + embedded policy"
```
