pub struct MemoryRegion {
    data: Vec<u8>,
    limit: usize,
}

impl MemoryRegion {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            limit: size,
        }
    }

    pub fn write(&mut self, offset: usize, input: &[u8]) -> Result<(), &'static str> {
        if offset + input.len() > self.limit {
            return Err("Memory access out of bounds");
        }
        self.data[offset..offset + input.len()].copy_from_slice(input);
        Ok(())
    }

    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], &'static str> {
        if offset + len > self.limit {
            return Err("Memory access out of bounds");
        }
        Ok(&self.data[offset..offset + len])
    }
}
