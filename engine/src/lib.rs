#![feature(duration_consts_2)]
#![feature(duration_saturating_ops)]

use std::{collections::VecDeque, sync::Arc, time::Duration};

use futures::executor::block_on;
use inputs::spawn_input_thread;
use parking_lot::{Condvar, Mutex};
use render::RenderTarget;
use updates::spawn_update_thread;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod camera;
pub mod clock;
pub mod event;
pub mod graphics;
pub mod inputs;
pub mod render;
pub mod resources;
pub mod updates;

pub use crate::{
    graphics::common::Size,
    resources::{shaders, textures},
};

use event::{RunnerEvent, WindowEvent};

pub use num_traits;
pub use palette;
pub use parking_lot;
pub use wgpu;
pub use winit;

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
        event: &Event<RunnerEvent>,
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
    ) -> u32;
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
    fn resize(&mut self, device: &wgpu::Device, size: Size);
    fn update(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        delta: (f32, Duration),
    ) -> u32;
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

pub fn run<T>() -> Result<(), Box<dyn std::error::Error>>
where
    T: MainRunner + 'static,
{
    env_logger::init();
    let event_loop = EventLoop::with_user_event();
    let event_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut frame_time = std::time::Duration::from_secs_f32(1.0 / 60.0);
    let mut fps = 60.0;
    let mut clock = clock::Clock::new(60);

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

    let window = Arc::new(window);
    let queue = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

    spawn_input_thread(Arc::clone(&queue), Arc::clone(&thread_runner));
    spawn_update_thread(
        Arc::clone(&thread_runner),
        Arc::clone(&renderer),
        Arc::clone(&window),
    );

    event_loop.run(move |event, _, control_flow| {
        runner.global_event(&event, &window, control_flow);
        match event {
            Event::DeviceEvent { event, .. } => {
                let (lock, cvar) = &*queue;
                lock.lock().push_back(RunnerEvent::Device(event));
                cvar.notify_one();
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                let event: event::WindowEvent = event.into();
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        let runner = Arc::clone(&thread_runner);
                        let renderer = Arc::clone(&renderer);
                        renderer.resize(size, runner);
                    }
                    WindowEvent::ScaleFactorChanged { size, .. } => {
                        let runner = Arc::clone(&thread_runner);
                        let renderer = Arc::clone(&renderer);
                        renderer.resize(size, runner);
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
                        let (lock, cvar) = &*queue;
                        lock.lock().push_back(RunnerEvent::Window(event));
                        cvar.notify_one();
                    }
                }
            }
            Event::UserEvent(event) => {
                let (lock, cvar) = &*queue;
                lock.lock().push_back(event);
                cvar.notify_one();
            }
            Event::RedrawRequested(_) => {
                clock.tick();

                clock.target_rate = runner.update(&window, &renderer.device, &renderer.queue);
                match renderer.render(&window, Arc::clone(&thread_runner), &mut runner) {
                    Ok(_) => {
                        event_proxy
                            .send_event(RunnerEvent::RenderComplete {
                                frame_time,
                                tick_rate: fps,
                            })
                            .unwrap();
                        let (tick_rate, curr_time) = clock.wait();
                        frame_time = curr_time;
                        fps = tick_rate;
                    }
                    Err(wgpu::SwapChainError::Lost) => {
                        let size = {
                            let target = renderer.target.lock();
                            target.size()
                        };
                        renderer.resize(size, Arc::clone(&thread_runner));
                    }
                    Err(wgpu::SwapChainError::OutOfMemory) => {}
                    Err(e) => log::warn!("{}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
