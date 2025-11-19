use anyhow::{Result, bail};

/// The different sizes used for memory accesses
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum MemAccessSize {
    /// 8 bits
    Byte = 1,
    /// 16 bits
    HalfWord = 2,
    /// 32 bits
    Word = 4,
}

pub struct Memory {
    pub vec: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        assert_eq!(size % 4, 0);

        Self { vec: vec![0; size] }
    }

    pub fn read(&self, addr: u32, osize: MemAccessSize) -> Result<u32> {
        let addr = addr as usize;
        let size = osize as usize;

        // if addr % size != 0 {
        //     bail!("[memory] read with invalid alignment, address: {addr:#x}, size: {size}");
        // }

        if addr + size > self.vec.len() {
            bail!("[memory] read is out of range, address: {addr:#x}, size: {size}");
        }

        Ok(match osize {
            MemAccessSize::Byte => self.vec[addr].into(),
            MemAccessSize::HalfWord => {
                u16::from_le_bytes(self.vec[addr..addr + size].try_into().unwrap()).into()
            }
            MemAccessSize::Word => {
                u32::from_le_bytes(self.vec[addr..addr + size].try_into().unwrap())
            }
        })
    }

    pub fn write(&mut self, addr: u32, osize: MemAccessSize, data: u32) -> Result<()> {
        let addr = addr as usize;
        let size = osize as usize;

        // if addr % size != 0 {
        //     bail!("[memory] write with invalid alignment, address: {addr:#x}, size: {size}");
        // }

        if addr + size > self.vec.len() {
            bail!("[memory] write is out of range, address: {addr:#x}, size: {size}");
        }

        let data = data.to_le_bytes();
        self.vec[addr..addr + size].copy_from_slice(&data[..size]);

        Ok(())
    }

    pub fn flash(&mut self, data: &[u8]) {
        self.vec[..data.len()].copy_from_slice(data);
    }
}
