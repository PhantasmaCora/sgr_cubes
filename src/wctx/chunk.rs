
use ndarray::{
    Array3,
    ArrayView2,
    s
};

use serde::{
    Serialize,
    Deserialize
};

use crate::wctx::world::Vertex;

use crate::wctx::block::{
    BlockRegistry,
    BlockShapeRegistry
};

use crate::wctx::rotation_group;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_CHUNKS: usize = 8;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BlockInstance {
    pub blockdef: u16,
    pub exparam: u8,
    pub light: u8
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub data: Array3<BlockInstance>,
    #[serde(skip_serializing)]
    #[serde(default = "get_a_true")]
    pub dirty: bool,
    #[serde(skip)]
    pub draw_cache: ChunkDrawCache
}
fn get_a_true() -> bool {
    true
}


impl Chunk {
    pub fn new() -> Chunk {
        let proto_bi = BlockInstance{
            blockdef: 0,
            exparam: 0,
            light: 255,
        };
        Self::from_blockinstance(proto_bi)
    }

    pub fn from_blockinstance( bi: BlockInstance ) -> Chunk {
        let data = Array3::from_elem((CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE), bi);
        let dirty = true;
        let draw_cache = ChunkDrawCache{ vertices: Vec::<Vertex>::new(), indices: Vec::<u16>::new() };

        Self {
            data,
            dirty,
            draw_cache
        }
    }

    pub fn update_draw_cache(&mut self, world_pos: (usize, usize, usize), registry: &BlockRegistry, shape_registry: &BlockShapeRegistry, cdc: ChunkDrawContext) {
        let mut tverts = Vec::<Vertex>::new();
        let mut tinds = Vec::<u16>::new();

        let mut iiter = self.data.indexed_iter();

        // iterate over blockinstances in the chunk until done.
        while let Some(tup) = iiter.next() {
            let pos = tup.0;
            let bi = tup.1;

            if bi.blockdef == 0 {
                continue;
            }

            if let Some(bdef) = registry.get(bi.blockdef) {
                let bdc = self.create_bdc( pos, registry, shape_registry, &cdc );
                shape_registry.get(bdef.shape_id).unwrap().generate_draw_buffers( &mut tverts, &mut tinds, &bdef, bi.exparam, bdc, world_pos, pos);
            }
        }

        // transfer final data over
        self.draw_cache.vertices = tverts;
        self.draw_cache.indices = tinds;

        self.dirty = false;
    }

    pub fn create_bdc(&self, pos: (usize, usize, usize), registry: &BlockRegistry, shape_registry: &BlockShapeRegistry, cdc: &ChunkDrawContext) -> BlockDrawContext {
        let mut out = [false; 6];

        for idx in 0..6 {
            let v = rotation_group::rf_to_vector( rotation_group::num_to_rf(idx).unwrap() );
            let opos = ( pos.0 as i32 + v.x as i32, pos.1 as i32 + v.y as i32, pos.2 as i32 + v.z as i32 );
            let mut bi = BlockInstance{blockdef: 0, exparam: 0, light: 0};
            if opos.0 < 0 {
                match cdc.minus_x {
                    Some(slice) => { bi = slice[ (opos.1 as usize, opos.2 as usize) ] },
                    None => {}
                };
            } else if opos.0 > (CHUNK_SIZE - 1).try_into().unwrap() {
                match cdc.plus_x {
                    Some(slice) => { bi = slice[ (opos.1 as usize, opos.2 as usize) ]; },
                    None => {}
                };
            } else if opos.1 < 0 {
                match cdc.minus_y {
                    Some(slice) => { bi = slice[ (opos.0 as usize, opos.2 as usize) ]; },
                    None => {}
                };
            } else if opos.1 > (CHUNK_SIZE - 1).try_into().unwrap() {
                match cdc.plus_y {
                    Some(slice) => { bi = slice[ (opos.0 as usize, opos.2 as usize) ]; },
                    None => {}
                };
            } else if opos.2 < 0 {
                match cdc.minus_z {
                    Some(slice) => { bi = slice[ (opos.0 as usize, opos.1 as usize) ]; },
                    None => {}
                };
            } else if opos.2 > (CHUNK_SIZE - 1).try_into().unwrap() {
                match cdc.plus_z {
                    Some(slice) => { bi = slice[ (opos.0 as usize, opos.1 as usize) ]; },
                    None => {}
                };
            } else {
                bi = self.data[ (opos.0 as usize, opos.1 as usize, opos.2 as usize) ];
            }

            let bdef = registry.get(bi.blockdef).unwrap();
            if !bdef.transparent {
                let sdef = shape_registry.get(bdef.shape_id).unwrap();
                out[ idx as usize ] = sdef.does_obstruct( bi.exparam, rotation_group::reverse_rf( rotation_group::num_to_rf( idx ).unwrap() ) );
            }
        }

        BlockDrawContext {
            obstructions: out
        }
    }

}


fn is_in_bounds( pos: (usize, usize, usize) ) -> bool {
    pos.0 < CHUNK_SIZE && pos.1 < CHUNK_SIZE && pos.2 < CHUNK_SIZE
}

#[derive(Clone)]
pub struct ChunkDrawCache {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

impl Default for ChunkDrawCache {
    fn default() -> ChunkDrawCache {
        Self {
            vertices: Vec::<Vertex>::new(),
            indices: Vec::<u16>::new()
        }
    }
}

impl ChunkDrawCache {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

pub struct BlockDrawContext {
    pub obstructions: [bool; 6],
}

impl Default for BlockDrawContext {
    fn default() -> BlockDrawContext {
        Self {
            obstructions: [false; 6]
        }
    }
}

pub struct ChunkDrawContext<'a> {
    pub minus_z: Option<ArrayView2<'a, BlockInstance>>,
    pub plus_z: Option<ArrayView2<'a, BlockInstance>>,
    pub minus_y: Option<ArrayView2<'a, BlockInstance>>,
    pub plus_y: Option<ArrayView2<'a, BlockInstance>>,
    pub minus_x: Option<ArrayView2<'a, BlockInstance>>,
    pub plus_x: Option<ArrayView2<'a, BlockInstance>>
}

impl<'a> ChunkDrawContext<'a> {
    pub fn new() -> ChunkDrawContext<'a> {
        Self{
            minus_z: None,
            plus_z: None,
            minus_y: None,
            plus_y: None,
            minus_x: None,
            plus_x: None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChunkManager {
    pub data: Array3<Chunk>
}

impl ChunkManager {
    pub fn new() -> ChunkManager {
        let constr = | loc: (usize, usize, usize) | -> Chunk {
            if loc.1 < 2 {
                let proto_bi = BlockInstance{
                    blockdef: 1,
                    exparam: 0,
                    light: 255,
                };
                Chunk::from_blockinstance(proto_bi)
            } else {
                Chunk::new()
            }
        };
        let data = Array3::from_shape_fn( (WORLD_CHUNKS, WORLD_CHUNKS, WORLD_CHUNKS), constr );

        Self{
            data
        }
    }

    pub fn get_block(&self, world_pos: (usize, usize, usize) ) -> & BlockInstance {
        let chunk_index = ( world_pos.0 / CHUNK_SIZE, world_pos.1 / CHUNK_SIZE, world_pos.2 / CHUNK_SIZE );
        let inner_index = ( world_pos.0 % CHUNK_SIZE, world_pos.1 % CHUNK_SIZE, world_pos.2 % CHUNK_SIZE );
        &self.data[chunk_index].data[inner_index]
    }

    pub fn get_mut_block(&mut self, world_pos: (usize, usize, usize) ) -> &mut BlockInstance {
        let chunk_index = ( world_pos.0 / CHUNK_SIZE, world_pos.1 / CHUNK_SIZE, world_pos.2 / CHUNK_SIZE );
        let inner_index = ( world_pos.0 % CHUNK_SIZE, world_pos.1 % CHUNK_SIZE, world_pos.2 % CHUNK_SIZE );
        self.data[chunk_index].dirty = true;
        // set adjacent chunks as dirty if needed
        if inner_index.0 == 0 && chunk_index.0 > 0 { self.data[ (chunk_index.0 - 1, chunk_index.1, chunk_index.2) ].dirty = true; }
        if inner_index.0 == CHUNK_SIZE - 1 && chunk_index.0 < WORLD_CHUNKS - 1 { self.data[ (chunk_index.0 + 1, chunk_index.1, chunk_index.2) ].dirty = true; }
        if inner_index.1 == 0 && chunk_index.1 > 0 { self.data[ (chunk_index.0, chunk_index.1 - 1, chunk_index.2) ].dirty = true; }
        if inner_index.1 == CHUNK_SIZE - 1 && chunk_index.1 < WORLD_CHUNKS - 1 { self.data[ (chunk_index.0, chunk_index.1 + 1, chunk_index.2) ].dirty = true; }
        if inner_index.2 == 0 && chunk_index.2 > 0 { self.data[ (chunk_index.0, chunk_index.1, chunk_index.2 - 1) ].dirty = true; }
        if inner_index.2 == CHUNK_SIZE - 1 && chunk_index.2 < WORLD_CHUNKS - 1 { self.data[ (chunk_index.0, chunk_index.1, chunk_index.2 + 1) ].dirty = true; }


        &mut self.data[chunk_index].data[inner_index]
    }

    pub fn update_dirty_chunks(&mut self, registry: &BlockRegistry, shape_registry: &BlockShapeRegistry ) {
        let rebuild = |this: &mut Self, ch_idx: (usize, usize, usize), wpos: (usize, usize, usize)| {
            let mut dirty = false;
            {
                dirty = this.data.get( ch_idx ).expect("failed to get chunk").dirty;
            }
            if dirty {
                let mut cdc = ChunkDrawContext::new();

                if ch_idx.0 > 0 {
                    let ptr = this.data.get_ptr( ( ch_idx.0 - 1 as usize, ch_idx.1 as usize, ch_idx.2 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.minus_x = Some( (*ptr).data.slice(s![ CHUNK_SIZE - 1, 0..CHUNK_SIZE, 0..CHUNK_SIZE ]) );
                    }
                }
                if ch_idx.0 < WORLD_CHUNKS - 1 as usize {
                    let ptr = this.data.get_ptr( ( ch_idx.0 + 1 as usize, ch_idx.1 as usize, ch_idx.2 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.plus_x = Some( (*ptr).data.slice(s![ 0, 0..CHUNK_SIZE, 0..CHUNK_SIZE ]) );
                    }
                }

                if ch_idx.1 > 0 {
                    let ptr = this.data.get_ptr( ( ch_idx.0 as usize, ch_idx.1 - 1 as usize, ch_idx.2 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.minus_y = Some( (*ptr).data.slice(s![ 0..CHUNK_SIZE, CHUNK_SIZE - 1, 0..CHUNK_SIZE ]) );
                    }
                }
                if ch_idx.1 < WORLD_CHUNKS - 1 as usize {
                    let ptr = this.data.get_ptr( ( ch_idx.0 as usize, ch_idx.1 + 1 as usize, ch_idx.2 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.plus_y = Some( (*ptr).data.slice(s![ 0..CHUNK_SIZE, 0, 0..CHUNK_SIZE ]) );
                    }
                }

                if ch_idx.2 > 0 {
                    let ptr = this.data.get_ptr( ( ch_idx.0 as usize, ch_idx.1 as usize, ch_idx.2 - 1 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.minus_z = Some( (*ptr).data.slice(s![ 0..CHUNK_SIZE, 0..CHUNK_SIZE, CHUNK_SIZE - 1 ]) );
                    }
                }
                if ch_idx.2 < WORLD_CHUNKS - 1 as usize {
                    let ptr = this.data.get_ptr( ( ch_idx.0 as usize, ch_idx.1 as usize, ch_idx.2 + 1 as usize ) ).expect("Failed to get chunk pointer!");
                    unsafe {
                        cdc.plus_z = Some( (*ptr).data.slice(s![ 0..CHUNK_SIZE, 0..CHUNK_SIZE, 0 ]) );
                    }
                }

                let ch = this.data.get_mut( ch_idx ).expect("failed to get chunk");
                ch.update_draw_cache(wpos, registry, shape_registry, cdc);
            }
        };
        for x in 0..WORLD_CHUNKS {
            for y in 0..WORLD_CHUNKS {
                for z in 0..WORLD_CHUNKS {
                    rebuild( self, (x, y, z), (x * CHUNK_SIZE, y * CHUNK_SIZE, z * CHUNK_SIZE) );
                }
            }
        }
    }

    pub fn get_render_chunks(&self) -> Vec<ChunkDrawCache> {
        let mut cache_vec = Vec::<ChunkDrawCache>::new();

        for ch in self.data.iter() {
            let cache = &ch.draw_cache;
            if !cache.is_empty() {
                cache_vec.push( cache.clone() );
            }
        }

        cache_vec
    }
}
