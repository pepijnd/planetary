use std::sync::Arc;

use parking_lot::Mutex;
use winit::window::Window;

use crate::{MainRunner, Size, ThreadRunner};

pub struct RenderTarget {
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub target: Arc<Mutex<RenderTarget>>,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        Self {
            sc_desc: sc_desc.clone(),
            swap_chain,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, surface: &wgpu::Surface, size: Size) {
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.rebuild(device, surface);
    }

    pub fn size(&self) -> Size {
        Size::new(self.sc_desc.width, self.sc_desc.height)
    }

    pub fn sc_desc(&self) -> &wgpu::SwapChainDescriptor {
        &self.sc_desc
    }

    pub fn frame(&self) -> Result<wgpu::SwapChainFrame, wgpu::SwapChainError> {
        self.swap_chain.get_current_frame()
    }

    pub fn rebuild(&mut self, device: &wgpu::Device, surface: &wgpu::Surface) {
        self.swap_chain = device.create_swap_chain(surface, &self.sc_desc);
    }
}

impl RenderState {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("render_state_device"),
                    features: wgpu::Features::TEXTURE_COMPRESSION_BC,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        let target = RenderTarget::new(&device, &surface, &sc_desc);
        let target = Arc::new(Mutex::new(target));

        Self {
            surface,
            device,
            queue,
            target,
        }
    }

    pub fn resize<T>(self: &Arc<Self>, size: Size, runner: Arc<Mutex<T>>)
    where
        T: ThreadRunner,
    {
        let mut target = self.target.lock();
        let mut runner = runner.lock();
        target.resize(&self.device, &self.surface, size);
        runner.resize(&self.device, target.size());
    }

    pub fn render<T, M>(
        self: &Arc<Self>,
        window: &winit::window::Window,
        thread_runner: Arc<Mutex<T>>,
        runner: &mut M,
    ) -> Result<(), wgpu::SwapChainError>
    where
        T: ThreadRunner,
        M: MainRunner,
    {
        let target = self.target.lock();
        let frame = target.frame()?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });
        {
            let mut thread_runner = thread_runner.lock();
            thread_runner.render(
                &self.device,
                &self.queue,
                &target,
                &frame.output.view,
                &mut encoder,
                window,
            );
        }

        runner.render(
            &self.device,
            &self.queue,
            &target,
            &frame.output.view,
            &mut encoder,
            window,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
