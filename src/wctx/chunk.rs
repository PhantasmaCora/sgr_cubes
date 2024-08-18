
use ndarray::{
    Array3
};

use crate::wctx::Vertex;

use crate::wctx::block::{
    BlockRegistry,
    BlockShapeRegistry
};

const CHUNK_SIZE: usize = 16;
const WORLD_CHUNKS: usize = 8;

#[derive(Copy, Clone)]
pub struct BlockInstance {
    pub blockdef: u16,
    pub exparam: u8,
    pub light: u8
}

pub struct Chunk {
    pub data: Array3<BlockInstance>,
    pub dirty: bool,
    pub draw_cache: ChunkDrawCache
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
        let draw_cache = ChunkDrawCache{ vertices: Box::new( [Vertex{ position:[0.,0.,0.], uv: [0.,0.], array_index: 0, light: 1.0 }; 0] ), indices: Box::new( [1; 0] ) };

        Self {
            data,
            dirty,
            draw_cache
        }
    }

    pub fn update_draw_cache(&mut self, world_pos: (usize, usize, usize), registry: &BlockRegistry, shape_registry: &BlockShapeRegistry) {
        let mut tverts = Vec::<Vertex>::new();
        let mut tinds = Vec::<u16>::new();

        let mut iiter = self.data.indexed_iter();

        // iterate over blockinstances in the chunk until done.
        for tup in iiter {
            let pos = tup.0;
            let bi = tup.1;

            if bi.blockdef == 0 {
                continue;
            }

            if let Some(bdef) = registry.get(bi.blockdef) {
                shape_registry.get(bdef.shape_id).unwrap().generate_draw_buffers( &mut tverts, &mut tinds, bdef, bi.exparam, &self.data, world_pos, pos);
            }
        }

        // transfer final data over

        self.draw_cache.vertices = tverts.into_boxed_slice();
        self.draw_cache.indices = tinds.into_boxed_slice();

        self.dirty = false;
    }

}


fn is_in_bounds( pos: (usize, usize, usize) ) -> bool {
    pos.0 < CHUNK_SIZE && pos.1 < CHUNK_SIZE && pos.2 < CHUNK_SIZE
}

#[derive(Clone)]
pub struct ChunkDrawCache {
    pub vertices: Box<[Vertex]>,
    pub indices: Box<[u16]>
}

impl ChunkDrawCache {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

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
        &mut self.data[chunk_index].data[inner_index]
    }

    pub fn update_dirty_chunks(&mut self, registry: &BlockRegistry, shape_registry: &BlockShapeRegistry ) {
        let rebuild = |ch: &mut Chunk, wpos: (usize, usize, usize)| {
            if ch.dirty {
                ch.update_draw_cache(wpos, registry, shape_registry);
            }
        };
        for x in 0..WORLD_CHUNKS {
            for y in 0..WORLD_CHUNKS {
                for z in 0..WORLD_CHUNKS {
                    rebuild( &mut self.data[ (x, y, z) ], (x * CHUNK_SIZE, y * CHUNK_SIZE, z * CHUNK_SIZE) );
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
