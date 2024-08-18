
use std::collections::HashMap;
use std::io::{Read, Error};
use std::fs::File;

use toml::{
    Value,
    Table
};

use crate::wctx::block::BlockRegistry;


pub struct BlockLoader {
    pub br: BlockRegistry,
    pub tex_hm: HashMap<String, u32>,
    pub atlas: crate::wctx::atlas_tex::AtlasTexture,
    pub shape_hm: HashMap<String, usize>,
    pub result: HashMap<String, u16>,
}

impl BlockLoader {
    pub fn load_blocks_from_file( &mut self, filename: &str, device: &wgpu::Device, queue: &wgpu::Queue, pal_img: image::DynamicImage ) -> Result<(), Error> {
        // load data from file
        let mut contents = String::new();
        {
            let mut file = File::open(filename)?;
            file.read_to_string(&mut contents)?;
        }

        // convert string to toml
        let tab = contents.parse::<Table>().unwrap();

        for (key, val) in tab.into_iter() {
            if let Value::Table(sub_tab) = val {
                if let Some(type_val) = sub_tab.get( &String::from("type") ) {
                    if !type_val.is_str() {
                        continue; // good enough for now
                    }
                    if type_val.as_str().unwrap() == "block" {
                        // load textures
                        let n_arr: Vec<Value> = sub_tab.get("texture_names").expect("Error: Block def lacking texture names").as_array().expect("Error: Block def texture names was not valid array").to_vec();
                        let n_arr: Vec<u32> = n_arr.into_iter().filter_map( |the_val: Value| -> Option<u32> {
                            let tname = the_val.as_str().expect("Error: non-string value found in block texture name array");
                            self.load_texture_check(tname.to_string().clone(), device, queue, &pal_img).ok()
                        } ).collect();
                        let t_arr = n_arr.clone();
                        // load shape name
                        let s_name = sub_tab.get("shape_name").expect("Error: Block def lacking shape name").as_str().expect("Error: Block def shape name was not valid string");
                        // load name
                        let temp_name = sub_tab.get("pretty_name").expect("Error: Block def lacking ingame name").as_str().expect("Error: Block def name was not valid string");
                        let p_name = String::from(temp_name).to_owned();

                        // register in registry!
                        let idx = self.br.add( *self.shape_hm.get( &String::from(s_name) ).expect("Error: Block def had invalid shape name"), p_name, t_arr );
                        self.result.insert( key, idx );
                    }


                } 
            }
        }

        Ok(())
    }

    fn load_texture_check(&mut self, texture_file: String, device: &wgpu::Device, queue: &wgpu::Queue, pal_img: &image::DynamicImage) -> Result<u32, Error> {
        let check = self.tex_hm.get( &texture_file );

        if let Some(id) = check {
            Ok(*id)
        } else {
            let long_addr = format!( "/res/texture/block/{}", &texture_file );
            let mut bytes = Vec::<u8>::new();
            {
                let mut file = File::open(long_addr)?;
                file.read_to_end(&mut bytes)?;
            }
            let image = image::load_from_memory(&bytes).unwrap();
            let texture = crate::wctx::texture::Texture::from_image_palettize(device, queue, &image, pal_img, None).unwrap();
            Ok( self.atlas.add_texture(&texture, device, queue).unwrap() )
        }
    }

}
