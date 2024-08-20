

use std::io::{
    Error,
    Read,
};
use std::collections::HashMap;
use std::path::PathBuf;

use figment::Figment;
use figment::providers::{Format, Toml};

use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Config {
    block: Vec<BlockPlan>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct BlockPlan {
    pretty_name: String,
    textures: Vec<String>,
    shape_name: String,
    transparent: Option<bool>,
}

pub struct BlockLoader {
    pub block_registry: crate::wctx::block::BlockRegistry,
    block_names: HashMap<String, u32>,
    pub texture_atlas: crate::wctx::atlas_tex::AtlasTexture,
    texture_names: HashMap<String, u32>,
    pub shape_registry: crate::wctx::block::BlockShapeRegistry,
    shape_names: HashMap<String, u32>,
    figment: Figment,
    config: Option<Config>,
}

impl BlockLoader {
    pub fn create(device: &wgpu::Device, queue: &wgpu::Queue) -> BlockLoader {
        let block_registry = crate::wctx::block::BlockRegistry::new();
        let block_names = HashMap::<String, u32>::new();
        let texture_atlas = crate::wctx::atlas_tex::AtlasTexture::new(&device, &queue, wgpu::TextureFormat::R8Uint, (16, 16));
        let texture_names = HashMap::<String, u32>::new();
        let shape_registry = crate::wctx::block::BlockShapeRegistry::new();
        let shape_names = HashMap::<String, u32>::new();
        let figment = Figment::new();
        let config = None;

        Self {
            block_registry,
            block_names,
            texture_atlas,
            texture_names,
            shape_registry,
            shape_names,
            figment,
            config
        }
    }

    pub fn submit_blockshape_direct(&mut self, bs: crate::wctx::block::BlockShape, name: &String ) -> u32 {
        let idx = self.shape_registry.add(bs);
        self.shape_names.insert( name.clone(), idx );
        idx
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

    pub fn resolve_blocks(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, pal_img: &image::DynamicImage) -> Result<(), Error> {
        if None == self.config {
            return Err( Error::new::<String>( std::io::ErrorKind::Other, "Cannot resolve blocks yet, config must be extracted first!!".into() ) );
        }
        for bp in self.config.as_ref().unwrap().block.clone() {
            let tex_indices: Vec<u32> = bp.textures.into_iter().map( | value | -> u32 {
                self.check_add_texture( value, device, queue, pal_img )
            } ).collect();
            let shape_idx = self.shape_names.get( &bp.shape_name ).ok_or(Error::new::<String>(std::io::ErrorKind::Other, "Shape name not found!".into() ))?;
            let pretty_name = bp.pretty_name.clone();

            let transparent = match bp.transparent {
                Some(value) => value,
                None => false
            };

            self.block_registry.add( *shape_idx, pretty_name, tex_indices, transparent );
        }

        Ok(())
    }

    fn check_add_texture(&mut self, tex_name: String, device: &wgpu::Device, queue: &wgpu::Queue, pal_img: &image::DynamicImage) -> u32 {
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
    }
}
