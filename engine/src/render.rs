use winit::window::Window;

use crate::Runner;

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
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

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
        }
    }

    pub fn resize<T>(&mut self, new_size: winit::dpi::PhysicalSize<u32>, runner: &mut T)
    where
        T: Runner,
    {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        runner.resize(&self.device, &self.sc_desc);
    }

    pub fn render<T>(
        &mut self,
        window: &winit::window::Window,
        runner: &mut T,
    ) -> Result<(), wgpu::SwapChainError>
    where
        T: Runner,
    {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });
        runner.render(
            &frame,
            &self.device,
            &self.queue,
            &self.sc_desc,
            &mut encoder,
            window,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
