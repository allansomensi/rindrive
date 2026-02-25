use clap::ValueEnum;

pub mod fullscan;
pub mod spotcheck;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum EngineType {
    SpotCheck,
    FullScan,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone)]
#[repr(C, align(4096))]
struct AlignedBlock([u8; 4096]);

pub struct AlignedBuffer {
    blocks: Vec<AlignedBlock>,
    size: usize,
}

impl AlignedBuffer {
    pub fn new(size: usize) -> Self {
        let num_blocks = (size + 4095).div_ceil(4096);
        Self {
            blocks: vec![AlignedBlock([0; 4096]); num_blocks],
            size,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.blocks.as_mut_ptr() as *mut u8, self.size) }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.blocks.as_ptr() as *const u8, self.size) }
    }
}
