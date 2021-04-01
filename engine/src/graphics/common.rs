use std::convert::TryInto;
use std::sync::{atomic::AtomicUsize, Arc};

use crevice::std140::Std140;
use parking_lot::{RwLock, RwLockReadGuard};
use wgpu::SwapChainDescriptor;
use winit::dpi::PhysicalSize;

use crate::graphics::helper::create_buffer_size;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }
}

impl From<PhysicalSize<u32>> for Size {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self::new(size.width, size.height)
    }
}

#[derive(Debug)]
pub struct PipelineSettings<'a> {
    pub layouts: &'a [&'a wgpu::BindGroupLayout],
    pub buffers: &'a [wgpu::VertexBufferLayout<'a>],
    pub topology: wgpu::PrimitiveTopology,
    pub samples: u32,
}

impl Default for PipelineSettings<'_> {
    fn default() -> Self {
        Self {
            layouts: &[],
            buffers: &[],
            topology: wgpu::PrimitiveTopology::TriangleList,
            samples: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineFormat {
    pub format: wgpu::TextureFormat,
}

impl From<&SwapChainDescriptor> for PipelineFormat {
    fn from(sc_desc: &SwapChainDescriptor) -> Self {
        Self {
            format: sc_desc.format,
        }
    }
}

#[derive(Debug)]
pub struct UniformBinding<T>
where
    T: crevice::std140::AsStd140,
{
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub binding: wgpu::BindGroup,
    _t: std::marker::PhantomData<T>,
}

impl<T> UniformBinding<T>
where
    T: crevice::std140::AsStd140,
{
    pub fn new(
        buffer: wgpu::Buffer,
        layout: wgpu::BindGroupLayout,
        binding: wgpu::BindGroup,
    ) -> Self {
        Self {
            buffer,
            layout,
            binding,
            _t: std::marker::PhantomData,
        }
    }
    pub fn update(&self, queue: &wgpu::Queue, data: T) {
        queue.write_buffer(&self.buffer, 0, data.as_std140().as_bytes())
    }
}

#[derive(Debug)]
pub struct TextureBinding {
    pub layout: TextureLayout,
    pub binding: wgpu::BindGroup,
}

impl TextureBinding {
    pub fn new(layout: TextureLayout, binding: wgpu::BindGroup) -> Self {
        Self { layout, binding }
    }
}

#[derive(Debug)]
pub struct TextureLayout {
    pub layout: wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub struct ItemBufferInner {
    buffer: RwLock<wgpu::Buffer>,
    num_items: AtomicUsize,
    generation: AtomicUsize,
    label: Option<String>,
    usage: wgpu::BufferUsage,
}

#[derive(Debug, Clone)]
pub struct ItemBuffer<T>
where
    T: bytemuck::Pod,
{
    pub inner: Arc<ItemBufferInner>,
    _t: std::marker::PhantomData<T>,
}

impl<T> ItemBuffer<T>
where
    T: bytemuck::Pod,
{
    pub fn new(
        buffer: wgpu::Buffer,
        num_items: usize,
        usage: wgpu::BufferUsage,
        label: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            inner: Arc::new(ItemBufferInner {
                buffer: RwLock::new(buffer),
                num_items: AtomicUsize::new(num_items),
                generation: AtomicUsize::default(),
                label: label.map(|s| s.as_ref().to_owned()),
                usage,
            }),
            _t: std::marker::PhantomData,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        let num_items = self
            .inner
            .num_items
            .load(std::sync::atomic::Ordering::Acquire);
        if data.len() > num_items || data.len() < num_items / 2 {
            let buffer = create_buffer_size::<T, _>(
                device,
                data.len(),
                self.inner.usage,
                self.inner.label.as_ref(),
            );
            let mut lock = self.inner.buffer.write();
            *lock = buffer;
            self.inner
                .generation
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        self.inner
            .num_items
            .store(data.len(), std::sync::atomic::Ordering::SeqCst);
        queue.write_buffer(&self.inner.buffer.read(), 0, bytemuck::cast_slice(data));
    }

    pub fn buffer(&self) -> RwLockReadGuard<wgpu::Buffer> {
        self.inner.buffer.read()
    }

    pub fn num_items(&self) -> usize {
        self.inner
            .num_items
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn id(&self) -> usize {
        self.inner
            .generation
            .load(std::sync::atomic::Ordering::Acquire)
    }
}

pub trait BundleData {
    type Data;
    type Id: PartialEq + Default;
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, ico: &Self::Data);
    fn id(&self) -> Self::Id;
}

pub enum RendererInvalid {
    Pipeline,
    Bundle,
}

pub trait Pipeline {
    type Settings;
    type Data: BundleData;

    fn build(device: &wgpu::Device, settings: &Self::Settings) -> Self;
    fn build_pipeline(
        &self,
        device: &wgpu::Device,
        format: PipelineFormat,
        samples: u32,
    ) -> wgpu::RenderPipeline;

    fn build_bundle(
        &self,
        device: &wgpu::Device,
        pipeline: &wgpu::RenderPipeline,
        format: PipelineFormat,
        samples: u32,
        data: &Self::Data,
    ) -> wgpu::RenderBundle;
}

pub struct Renderer<P>
where
    P: Pipeline,
{
    pub pipeline: wgpu::RenderPipeline,
    pub bundle: wgpu::RenderBundle,
    pipeline_valid: bool,
    bundle_valid: bool,
    pub format: PipelineFormat,
    pub renderer: P,
    pub data: P::Data,
    pub id: <<P as Pipeline>::Data as BundleData>::Id,
}

impl<P> Renderer<P>
where
    P: Pipeline,
{
    pub fn new(
        settings: &P::Settings,
        device: &wgpu::Device,
        format: PipelineFormat,
        samples: u32,
        data: P::Data,
    ) -> Self {
        let renderer = P::build(device, settings);
        let pipeline = renderer.build_pipeline(device, format, samples);
        let bundle = renderer.build_bundle(device, &pipeline, format, samples, &data);
        let id = Default::default();
        Self {
            pipeline,
            bundle,
            pipeline_valid: true,
            bundle_valid: true,
            renderer,
            format,
            data,
            id,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, samples: u32) {
        let id = self.data.id();
        if self.id != id {
            self.id = id;
            self.bundle_valid = false;
        }
        if !self.pipeline_valid {
            self.pipeline = self.renderer.build_pipeline(device, self.format, samples);
            self.pipeline_valid = true;
            self.bundle_valid = false;
        }
        if !self.bundle_valid {
            self.bundle = self.renderer.build_bundle(
                device,
                &self.pipeline,
                self.format,
                samples,
                &self.data,
            );
            self.bundle_valid = true;
        }
    }

    pub fn invalid(&mut self, invalid: RendererInvalid) {
        match invalid {
            RendererInvalid::Pipeline => self.pipeline_valid = false,
            RendererInvalid::Bundle => self.bundle_valid = false,
        }
    }
}

impl ItemBuffer<u32> {
    pub async fn mapped_read(&self, device: &wgpu::Device, mut offset: wgpu::BufferAddress) -> u32 {
        let bits = (offset as usize % 2) * 4;
        offset = (offset / 2) * 4;

        let buffer = self.buffer();
        let buffer_slice = buffer.slice(offset..offset + 8);
        let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);
        mapping.await.unwrap();
        let data = buffer_slice.get_mapped_range();
        u32::from_le_bytes(data[bits..bits + 4].try_into().unwrap())
    }
}
