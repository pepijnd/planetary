use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod camera;
pub mod graphics;
pub mod render;
mod resources;

pub use crate::resources::{shaders, textures};

pub trait Runner {
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
    fn input(&mut self, event: RunnerEvent, cf: &mut ControlFlow);
    fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor);
    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        delta: std::time::Duration,
    );
    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    );
}

pub enum RunnerEvent<'a> {
    Window(&'a WindowEvent<'a>),
    Device(&'a DeviceEvent)
}

pub fn run<T>() -> Result<(), Box<dyn std::error::Error>>
where
    T: Runner + 'static,
{
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut frame_time = std::time::Instant::now();

    let mut renderer = block_on(render::RenderState::new(&window));

    crate::resources::load(&renderer.device, &renderer.queue)?;

    let mut runner = T::build(
        &window,
        &renderer.device,
        &renderer.queue,
        &renderer.sc_desc,
    );

    event_loop.run(move |event, _, control_flow| {
        runner.global_event(&event, &window, control_flow);
        match event {
            Event::DeviceEvent { event, .. } => {
                runner.input(RunnerEvent::Device(&event), control_flow)
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size, &mut runner);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size, &mut runner);
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
                    runner.input(RunnerEvent::Window(event), control_flow);
                }
            },
            Event::RedrawRequested(_) => {
                let delta = frame_time.elapsed();
                frame_time = std::time::Instant::now();
                runner.update(
                    &window,
                    &renderer.device,
                    &renderer.queue,
                    &renderer.sc_desc,
                    delta,
                );
                match renderer.render(&window, &mut runner) {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => renderer.resize(renderer.size, &mut runner),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
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
