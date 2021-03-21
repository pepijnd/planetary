use anyhow::*;
use image::{EncodableLayout, GenericImageView};

use super::common::ItemBuffer;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsage,
    pub dimension: wgpu::TextureDimension,
    pub sampler: wgpu::Sampler,
    pub samples: u32,
    pub label: Option<String>,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<impl AsRef<str>>,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, &img, label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<impl AsRef<str>>,
    ) -> Self {
        let data = img.to_rgba8();
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let usage = wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST;
        let dimension = wgpu::TextureDimension::D2;
        let texture = Self::create_texture(device, size, dimension, format, usage, 1, label);
        texture.write_image(queue, data.as_bytes());
        texture
    }

    pub fn create_texture(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        dimension: wgpu::TextureDimension,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsage,
        samples: u32,
        label: Option<impl AsRef<str>>,
    ) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_ref().map(|s| s.as_ref()),
            size,
            mip_level_count: 1,
            sample_count: samples,
            dimension,
            format,
            usage,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            size,
            sampler,
            samples,
            format,
            usage,
            dimension,
            label: label.map(|s| s.as_ref().to_owned()),
        }
    }

    pub fn write_image(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &data,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * self.size.width,
                rows_per_image: self.size.height,
            },
            self.size,
        );
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        let size = wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth: 1,
        };
        let label = self.label.take();
        *self = Self::create_texture(
            device,
            size,
            self.dimension,
            self.format,
            self.usage,
            self.samples,
            label,
        )
    }

    pub fn with_samples(&mut self, device: &wgpu::Device, samples: u32) {
        let label = self.label.take();
        *self = Self::create_texture(
            device,
            self.size,
            self.dimension,
            self.format,
            self.usage,
            samples,
            label,
        )
    }

    pub fn make_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsage) -> ItemBuffer<u32> {
        let items = self.size.width * self.size.height;
        let buffer = crate::graphics::helper::create_buffer_size::<u32, _>(device, items as usize, usage, self.label.as_ref());
        crate::graphics::common::ItemBuffer::new(buffer, items as usize, usage, self.label.as_ref())
    }
}
