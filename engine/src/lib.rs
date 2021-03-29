use std::sync::Arc;

use futures::executor::block_on;
use parking_lot::Mutex;
use render::RenderTarget;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod camera;
pub mod event;
pub mod graphics;
pub mod render;
mod resources;

pub use crate::resources::{shaders, textures};

use event::WindowEvent;

pub trait MainRunner {
    type Runner: ThreadRunner + Send + Sync;

    fn build(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        runner: Arc<Mutex<Self::Runner>>,
    ) -> Self;
    fn global_event(
        &mut self,
        event: &Event<()>,
        window: &winit::window::Window,
        cf: &mut ControlFlow,
    );
    fn input(&mut self, event: RunnerEvent);
    fn resize(&mut self, device: &wgpu::Device, size: (u32, u32));
    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: std::time::Duration,
    );
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &RenderTarget,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    );
}

pub trait ThreadRunner {
    fn build(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self;
    fn global_event(
        &mut self,
        event: &Event<()>,
        window: &winit::window::Window,
        cf: &mut ControlFlow,
    );
    fn input(&mut self, event: RunnerEvent);
    fn resize(&mut self, device: &wgpu::Device, size: (u32, u32));
    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: std::time::Duration,
    );
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &RenderTarget,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    );
}

pub enum RunnerEvent {
    Window(WindowEvent),
    Device(DeviceEvent),
    None,
}

pub fn run<T>() -> Result<(), Box<dyn std::error::Error>>
where
    T: MainRunner + 'static,
{
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut frame_time = std::time::Instant::now();

    let renderer = Arc::new(block_on(render::RenderState::new(&window)));

    crate::resources::load(&renderer.device, &renderer.queue)?;

    let thread_runner = {
        let target = renderer.target.lock();
        <T::Runner as ThreadRunner>::build(
            &window,
            &renderer.device,
            &renderer.queue,
            &target.sc_desc(),
        )
    };
    let thread_runner = Arc::new(Mutex::new(thread_runner));

    let mut runner = {
        let target = renderer.target.lock();
        let thread_runner = Arc::clone(&thread_runner);
        T::build(
            &window,
            &renderer.device,
            &renderer.queue,
            &target.sc_desc(),
            thread_runner,
        )
    };

    use futures::executor::ThreadPool;
    let pool = ThreadPool::new().unwrap();

    let window = Arc::new(window);

    event_loop.run(move |event, _, control_flow| {
        runner.global_event(&event, &window, control_flow);
        match event {
            Event::DeviceEvent { event, .. } => {
                let runner = Arc::clone(&thread_runner);
                pool.spawn_ok(async move {
                    let mut runner = runner.lock();
                    runner.input(RunnerEvent::Device(event))
                });
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                let event: event::WindowEvent = event.into();
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        let runner = Arc::clone(&thread_runner);
                        let renderer = Arc::clone(&renderer);
                        pool.spawn_ok(async move {
                            renderer.resize(physical_size, runner);
                        });
                    }
                    WindowEvent::ScaleFactorChanged { size, .. } => {
                        let runner = Arc::clone(&thread_runner);
                        let renderer = Arc::clone(&renderer);
                        pool.spawn_ok(async move {
                            renderer.resize(size, runner);
                        });
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    event => {
                        let runner = Arc::clone(&thread_runner);
                        pool.spawn_ok(async move {
                            let mut runner = runner.lock();
                            runner.input(RunnerEvent::Window(event))
                        });
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let delta = frame_time.elapsed();
                frame_time = std::time::Instant::now();
                {
                    let runner = Arc::clone(&thread_runner);
                    let renderer = Arc::clone(&renderer);
                    let window = Arc::clone(&window);
                    pool.spawn_ok(async move {
                        {
                            let mut runner = runner.lock();
                            runner.update(&window, &renderer.device, &renderer.queue, delta);
                        }
                    });
                }
                match renderer.render(&window, Arc::clone(&thread_runner), &mut runner) {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => {
                        let size = {
                            let target = renderer.target.lock();
                            target.size()
                        };
                        renderer.resize(size, Arc::clone(&thread_runner));
                    }
                    Err(wgpu::SwapChainError::OutOfMemory) => {}
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
