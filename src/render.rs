use winit::event::WindowEvent;
use winit::window::Window;

use crate::{editor::Editor, graphics::shaders::shader_add};

use wgpu::include_spirv;

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    editor: Editor,
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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        shader_add(
            &device,
            "ico.vert",
            &include_spirv!("../data/shaders/ico.vert.spv"),
        );
        shader_add(
            &device,
            "ico.frag",
            &include_spirv!("../data/shaders/ico.frag.spv"),
        );
        shader_add(
            &device,
            "ico.select.frag",
            &include_spirv!("../data/shaders/ico_select.frag.spv"),
        );

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let editor = Editor::new(&window, &device, &queue, &sc_desc);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            editor,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.editor.resize(&self.device, &self.sc_desc);
    }
    pub fn handle_event(
        &mut self,
        event: &winit::event::Event<()>,
        window: &winit::window::Window,
    ) {
        self.editor.handle_event(window, event)
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.editor.input(event)
    }

    pub fn update(&mut self, window: &winit::window::Window, delta: std::time::Duration) {
        self.editor
            .update(&self.device, &self.queue, &self.sc_desc, window, delta);
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });
        self.editor.render(
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
