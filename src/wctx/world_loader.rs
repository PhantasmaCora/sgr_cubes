 
use std::io::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::{
    Write,
    Read
};

use toml::Table;

use figures::units::UPx;

use crate::wctx::world;

#[derive(Clone)]
pub struct WorldPreview {
    pub path_name: String,
    pub name: String,
    pub size: usize,
}


impl WorldPreview {
    pub fn load_world(&self) -> Result<world::WorldSavestate, Error> {
        let mut pbuf = PathBuf::new();
        pbuf.push( &self.path_name );
        pbuf.push("world_savestate.pkl");

        let mut fi = std::fs::File::open(pbuf)?;
        let mut bytes = Vec::<u8>::new();
        fi.read_to_end(&mut bytes)?;

        let deserialized: world::WorldSavestate = serde_pickle::from_slice(&bytes, Default::default()).unwrap();
        Ok(deserialized)

    }

}



pub struct WorldLoader {
    pub previews: Vec<WorldPreview>,
    pub name_map: HashMap<String, usize>,
}

impl WorldLoader {

    pub fn load_previews(&mut self, gfx: &cushy::kludgine::Graphics ) -> Vec<(WorldPreview, cushy::kludgine::Texture)> {
        self.previews.clear();

        let pdirs = directories::ProjectDirs::from( "", "PhantasmaCora Games", "SGR_Cubes" ).expect("Failed to get project directories");
        let mut pbuf = pdirs.data_dir().to_path_buf();
        pbuf.push( "worlds/" );

        let mut texes = Vec::<cushy::kludgine::Texture>::new();

        let d_info = fs_extra::dir::get_dir_content2(pbuf.clone(), &fs_extra::dir::DirOptions{ depth: 1_u64 }).unwrap();

        for subd in d_info.directories {
            let mut lbuf = PathBuf::new();
            lbuf.push(subd.clone());
            lbuf.push("info.toml");
            let mut file = File::open(lbuf);
            if let Ok(mut fi) = file {
                let mut contents = String::new();
                let read_result = fi.read_to_string(&mut contents);
                if let Ok(size) = read_result {
                    let table_unwrap = contents.parse::<Table>();

                    if let Ok(tab) = table_unwrap {
                        let mut texture = cushy::kludgine::Texture::new(
                            gfx,
                            figures::Size{width: UPx::new(512), height: UPx::new(512)},
                            wgpu::TextureFormat::Rgba8UnormSrgb,
                            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                            wgpu::FilterMode::Linear
                        );

                        let mut ibuf = PathBuf::new();
                        ibuf.push(subd.clone());
                        ibuf.push("preview.png");
                        let mut ifile = File::open(ibuf);
                        if let Ok(mut fi) = ifile {
                            let mut image_data = Vec::<u8>::new();
                            let read_result = fi.read_to_end(&mut image_data);
                            if let Ok(size2) = read_result {
                                let texture2 = crate::wctx::texture::Texture::from_bytes(gfx.device(), gfx.queue(), &image_data, "copy texture").unwrap();
                                let mut encoder = gfx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("TexCopy Encoder"),
                                });
                                encoder.copy_texture_to_texture(
                                    wgpu::ImageCopyTexture{
                                        texture: &texture2.texture,
                                        mip_level: 0,
                                        origin: wgpu::Origin3d{x: 0, y: 0, z: 0},
                                        aspect: wgpu::TextureAspect::All
                                    },
                                    wgpu::ImageCopyTexture{
                                        texture: &texture.wgpu(),
                                        mip_level: 0,
                                        origin: wgpu::Origin3d{x: 0, y: 0, z: 0},
                                        aspect: wgpu::TextureAspect::All
                                    },
                                    texture2.texture.size()
                                );
                                gfx.queue().submit( std::iter::once( encoder.finish() ) );
                            }
                        }

                        let preview = WorldPreview{
                            path_name: subd,
                            name: tab["name"].as_str().expect("name must be a string!").to_string(),
                            size: tab["size"].as_integer().unwrap().try_into().expect("size should only be 0, 1, 2"),
                        };

                        self.name_map.insert( preview.name.clone(), self.previews.len() );
                        self.previews.push(preview);
                        texes.push(texture);

                    }
                }
            }
        }

        let mut ret = Vec::<(WorldPreview, cushy::kludgine::Texture)>::new();
        let mut pvclon = self.previews.clone();
        for idx in 0..texes.len() {
            ret.push( ( pvclon.pop().unwrap(), texes.pop().unwrap() ) )
        }
        ret
    }


}
