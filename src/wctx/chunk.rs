
use std::cmp::max;

use ndarray::{
    Array3
};

use serde::{
    Serialize,
    Deserialize
};

use crate::wctx::blockmesh::{
    BlockMesh,
    BlockVertex
};
use crate::wctx::blockdef::BlockDef;
use crate::wctx::registry::Registry;

use crate::wctx::rotation_group;

use cgmath::{
    Vector3,
    Point3,
    InnerSpace
};

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_CHUNKS: [usize; 3] = [ 8, 12, 16 ];

#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub data: Array3<BlockDef>,
    #[serde(skip_serializing)]
    #[serde(default = "get_a_true")]
    pub dirty: bool,
    #[serde(skip)]
    pub draw_cache: ChunkDrawCache,
    #[serde(skip)]
    pub low_draw_cache: ChunkDrawCache
}
fn get_a_true() -> bool {
    true
}

impl Chunk {
    pub fn new() -> Chunk {
        let proto_bd = BlockDef{
            blockmesh: 0,
            exparam: 0,
        };
        Self::from_blockdef(proto_bd)
    }

    pub fn from_blockdef( bd: BlockDef ) -> Chunk {
        let data = Array3::from_elem((CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE), bd);
        let dirty = true;
        let draw_cache = ChunkDrawCache::default();
        let low_draw_cache = ChunkDrawCache::default();

        Self {
            data,
            dirty,
            draw_cache,
            low_draw_cache
        }
    }

    fn is_solid(&self, pos: [usize; 3], registry: &Registry<BlockMesh> ) -> bool {
        registry.get( self.data[pos].blockmesh ).unwrap().solid
    }

    pub fn update_draw_cache( &mut self, mesh_registry: &Registry<BlockMesh>, worldpos: (usize, usize, usize) ) {
        //println!("updating chunk at {:?}", worldpos);

        let mut tverts = Vec::<BlockVertex>::new();
        let mut tinds = Vec::<u32>::new();

        let mut lowtverts = Vec::<BlockVertex>::new();
        let mut lowtinds = Vec::<u32>::new();

        let mut idxs = Vec::<[usize; 3]>::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if x < 1 || x >= CHUNK_SIZE - 1 || y < 1 || y >= CHUNK_SIZE - 1 || z < 1 || z >= CHUNK_SIZE - 1 {
                        idxs.push( [x,y,z] );
                    } else if self.is_solid([x-1, y, z], mesh_registry) && self.is_solid([x+1, y, z], mesh_registry) &&
                        self.is_solid([x, y-1, z], mesh_registry) && self.is_solid([x, y+1, z], mesh_registry) &&
                        self.is_solid([x, y, z-1], mesh_registry) && self.is_solid([x, y, z+1], mesh_registry) {

                    } else {
                        idxs.push( [x,y,z] );
                    }
                }
            }
        }


        let mut iiter = idxs.iter();

        // iterate over blockinstances in the chunk until done.
        while let Some(idx) = iiter.next() {
            let pos = idx;
            let bi = &self.data[*idx];

            if bi.blockmesh == 0 {
                continue;
            }

            if let Some(bmesh) = mesh_registry.get(bi.blockmesh) {
                let (mut newverts, mut newinds) = bmesh.generate_verts(false, bi, (pos[0] as u32 + worldpos.0 as u32, pos[1] as u32 + worldpos.1 as u32, pos[2] as u32 + worldpos.2 as u32), tverts.len() as u32 );
                tverts.append(&mut newverts);
                tinds.append(&mut newinds);

                (newverts, newinds) = bmesh.generate_verts(true, bi, (pos[0] as u32 + worldpos.0 as u32, pos[1] as u32 + worldpos.1 as u32, pos[2] as u32 + worldpos.2 as u32), lowtverts.len() as u32 );

                lowtverts.append(&mut newverts);
                lowtinds.append(&mut newinds);
            }
        }

        // transfer final data over
        self.draw_cache.vertices = tverts;
        self.draw_cache.indices = tinds;

        self.low_draw_cache.vertices = lowtverts;
        self.low_draw_cache.indices = lowtinds;

        self.dirty = false;
    }
}

#[derive(Clone)]
pub struct ChunkDrawCache {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>
}

impl Default for ChunkDrawCache {
    fn default() -> ChunkDrawCache {
        Self {
            vertices: Vec::<BlockVertex>::new(),
            indices: Vec::<u32>::new()
        }
    }
}

impl ChunkDrawCache {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkManager {
    pub data: Array3<Chunk>,
    pub size: usize
}

impl ChunkManager {
    pub fn new(size: usize) -> ChunkManager {
        let constr = | loc: (usize, usize, usize) | -> Chunk {
            if loc.1 < 2 {
                let proto_bi = BlockDef{
                    blockmesh: 1,
                    exparam: 0
                };
                Chunk::from_blockdef(proto_bi)
            } else {
                Chunk::new()
            }
        };
        let data = Array3::from_shape_fn( (WORLD_CHUNKS[size], WORLD_CHUNKS[size], WORLD_CHUNKS[size]), constr );

        Self{
            size,
            data
        }
    }

    pub fn get_block(&self, world_pos: (usize, usize, usize) ) -> & BlockDef {
        let chunk_index = ( world_pos.0 / CHUNK_SIZE, world_pos.1 / CHUNK_SIZE, world_pos.2 / CHUNK_SIZE );
        let inner_index = ( world_pos.0 % CHUNK_SIZE, world_pos.1 % CHUNK_SIZE, world_pos.2 % CHUNK_SIZE );
        &self.data[chunk_index].data[inner_index]
    }

    pub fn get_mut_block(&mut self, world_pos: (usize, usize, usize) ) -> &mut BlockDef {
        let chunk_index = ( world_pos.0 / CHUNK_SIZE, world_pos.1 / CHUNK_SIZE, world_pos.2 / CHUNK_SIZE );
        let inner_index = ( world_pos.0 % CHUNK_SIZE, world_pos.1 % CHUNK_SIZE, world_pos.2 % CHUNK_SIZE );
        self.data[chunk_index].dirty = true;

        &mut self.data[chunk_index].data[inner_index]
    }

    pub fn update_dirty_chunks(&mut self, mesh_registry: &Registry<BlockMesh> ) {
        let rebuild = |ch: ((usize, usize, usize), &mut Chunk)| {
            if !ch.1.dirty {
                return;
            }
            ch.1.update_draw_cache( mesh_registry, ( ch.0.0 * CHUNK_SIZE, ch.0.1 * CHUNK_SIZE, ch.0.2 * CHUNK_SIZE ) );
        };

        self.data.indexed_iter_mut().for_each( rebuild );
    }

    pub fn get_render_chunks(&self, pos: Point3<f32>, viewvec: Vector3<f32> ) -> Vec<ChunkDrawCache> {
        let mut cache_vec = Vec::<ChunkDrawCache>::new();

        let mut iiter = self.data.indexed_iter();

        let local_idx = ( (pos.x / 16.0) as usize, (pos.y / 16.0) as usize, (pos.z / 16.0) as usize );

        while let Some(tp) = iiter.next() {
            let (cpos, ch) = tp;

            let ds = max( local_idx.0.abs_diff(cpos.0), max( local_idx.1.abs_diff(cpos.1), local_idx.2.abs_diff(cpos.2) ) );

            if ds < 2usize {
                cache_vec.push( ch.draw_cache.clone() );
            } else if ds < 3usize {
                cache_vec.push( ch.low_draw_cache.clone() );
            }
        }

        cache_vec
    }

    pub fn get_all_render_chunks(&self, low: bool) -> Vec<ChunkDrawCache> {
        let mut iter = self.data.iter();
        let mut cache_vec = Vec::<ChunkDrawCache>::new();

        while let Some(ch) = iter.next() {

            if ch.draw_cache.is_empty() { continue; }

            if !low {
                cache_vec.push( ch.draw_cache.clone() );
            } else {
                cache_vec.push( ch.low_draw_cache.clone() );
            }
        }

        cache_vec
    }

}
