use crate::Size;

use super::common::ItemBuffer;

use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub view_dimension: wgpu::TextureViewDimension,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsage,
    pub dimension: wgpu::TextureDimension,
    pub sampler: wgpu::Sampler,
    pub samples: u32,
    pub label: Option<String>,
}

pub struct TextureDescriptor {
    pub size: wgpu::Extent3d,
    pub dimension: wgpu::TextureDimension,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsage,
    pub samples: u32,
    pub levels: u32,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_texture(
        device: &wgpu::Device,
        desc: &TextureDescriptor,
        label: Option<impl AsRef<str>>,
    ) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_ref().map(|s| s.as_ref()),
            size: desc.size,
            mip_level_count: desc.levels,
            sample_count: desc.samples,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let view_dimension = if desc.size.depth > 1 {
            wgpu::TextureViewDimension::D2Array
        } else {
            wgpu::TextureViewDimension::D2
        };

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
            view_dimension,
            size: desc.size,
            sampler,
            samples: desc.samples,
            format: desc.format,
            usage: desc.usage,
            dimension: desc.dimension,
            label: label.map(|s| s.as_ref().to_owned()),
        }
    }

    pub fn create_texture_with_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        desc: &TextureDescriptor,
        label: Option<impl AsRef<str>>,
    ) -> Texture {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: label.as_ref().map(|s| s.as_ref()),
                size: desc.size,
                mip_level_count: desc.levels,
                sample_count: desc.samples,
                dimension: desc.dimension,
                format: desc.format,
                usage: desc.usage,
            },
            data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let view_dimension = if desc.size.depth > 1 {
            wgpu::TextureViewDimension::D2Array
        } else {
            wgpu::TextureViewDimension::D2
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            view_dimension,
            size: desc.size,
            sampler,
            samples: desc.samples,
            format: desc.format,
            usage: desc.usage,
            dimension: desc.dimension,
            label: label.map(|s| s.as_ref().to_owned()),
        }
    }

    pub fn write_data(&self, queue: &wgpu::Queue, data: &[u8]) {
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

    pub fn with_size(&self, device: &wgpu::Device, size: Size) -> Self {
        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let label = self.label.clone();
        Self::create_texture(
            &device,
            &TextureDescriptor {
                size,
                dimension: self.dimension,
                format: self.format,
                usage: self.usage,
                samples: self.samples,
                levels: 1,
            },
            label,
        )
    }

    pub fn with_samples(&mut self, device: &wgpu::Device, samples: u32) -> Self {
        let label = self.label.take();
        Self::create_texture(
            device,
            &TextureDescriptor {
                size: self.size,
                dimension: self.dimension,
                format: self.format,
                usage: self.usage,
                samples,
                levels: 1,
            },
            label,
        )
    }

    pub fn make_buffer(&self, device: &wgpu::Device, usage: wgpu::BufferUsage) -> ItemBuffer<u32> {
        let width = {
            let align = 256 / std::mem::size_of::<u32>();
            let offset = self.size.width as usize % align;
            if offset == 0 {
                self.size.width as usize
            } else {
                self.size.width as usize / align * align + align
            }
        } as u32;
        let items = width * self.size.height;
        let buffer = crate::graphics::helper::create_buffer_size::<u32, _>(
            device,
            items as usize,
            usage,
            self.label.as_ref(),
        );
        crate::graphics::common::ItemBuffer::new(buffer, items as usize, usage, self.label.as_ref())
    }

    pub fn depth(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        samples: u32,
        label: Option<impl AsRef<str>>,
    ) -> Self {
        Self::create_texture(
            device,
            &TextureDescriptor {
                size,
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
                samples,
                levels: 1,
            },
            label,
        )
    }

    pub fn msaa(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        samples: u32,
        label: Option<impl AsRef<str>>,
    ) -> Self {
        Self::create_texture(
            device,
            &TextureDescriptor {
                size,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                samples,
                levels: 1,
            },
            label,
        )
    }

    pub fn select(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        label: Option<impl AsRef<str>>,
    ) -> Self {
        Self::create_texture(
            device,
            &TextureDescriptor {
                size,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Uint,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC,
                samples: 1,
                levels: 1,
            },
            label,
        )
    }
}
