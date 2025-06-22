 
use std::string::String;

use ply_rs::ply::{
    Property,
    PropertyAccess
};

use cgmath::{
    Vector3,
    Quaternion,
    One
};

use crate::wctx::rotation_group;
use crate::wctx::blockdef::BlockDef;
use crate::wctx::registry::Registerable;


pub struct BlockMesh {
    registry_id: u32,
    verts: Vec<BlockTemplateVertex>,
    indices: Vec<u32>,
    pub pretty_name: String,
    pub has_lod: bool,
    lod: Option<(Vec<BlockTemplateVertex>, Vec<u32>)>,
    pub rot_group: rotation_group::RotType,
    pub solid: bool
}

impl BlockMesh {
    pub fn new(verts: Vec<BlockTemplateVertex>, indices: Vec<u32>, pretty_name: String, has_lod: bool, lod: Option<(Vec<BlockTemplateVertex>, Vec<u32>)>, rot_group: rotation_group::RotType, solid: bool) -> BlockMesh {

        Self {
            registry_id: 0,
            verts,
            indices,
            pretty_name,
            has_lod,
            lod,
            rot_group,
            solid
        }
    }

    pub fn generate_verts(&self, lod: bool, bdef: &BlockDef, spatial_pos: (u32, u32, u32), vbuf_offset: u32 ) -> (Vec<BlockVertex>, Vec<u32>) {

        let mut quat = Quaternion::<f32>::one();
        match self.rot_group {
            rotation_group::RotType::RotFace => {
                quat = rotation_group::generate_quat_from_rf( rotation_group::num_to_rf( bdef.get_rotation() ).unwrap() );
            },
            rotation_group::RotType::RotVert => {
                quat = rotation_group::generate_quat_from_rv( rotation_group::num_to_rv( bdef.get_rotation() ).unwrap() );
            },
            rotation_group::RotType::RotEdge => {
                quat = rotation_group::generate_quat_from_re( rotation_group::num_to_re( bdef.get_rotation() ).unwrap() );
            },
            rotation_group::RotType::Static => {},
            _ => {}
        }

        let mut out_verts: Vec<BlockVertex> = Vec::new();
        let mut out_idxs: Vec<u32> = Vec::new();

        for v in ( if lod && self.has_lod { &self.lod.as_ref().unwrap().0 } else { &self.verts }) {

            let mut tmp_pos = Vector3::new( v.position[0], v.position[1], v.position[2] );

            tmp_pos = quat * tmp_pos;
            tmp_pos += Vector3::new( spatial_pos.0 as f32 + 0.5, spatial_pos.1 as f32 + 0.5, spatial_pos.2 as f32 + 0.5 );

            let mut tmp_norm = Vector3::new( v.normal[0], v.normal[1], v.normal[2] );
            tmp_norm = quat * tmp_norm;

            out_verts.push( BlockVertex{
                position: [tmp_pos.x, tmp_pos.y, tmp_pos.z],
                color: [1.0, 1.0, 1.0],
                normal: [tmp_norm.x, tmp_norm.y, tmp_norm.z],
                ao: 1.0,
                metalrough: 512,
                tex_index: 0
            } );
        }

        for i in (if lod && self.has_lod { &self.lod.as_ref().unwrap().1 } else { &self.indices} ) {
            out_idxs.push(i + vbuf_offset);
        }

        return (out_verts, out_idxs);
    }
}

impl Registerable for BlockMesh {
    fn register(&mut self, id: u32) {
        self.registry_id = id;
    }
}


pub struct BlockTemplateVertex {
    position: [f32; 3],
    normal: [f32; 3],
    ao: f32,
    palette_index: u8
}

impl BlockTemplateVertex {
    pub fn new(position: [f32; 3],normal: [f32; 3], ao: f32, palette_index: u8) -> BlockTemplateVertex {
        Self{
            position,
            normal,
            ao,
            palette_index
        }
    }
}




#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
    ao: f32,
    metalrough: i32,
    tex_index: u32
}

impl BlockVertex {
    pub fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3], ao: f32, metalrough: i32, tex_index: u32 ) -> BlockVertex {
        Self{
            position,
            color,
            normal,
            ao,
            metalrough,
            tex_index
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // color
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // normal
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // ao
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute { // metalrough
                    offset: std::mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Sint32,
                },
                wgpu::VertexAttribute { // tex_index
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                }
            ]
        }
    }
}
