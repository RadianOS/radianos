use crate::{db, dense_bitfield};

// Define system capabilities that can be granted to components or processes.
dense_bitfield!(
    Capability u16
    READ_FILESYSTEM = 0x01,
    WRITE_LOG = 0x02,
    SPAWN_TASK = 0x04,
    NETWORK_ACCESS = 0x08,
);
impl Capability {
    pub fn new() -> Self {
        Self(0)
    }
}
