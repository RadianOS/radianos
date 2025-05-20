use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Capability {
    ReadFilesystem,
    WriteLog,
    SpawnTask,
    NetworkAccess,
    SendMessage,
    ReceiveMessage,
}

pub struct CapSet {
    allowed: Vec<Capability>,
}

impl CapSet {
    pub fn new(allowed: Vec<Capability>) -> Self {
        Self { allowed }
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.allowed.contains(cap)
    }
}

pub fn log_message(capset: &CapSet, msg: &str) -> Result<(), &'static str> {
    if !capset.has(&Capability::WriteLog) {
        return Err("Permission denied: WriteLog");
    }

    let mut path = dirs::home_dir().ok_or("Failed to get home directory")?;
    path.push("radianos/mutable/logs");

    create_dir_all(&path).map_err(|_| "Failed to create log directory")?;

    path.push("radian_core.log");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|_| "IO error")?;

    file.write_all(msg.as_bytes()).map_err(|_| "IO error")
}
