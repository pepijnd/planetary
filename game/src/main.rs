use std::{sync::Arc, time::Duration};

use editor::{Editor, MainGameThread};

use engine::{
    event::RunnerEvent, parking_lot::Mutex, render::RenderTarget, wgpu, winit, MainRunner, Size,
    ThreadRunner,
};

use ui::EditorUi;

pub mod editor;
pub mod pipelines;
pub mod structures;
pub mod ui;

impl MainRunner for MainGameThread {
    type Runner = Editor;

    fn build(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        runner: Arc<Mutex<Self::Runner>>,
    ) -> Self {
        let ui = {
            let mut runner = runner.lock();
            let state = &mut runner.state;
            EditorUi::new(window, device, queue, sc_desc, state)
        };
        Self { ui, runner }
    }

    fn global_event(
        &mut self,
        event: &winit::event::Event<RunnerEvent>,
        window: &winit::window::Window,
        _cf: &mut winit::event_loop::ControlFlow,
    ) {
        self.ui.handle_event(window, event)
    }

    fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        let mut runner = self.runner.lock();
        self.ui.resize(&device, size, &mut runner.state);
    }

    fn update(
        &mut self,
        window: &winit::window::Window,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> u32 {
        self.ui.update(window, self.runner.lock().delta);
        self.runner.lock().state.target_fps
    }
    fn input(&mut self, _event: engine::event::RunnerEvent) {}

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _target: &RenderTarget,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    ) {
        let mut runner = self.runner.lock();
        self.ui
            .render(&mut runner.state, frame, encoder, queue, device, window)
    }
}

impl ThreadRunner for Editor {
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
        _event: &winit::event::Event<()>,
        _window: &winit::window::Window,
        _cf: &mut winit::event_loop::ControlFlow,
    ) {
    }

    fn input(&mut self, event: RunnerEvent) {
        self.input(event);
    }

    fn resize(&mut self, device: &wgpu::Device, size: Size) {
        self.resize(device, size)
    }

    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: (f32, Duration),
    ) -> u32 {
        self.state.tick_rate = delta.0;
        self.state.tick_time = delta.1;
        self.update(device, queue, window);
        self.state.target_tick_rate
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &RenderTarget,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    ) {
        self.render(device, queue, target, frame, encoder, window)
    }
}

fn main() -> Result<(), std::boxed::Box<(dyn std::error::Error)>> {
    engine::run::<MainGameThread>()
}
