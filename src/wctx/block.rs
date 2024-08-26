
use std::cmp::min;

use cgmath::{
    Rotation,
};

use crate::wctx::rotation_group;
use crate::wctx::rotation_group::RotFace;
use crate::wctx::world::Vertex;


use cgmath::One;


pub struct Block {
    registry_id: u16,
    pub shape_id: u32,
    //parameter_type: ParamType,
    pub textures: Vec<u32>,
    pub pretty_name: String,
    pub transparent: bool,
}

pub struct BlockRegistry {
    blocks: Vec<Block>,
}

impl BlockRegistry {
    pub fn new() -> BlockRegistry {
        // Always create the air block at position zero!
        let air = Block { registry_id: 0, shape_id: 0, pretty_name: "Air".into(), transparent: true, textures: vec![0] };
        let mut blocks = Vec::<Block>::new();
        blocks.push(air);

        Self {
            blocks
        }
    }

    pub fn add(&mut self, shape_id: u32, pretty_name: String, textures: Vec<u32>, transparent: bool ) -> u16 {
        let registry_id = self.blocks.len() as u16;
        self.blocks.push( Block { registry_id, shape_id, pretty_name, textures, transparent } );
        registry_id
    }

    pub fn get(&self, index: u16) -> Option<&Block> {
        self.blocks.get(index as usize)
    }

    pub fn get_num_blocks(&self) -> u16 {
        return self.blocks.len() as u16;
    }
}

pub struct BlockShape {
    faces: Vec<FaceDef>,
    obstructs: [bool; 6],
    rot_group: rotation_group::RotType
}

impl BlockShape {
    pub fn generate_draw_buffers(&self, vertex_buffer: &mut Vec<Vertex>, index_buffer: &mut Vec<u16>, blockdef: &Block, exparam: u8, bdc: crate::wctx::chunk::BlockDrawContext, world_pos: (usize, usize, usize), pos: (usize, usize, usize) ) {
        let mut quat = cgmath::Quaternion::<f32>::one();
        match self.rot_group {
            rotation_group::RotType::RotFace => {
                quat = rotation_group::generate_quat_from_rf( rotation_group::num_to_rf( exparam & 0b0000_0111 ).unwrap() );
            },
            rotation_group::RotType::RotVert => {
                quat = rotation_group::generate_quat_from_rv( rotation_group::num_to_rv( exparam & 0b0000_0111 ).unwrap() );
            },
            rotation_group::RotType::RotEdge => {
                quat = rotation_group::generate_quat_from_re( rotation_group::num_to_re( exparam & 0b0000_1111 ).unwrap() );
            },
            rotation_group::RotType::Static => {},
            _ => {}
        }

        for fi in self.faces.iter().enumerate() {
            let (f, face) = fi;

            // check whether there's an obstruction here
            if let Some(obstruct) = face.obstructed_by {
                if bdc.obstructions[ rotation_group::rf_to_num( rotation_group::rotate_rf(obstruct, &quat).unwrap() ) as usize ] {
                    continue;
                }
            }

            let mut temp_indices = Vec::<u32>::new();
            let center = cgmath::Vector3::<f32>::new( pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5 );
            for vertdef in face.vertices.iter() {
                temp_indices.push( vertex_buffer.len().try_into().unwrap() );
                let tex_index = blockdef.textures[ min( f, blockdef.textures.len() - 1 ) ];
                let mut vec = cgmath::Vector3::new( vertdef[0], vertdef[1], vertdef[2] );
                vec = quat * vec;
                vertex_buffer.push( Vertex::new( [ world_pos.0 as f32 + center.x + vec.x, world_pos.1 as f32 + center.y + vec.y, world_pos.2 as f32 + center.z + vec.z ], [vertdef[3], vertdef[4]], tex_index, 1.0) );
            }

            for ind in face.indices.iter() {
                index_buffer.push( temp_indices[ *ind as usize ] as u16 );
            }
        }

    }

    pub fn does_obstruct(&self, exparam: u8, dir: rotation_group::RotFace) -> bool {
        let mut quat = cgmath::Quaternion::<f32>::one();
        match self.rot_group {
            rotation_group::RotType::RotFace => {
                quat = rotation_group::generate_quat_from_rf( rotation_group::num_to_rf( exparam & 0b0000_0111 ).unwrap() );
            },
            rotation_group::RotType::RotVert => {
                quat = rotation_group::generate_quat_from_rv( rotation_group::num_to_rv( exparam & 0b0000_0111 ).unwrap() );
            },
            rotation_group::RotType::RotEdge => {
                quat = rotation_group::generate_quat_from_re( rotation_group::num_to_re( exparam & 0b0000_1111 ).unwrap() );
            },
            rotation_group::RotType::Static => {},
            _ => {}
        }
        self.obstructs[ rotation_group::rf_to_num( rotation_group::vector_to_rf( quat.invert() * rotation_group::rf_to_vector(dir) ).expect("Error! failed to convert vector to rf") ) as usize ]
    }

}

pub struct FaceDef {
    pub obstructed_by: Option<RotFace>,
    pub vertices: Vec< [f32; 5] >,
    pub indices: Vec<u32>
}

pub struct BlockShapeRegistry {
    pub bshapes: Vec<BlockShape>,
}

impl BlockShapeRegistry {
    pub fn new() -> BlockShapeRegistry {
        Self{
            bshapes: Vec::<BlockShape>::new(),
        }
    }

    pub fn add(&mut self, blockshape: BlockShape) -> u32 {
        let registry_id = self.bshapes.len() as u32;
        self.bshapes.push( blockshape );
        registry_id
    }

    pub fn get(&self, index: u32) -> Option<&BlockShape> {
        self.bshapes.get(index as usize)
    }
}

pub fn make_cube_shape() -> BlockShape {
    BlockShape {
        faces: vec![
            FaceDef{ obstructed_by: Some(RotFace::PlusY), vertices: vec![ [ -0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, 0.5, -0.5, 1.0, 0.0 ], [ -0.5, 0.5, 0.5, 0.0, 1.0 ], [ 0.5, 0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1, 1, 2, 3 ] },
            FaceDef{ obstructed_by: Some(RotFace::MinusY), vertices: vec![ [ -0.5, -0.5, -0.5, 0.0, 0.0 ], [ 0.5, -0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 1, 2, 1, 3, 2 ] },
            FaceDef{ obstructed_by: Some(RotFace::PlusZ), vertices: vec![ [ -0.5, 0.5, 0.5, 0.0, 0.0 ], [ 0.5, 0.5, 0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1, 1, 2, 3 ] },
            FaceDef{ obstructed_by: Some(RotFace::MinusZ), vertices: vec![ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ -0.5, 0.5, -0.5, 1.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ -0.5, -0.5, -0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1, 1, 2, 3 ] },
            FaceDef{ obstructed_by: Some(RotFace::PlusX), vertices: vec![ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, 0.5, 0.5, 1.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 1, 2, 1, 3, 2 ] },
            FaceDef{ obstructed_by: Some(RotFace::MinusX), vertices: vec![ [ -0.5, 0.5, 0.5, 0.0, 0.0 ], [ -0.5, 0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ -0.5, -0.5, -0.5, 1.0, 1.0 ] ] , indices: vec![ 0, 1, 2, 1, 3, 2 ] },
        ],
        obstructs: [true; 6],
        rot_group: rotation_group::RotType::Static
    }
}

pub fn make_slope_shape() -> BlockShape {
    BlockShape {
        faces: vec![
            FaceDef{ obstructed_by: Some(RotFace::MinusY), vertices: vec![ [ -0.5, -0.5, -0.5, 0.0, 0.0 ], [ 0.5, -0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 1, 2, 1, 3, 2 ] }, // minus Y cube face
            FaceDef{ obstructed_by: Some(RotFace::MinusZ), vertices: vec![ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ -0.5, 0.5, -0.5, 1.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ -0.5, -0.5, -0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1, 1, 2, 3 ] }, // minus Z cube face
            FaceDef{ obstructed_by: Some(RotFace::PlusX), vertices: vec![ [ 0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, -0.5, -0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1] }, // plus X tri face
            FaceDef{ obstructed_by: Some(RotFace::MinusX), vertices: vec![ [ -0.5, 0.5, -0.5, 0.0, 0.0 ], [ -0.5, -0.5, -0.5, 0.0, 1.0 ], [ -0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 1, 2] }, // minus X tri face
            FaceDef{ obstructed_by: None, vertices: vec![ [ -0.5, 0.5, -0.5, 0.0, 0.0 ], [ 0.5, 0.5, -0.5, 1.0, 0.0 ], [ -0.5, -0.5, 0.5, 0.0, 1.0 ], [ 0.5, -0.5, 0.5, 1.0, 1.0 ] ], indices: vec![ 0, 2, 1, 1, 2, 3 ] },
        ],
        obstructs: [ false, true, false, true, false, false ],
        rot_group: rotation_group::RotType::RotEdge
    }
}

pub fn make_corner_shape() -> BlockShape {
    BlockShape {
        faces: vec![
            FaceDef{ obstructed_by: Some(RotFace::MinusY), vertices: vec![ [-0.5, -0.5, -0.5, 0.0, 0.0], [0.5, -0.5, -0.5, 1.0, 0.0], [-0.5, -0.5, 0.5, 0.0, 1.0] ], indices: vec![0,1,2] },
            FaceDef{ obstructed_by: Some(RotFace::MinusX), vertices: vec![ [-0.5, -0.5, -0.5, 0.0, 0.0], [-0.5, 0.5, -0.5, 1.0, 0.0], [-0.5, -0.5, 0.5, 0.0, 1.0] ], indices: vec![0,2,1] },
            FaceDef{ obstructed_by: Some(RotFace::MinusZ), vertices: vec![ [-0.5, -0.5, -0.5, 0.0, 0.0], [0.5, -0.5, -0.5, 1.0, 0.0], [-0.5, 0.5, -0.5, 0.0, 1.0] ], indices: vec![0,2,1] },
            FaceDef{ obstructed_by: None, vertices: vec![ [0.5, -0.5, -0.5, 1.0, 1.0], [-0.5, -0.5, 0.5, 0.0, 1.0], [-0.5, 0.5, -0.5, 0.5, 0.0] ], indices: vec![0,2,1] }
        ],
        obstructs: [false; 6],
        rot_group: rotation_group::RotType::RotVert
    }
}
