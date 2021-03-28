use editor::Editor;
use engine::Runner;

pub mod editor;
pub mod pipelines;
pub mod structures;
pub mod ui;

impl Runner for Editor {
    fn build(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        Self::new(window, device, queue, sc_desc)
    }

    fn global_event(
        &mut self,
        event: &winit::event::Event<()>,
        window: &winit::window::Window,
        _cf: &mut winit::event_loop::ControlFlow,
    ) {
        self.ui.handle_event(window, event)
    }

    fn input(
        &mut self,
        event: engine::RunnerEvent,
        _cf: &mut winit::event_loop::ControlFlow,
    ) {
        self.input(event);
    }

    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.resize(device, sc_desc)
    }

    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        delta: std::time::Duration,
    ) {
        self.update(device, queue, sc_desc, window, delta)
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    ) {
        self.render(frame, device, queue, sc_desc, encoder, window)
    }
}

fn main() -> Result<(), std::boxed::Box<(dyn std::error::Error)>> {
    engine::run::<Editor>()
}
