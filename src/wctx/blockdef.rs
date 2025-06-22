 
use serde::{
    Serialize,
    Deserialize
};

#[repr(C)]
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub blockmesh: u32,
    pub exparam: u32
}

impl BlockDef {
    pub fn get_rotation(&self) -> u8 {
        ( self.exparam & 31 ) as u8
    }

    pub fn set_rotation(&mut self, rot: u8) {
        self.exparam = (self.exparam & !31) + rot as u32;
    }

    pub fn get_palette_index(&self) -> u32 {
        (self.exparam & !31) >> 5
    }

    pub fn set_palette_index(&mut self, idx: u32) {
        self.exparam = (self.exparam & 31) + (idx << 5);
    }
}
