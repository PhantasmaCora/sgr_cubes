

#[repr(u8)]
pub enum ParamType {
    RotFace,
    RotEdge,
    RotVert,

}

pub enum Direction {
    PlusZ,
    MinusZ,
    PlusX,
    MinusX,
    PlusY,
    MinusY
}

pub struct Block<'a> {
    registry_id: u16,
    //shape: &BlockShape,
    //parameter_type: ParamType,
    pub texture: u32,
    pub pretty_name: &'a str,
}

pub struct BlockShape {

}

pub struct BlockRegistry<'a> {
    blocks: Box<Vec<Block<'a> >>,
}

impl<'a> BlockRegistry<'a> {
    pub fn new() -> BlockRegistry<'a> {
        // Always create the air block at position zero!
        let air = Block { registry_id: 0, pretty_name: &"Air", texture: 0 };
        let mut b = Vec::<Block>::new();
        b.push(air);

        let blocks = Box::new(b);

        Self {
            blocks
        }
    }

    pub fn add(&mut self, pretty_name: &'a str, texture: u32 ) -> u16 {
        let registry_id = self.blocks.len() as u16;
        self.blocks.push( Block { registry_id, pretty_name, texture } );
        registry_id
    }

    pub fn get(&self, index: u16) -> Option<&Block> {
        self.blocks.get(index as usize)
    }
}
