

use std::io::{
    Error,
    Read,
};
use std::collections::HashMap;
use std::path::PathBuf;

use figment::Figment;
use figment::providers::{Format, Toml};

use serde::Deserialize;

use ply_rs::parser::Parser;
use ply_rs::ply::*;
use ply_rs::ply::Property::{
    Float,
    ListUInt
};

use crate::wctx::blockmesh::{
    BlockMesh,
    BlockTemplateVertex
};
use crate::wctx::registry::Registry;
use crate::wctx::rotation_group::RotType;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Config {
    block: Vec<BlockPlan>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct BlockPlan {
    pretty_name: String,
    mesh: String,
    lod_mesh: String,
    rot_group: String,
    solid: bool
}

pub struct BlockLoader {
    pub mesh_registry: Registry<BlockMesh>,
    block_names: HashMap<String, u32>,
    pub texture_atlas: crate::wctx::atlas_tex::AtlasTexture,
    figment: Figment,
    config: Option<Config>,
}

impl BlockLoader {
    pub fn create(device: &wgpu::Device, queue: &wgpu::Queue) -> BlockLoader {
        let mesh_registry = Registry::<BlockMesh>::new();
        let block_names = HashMap::<String, u32>::new();
        let texture_atlas = crate::wctx::atlas_tex::AtlasTexture::new(&device, &queue, wgpu::TextureFormat::R8Uint, (32, 32));
        let figment = Figment::new();
        let config = None;

        Self {
            mesh_registry,
            block_names,
            texture_atlas,
            figment,
            config
        }
    }

    pub fn load_toml_from_file(&mut self, filename: PathBuf) -> Result<(), Error> {
        if self.config != None {
            return Err( Error::new::<String>( std::io::ErrorKind::Other, "Cannot load more TOML files, config has already been extracted!".into() ) );
        }
        let toml_dat = Toml::file(filename);
        self.figment = self.figment.clone().merge(toml_dat);
        Ok(())
    }

    pub fn do_extract(&mut self) -> Result<(), figment::Error> {
        self.config = self.figment.extract()?;
        Ok(())
    }

    pub fn resolve_blocks(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), Error> {
        if None == self.config {
            return Err( Error::new::<String>( std::io::ErrorKind::Other, "Cannot resolve blocks yet, config must be extracted first!!".into() ) );
        }

        // make sure to create air
        let nilmesh = BlockMesh::new(
            Vec::<BlockTemplateVertex>::new(),
            Vec::<u32>::new(),
            "Air".to_string(),
            false,
            None,
            RotType::Static,
            false
        );
        self.mesh_registry.add(nilmesh);

        for bp in self.config.as_ref().unwrap().block.clone() {

            let mut rot_group = RotType::Static;

            if bp.rot_group == "RotFace" { rot_group = RotType::RotFace; }
            if bp.rot_group == "RotEdge" { rot_group = RotType::RotVert; }
            if bp.rot_group == "RotVert" { rot_group = RotType::RotEdge; }

            let mut lod_mesh = None;
            let mut has_lod = false;

            if !bp.lod_mesh.is_empty() {
                lod_mesh = Some( Self::load_ply( format!("res/meshes/{}", &bp.lod_mesh) ) );
                has_lod = true;
            }

            let (verts, indices) = Self::load_ply( format!("res/meshes/{}", &bp.mesh) );

            let solid = bp.solid;

            // println!("added a bmesh with {} verts", verts.len() );

            let bmesh = BlockMesh::new(
                verts,
                indices,
                bp.pretty_name,
                has_lod,
                lod_mesh,
                rot_group,
                solid
            );

            let ridx = self.mesh_registry.add(bmesh);
            //println!("at {}", ridx);
        }

        Ok(())
    }

    /*fn check_add_texture(&mut self, tex_name: String, device: &wgpu::Device, queue: &wgpu::Queue, pal_img: &image::DynamicImage) -> u32 {
        let check = self.texture_names.get(&tex_name);
        if let Some(idx) = check {
            return *idx;
        } else {
            let mut bytes = Vec::<u8>::new();
            {
                let mut file = std::fs::File::open( format!("res/texture/block/{}", &tex_name) ).expect("Failed to open image");
                file.read_to_end(&mut bytes).expect("Failed to read from image");
            }
            let image = image::load_from_memory(&bytes).expect("Failed to load image");
            let texture = crate::wctx::texture::Texture::from_image_palettize(&device, &queue, &image, &pal_img, Some(&tex_name)).expect("Failed to convert texture into palette");
            let tex_idx = self.texture_atlas.add_texture(&texture, &device, &queue).expect("Failed to add texture to atlas");
            return tex_idx;
        }
    }*/

    fn load_ply(path: String) -> ( Vec<BlockTemplateVertex>, Vec<u32> ) {

        let mut f = std::fs::File::open(path).unwrap();

        // create a parser
        let p = Parser::<DefaultElement>::new();

        // use the parser: read the entire file
        let ply = p.read_ply(&mut f);

        // Did it work?
        assert!(ply.is_ok());

        let plyu = ply.unwrap();

        let mut verts = Vec::<BlockTemplateVertex>::new();

        for vi in 0..plyu.payload["vertex"].len() {
            let v = &plyu.payload["vertex"][vi];
            if let (Float(x), Float(y), Float(z), Float(nx), Float(ny), Float(nz), Float(ao), Float(subcolor)) = (v["x"].clone(), v["y"].clone(), v["z"].clone(), v["nx"].clone(), v["ny"].clone(), v["nz"].clone(), v["ao"].clone(), v["subcolor"].clone()) {
                verts.push( BlockTemplateVertex::new(
                        [ x, y, z ],
                        [ nx, ny, nz ],
                        ao,
                        subcolor as u8
                    )
                );

            }


        }


        let mut indices = Vec::<u32>::new();
        for idxs in 0..plyu.payload["face"].len() {
            if let ListUInt(ls) = & plyu.payload["face"][idxs]["vertex_indices"] {
                indices.append(&mut ls.clone());
            }
        }

        (verts, indices)
    }

}
