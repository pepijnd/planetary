use std::{borrow::Cow, collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use resources::*;
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsage};

use crate::graphics::texture::{Texture, TextureDescriptor};

lazy_static! {
    static ref TEXTURES: Arc<Mutex<HashMap<String, Texture>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref SHADERS: Arc<Mutex<HashMap<String, wgpu::ShaderModule>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn textures() -> Arc<Mutex<HashMap<String, Texture>>> {
    Arc::clone(&TEXTURES)
}

pub fn shaders() -> Arc<Mutex<HashMap<String, wgpu::ShaderModule>>> {
    Arc::clone(&SHADERS)
}

pub fn load(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("loading resources");
    let resources = resources::read()?;

    let mut buffer = Vec::new();

    for ResourceItem { label, resource } in resources {
        match resource {
            Resource::Image(image) => {
                buffer.clear();
                let size = image.read(&mut buffer)?;
                log::info!(
                    "loading texture array: {} {:?}",
                    label,
                    (image.size, image.depth, image.levels)
                );

                let texture = make_texture(
                    device,
                    queue,
                    &buffer[..size],
                    image.size,
                    image.depth,
                    image.levels,
                    &label,
                );
                TEXTURES.lock().insert(label, texture);
            }
            Resource::Shader(Shader { data }) => {
                log::info!("creating shader module {}", label);
                let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some(&label),
                    source: wgpu::ShaderSource::SpirV(Cow::from(&data)),
                    flags: wgpu::ShaderFlags::default(),
                });
                SHADERS.lock().insert(label, shader);
            }
        }
    }

    Ok(())
}

fn make_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[u8],
    size: (u32, u32),
    depth: u32,
    levels: u32,
    label: &str,
) -> Texture {
    Texture::create_texture_with_data(
        device,
        queue,
        data,
        &TextureDescriptor {
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth,
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Bc3RgbaUnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            samples: 1,
            levels,
        },
        Some(label),
    )
}
