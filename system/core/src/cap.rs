#[repr(u8)]
#[derive(Default, Clone, Debug)]
pub enum Capability {
    ReadFilesystem = 0,
    WriteLog,
    SpawnTask,
    NetworkAccess,
}

