
use ndarray::{
    Array3
};

use crate::wctx::Vertex;

use super::block::{
    BlockRegistry,
    BlockShapeRegistry
};

const CHUNK_SIZE: usize = 16;

#[derive(Copy, Clone)]
pub struct BlockInstance {
    pub blockdef: u16,
    pub exparam: u8,
    pub light: u8
}

pub struct Chunk {
    pub data: Array3<BlockInstance>,
    pub draw_cache: ChunkDrawCache
}

impl Chunk {
    pub fn new() -> Chunk {
        let proto_bi = BlockInstance{
            blockdef: 1,
            exparam: 0,
            light: 255,
        };
        let data = Array3::from_elem((CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE), proto_bi);
        let draw_cache = ChunkDrawCache{ vertices: Box::new( [Vertex{ position:[0.,0.,0.], uv: [0.,0.], array_index: 0, light: 1.0 }; 0] ), indices: Box::new( [1; 0] ) };

        Self {
            data,
            draw_cache
        }
    }

    pub fn update_draw_cache(&mut self, registry: &BlockRegistry, shape_registry: &BlockShapeRegistry) {
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
                shape_registry.get(bdef.shape_id).unwrap().generate_draw_buffers( &mut tverts, &mut tinds, &bdef, bi.exparam, &self.data, pos);
            }
        }

        print!("{}", tverts.len());

        // transfer final data over

        self.draw_cache.vertices = tverts.into_boxed_slice();
        self.draw_cache.indices = tinds.into_boxed_slice();

    }

}


fn is_in_bounds( pos: (usize, usize, usize) ) -> bool {
    pos.0 < CHUNK_SIZE && pos.1 < CHUNK_SIZE && pos.2 < CHUNK_SIZE
}

fn add_vert_with_i(tverts: &mut Vec<Vertex>, find: Vertex) -> u16 {
    let res_len = tverts.len();
    tverts.push(find);
    res_len.try_into().unwrap()
}

/* fn make_quad_cubeface(tverts: &mut Vec<Vertex>, tinds: &mut Vec<u16>, pos: (usize, usize, usize), bdef: &super::block::Block, dir: (i16, i16, i16, i16, i16, i16, i16, i16, i16) ) {
    let center = (pos.0 as f32 + 0.5 + dir.0 as f32 / 2., pos.1 as f32 + 0.5 + dir.1 as f32 / 2., pos.2 as f32 + 0.5 + dir.2 as f32 / 2.);

    let tl = add_vert_with_i( tverts, Vertex::new( [ center.0 - 0.5 * dir.3 as f32 - 0.5 * dir.6 as f32, center.1 - 0.5 * dir.4 as f32 - 0.5 * dir.7 as f32, center.2 - 0.5 * dir.5 as f32 - 0.5 * dir.8 as f32 ],
        [0.0, 0.0], bdef.texture, 1.0 ) );
        //[bdef.texture.left, bdef.texture.top] ) );

    let bl = add_vert_with_i( tverts, Vertex::new( [ center.0 - 0.5 * dir.3 as f32 + 0.5 * dir.6 as f32, center.1 - 0.5 * dir.4 as f32 + 0.5 * dir.7 as f32, center.2 - 0.5 * dir.5 as f32 + 0.5 * dir.8 as f32 ],
        [0.0, 1.0], bdef.texture, 1.0 ) );
        //[bdef.texture.left, bdef.texture.bottom] ) );

    let tr = add_vert_with_i( tverts, Vertex::new( [ center.0 + 0.5 * dir.3 as f32 - 0.5 * dir.6 as f32, center.1 + 0.5 * dir.4 as f32 - 0.5 * dir.7 as f32, center.2 + 0.5 * dir.5 as f32 - 0.5 * dir.8 as f32 ],
        [1.0, 0.0], bdef.texture, 1.0 ) );
        //[bdef.texture.right, bdef.texture.top] ) );

    let br = add_vert_with_i( tverts, Vertex::new( [ center.0 + 0.5 * dir.3 as f32 + 0.5 * dir.6 as f32, center.1 + 0.5 * dir.4 as f32 + 0.5 * dir.7 as f32, center.2 + 0.5 * dir.5 as f32 + 0.5 * dir.8 as f32 ],
        [1.0, 1.0], bdef.texture, 1.0 ) );
        //[bdef.texture.right, bdef.texture.bottom] ) );

    // tri 1
    tinds.push(tl);
    tinds.push(bl);
    tinds.push(tr);
    // tri 2
    tinds.push(bl);
    tinds.push(br);
    tinds.push(tr);
} */

pub struct ChunkDrawCache {
    pub vertices: Box<[Vertex]>,
    pub indices: Box<[u16]>
}
