
use std::cmp::min;




pub enum Direction {
    PlusY,
    MinusY,
    PlusX,
    MinusX,
    PlusZ,
    MinusZ
}

pub fn dir_to_vector (dir: &Direction) -> cgmath::Vector3<i32> {
    match dir {
        Direction::PlusZ => cgmath::Vector3::<i32>::unit_z(),
        Direction::MinusZ => -1 * cgmath::Vector3::<i32>::unit_z(),
        Direction::PlusX => cgmath::Vector3::<i32>::unit_x(),
        Direction::MinusX => -1 * cgmath::Vector3::<i32>::unit_x(),
        Direction::PlusY => cgmath::Vector3::<i32>::unit_y(),
        Direction::MinusY => -1 * cgmath::Vector3::<i32>::unit_y()
    }
}

pub fn reverse_dir (dir: &Direction) -> Direction {
    match dir {
        Direction::PlusZ => Direction::MinusZ,
        Direction::MinusZ => Direction::PlusZ,
        Direction::PlusX => Direction::MinusX,
        Direction::MinusX => Direction::PlusX,
        Direction::PlusY => Direction::MinusY,
        Direction::MinusY => Direction::PlusY
    }
}

pub struct Block<'a> {
    registry_id: u16,
    pub shape_id: usize,
    //parameter_type: ParamType,
    pub textures: Box<[u32]>,
    pub pretty_name: &'a str,
}

pub struct BlockRegistry<'a> {
    blocks: Box<Vec<Block<'a> >>,
}

impl<'a> BlockRegistry<'a> {
    pub fn new() -> BlockRegistry<'a> {
        // Always create the air block at position zero!
        let air = Block { registry_id: 0, shape_id: 0, pretty_name: &"Air", textures: Box::new([0]) };
        let mut b = Vec::<Block>::new();
        b.push(air);

        let blocks = Box::new(b);

        Self {
            blocks
        }
    }

    pub fn add(&mut self, shape_id: usize, pretty_name: &'a str, textures: Box<[u32]> ) -> u16 {
        let registry_id = self.blocks.len() as u16;
        self.blocks.push( Block { registry_id, shape_id, pretty_name, textures } );
        registry_id
    }

    pub fn get(&self, index: u16) -> Option<&Block> {
        self.blocks.get(index as usize)
    }
}

pub struct BlockShape {
    faces: Box<[FaceDef]>,
    obstructs: [bool; 6]
}

impl BlockShape {
    pub fn generate_draw_buffers(&self, vertex_buffer: &mut Vec<super::Vertex>, index_buffer: &mut Vec<u16>, blockdef: &Block, exparam: u8, chunk_view: &ndarray::Array3<super::chunk::BlockInstance>, pos: (usize, usize, usize) ) {
        for fi in self.faces.iter().enumerate() {
            let (f, face) = fi;

            // check whether there's an obstruction here
            if let Some(block_dir) = &face.obstructed_by {
                let mut other_pos = cgmath::Vector3::<i32>::new( pos.0 as i32, pos.1 as i32, pos.2 as i32 );
                other_pos += dir_to_vector(block_dir);
                if other_pos.x >= 0 && other_pos.x < chunk_view.len_of(ndarray::Axis(0)) as i32 && other_pos.y >= 0 && other_pos.y < chunk_view.len_of(ndarray::Axis(1)) as i32 && other_pos.z >= 0 && other_pos.z < chunk_view.len_of(ndarray::Axis(2)) as i32 {
                    let b2 = &chunk_view[ [other_pos.x as usize, other_pos.y as usize, other_pos.z as usize] ];
                    if b2.blockdef != 0 {
                        continue;
                    }
                }
            }

            let mut temp_indices = Vec::<u32>::new();
            let center = cgmath::Vector3::<f32>::new( pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5 );
            for vertdef in face.vertices.iter() {
                temp_indices.push( vertex_buffer.len().try_into().unwrap() );
                let tex_index = blockdef.textures[ min( f, blockdef.textures.len() - 1 ) ];
                vertex_buffer.push( super::Vertex::new( [ center.x + vertdef[0], center.y + vertdef[1], center.z + vertdef[2] ], [vertdef[3], vertdef[4]], tex_index, 1.0) );
            }

            for ind in face.indices.into_iter() {
                index_buffer.push( temp_indices[ *ind as usize ] as u16 );
            }
        }

    }

}

pub struct FaceDef {
    pub obstructed_by: Option<Direction>,
    pub vertices: Box<[ [f32; 5] ]>,
    pub indices: Box<[u32]>
}

pub struct BlockShapeRegistry {
    pub bshapes: Box<Vec<BlockShape>>,
}

impl BlockShapeRegistry {
    pub fn new() -> BlockShapeRegistry {
        Self{
            bshapes: Box::new( Vec::<BlockShape>::new() ),
        }
    }

    pub fn add(&mut self, blockshape: BlockShape) -> usize {
        let registry_id = self.bshapes.len();
        self.bshapes.push( blockshape );
        registry_id
    }

    pub fn get(&self, index: usize) -> Option<&BlockShape> {
        self.bshapes.get(index)
    }
}

pub fn make_cube_shape() -> BlockShape {
    BlockShape {
        faces: Box::new([
            FaceDef{ obstructed_by: Some(Direction::PlusY), vertices: Box::new([ [ -0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, 0.5, -0.5, 1.0, 0.0 ], [ -0.5, 0.5, 0.5, 0.0, 1.0 ], [ 0.5, 0.5, 0.5, 1.0, 1.0 ] ]), indices: Box::new([ 0, 2, 1, 1, 2, 3 ]) },
            FaceDef{ obstructed_by: Some(Direction::MinusY), vertices: Box::new([ [ -0.5, -0.5, -0.5, 0.0, 0.0 ], [ 0.5, -0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ]), indices: Box::new([ 0, 1, 2, 1, 3, 2 ]) },
            FaceDef{ obstructed_by: Some(Direction::PlusZ), vertices: Box::new([ [ -0.5, 0.5, 0.5, 0.0, 0.0 ], [ 0.5, 0.5, 0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ]), indices: Box::new([ 0, 2, 1, 1, 2, 3 ]) },
            FaceDef{ obstructed_by: Some(Direction::MinusZ), vertices: Box::new([ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ -0.5, 0.5, -0.5, 1.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ -0.5, -0.5, -0.5, 1.0, 1.0 ] ]), indices: Box::new([ 0, 2, 1, 1, 2, 3 ]) },
            FaceDef{ obstructed_by: Some(Direction::PlusX), vertices: Box::new([ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, 0.5, 0.5, 1.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ]), indices: Box::new([ 0, 1, 2, 1, 3, 2 ]) },
            FaceDef{ obstructed_by: Some(Direction::MinusX), vertices: Box::new([ [ -0.5, 0.5, 0.5, 0.0, 0.0 ], [ -0.5, 0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ -0.5, -0.5, -0.5, 1.0, 1.0 ] ]) , indices: Box::new([ 0, 1, 2, 1, 3, 2 ]) },
        ]),
        obstructs: [true; 6]
    }

}

