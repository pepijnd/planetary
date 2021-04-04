use std::sync::Arc;

use engine::{num_traits::float::FloatConst, parking_lot::Mutex, wgpu::TextureFormat};

use engine::{
    camera::Camera,
    event::{RunnerEvent, WindowEvent},
    graphics::{
        common::{BundleData, ItemBuffer, PipelineFormat, Renderer, RendererInvalid},
        helper::begin_render_pass,
        texture::Texture,
    },
    palette,
    render::RenderTarget,
    wgpu, winit,
    winit::event::{ElementState, KeyboardInput, VirtualKeyCode},
    MainRunner, Size,
};

use crate::{
    pipelines::ico::{IcoBuffer, IcoRenderer, IcoRendererSettings, IcoUniform},
    structures::ico::Ico,
    ui::{EditorState, EditorUi},
};

pub struct MainGameThread {
    pub ui: EditorUi,
    pub runner: Arc<Mutex<<Self as MainRunner>::Runner>>,
}

pub struct Editor {
    pub camera: Camera,
    pub size: Size,

    pub ico: Ico,
    pub ico_buffer: IcoBuffer,
    pub ico_screen: Renderer<IcoRenderer>,
    pub ico_select: Renderer<IcoRenderer>,

    pub ico_uniform: IcoUniform,

    pub msaa: Texture,
    pub depth_texture: Texture,
    pub sampled_depth_texture: Texture,
    pub select: Texture,
    pub select_buffer: ItemBuffer<u32>,
    pub selected: u32,

    pub modifiers: winit::event::ModifiersState,
    pub state: EditorState,

    pub last_frame: std::time::Instant,
    pub delta: std::time::Duration,

    pub mouse_raw: [u32; 2],
    pub mouse_last: glam::Vec2,
    pub mouse_pos: glam::Vec2,
    pub mouse_pressed: bool,

    pub rotating: f32,
}

impl Editor {
    pub fn new(
        _window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let state = EditorState::default();

        let camera = Camera::new(sc_desc, f32::FRAC_PI_2() / 2.0, *state.zoom as f32);
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let depth_texture = Texture::depth(device, size, 1, Some("depth_texture"));
        let sampled_depth_texture = Texture::depth(
            device,
            size,
            *state.samples as u32,
            Some("depth_texture_sampled"),
        );
        let msaa = Texture::msaa(
            device,
            size,
            *state.samples as u32,
            Some("depth_texture_sampled"),
        );
        let select = Texture::select(device, size, Some("depth_texture"));

        let select_buffer = select.make_buffer(
            device,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        );

        let size = Size::new(sc_desc.width, sc_desc.height);

        let mut ico_buffer = IcoBuffer::build(device);
        let ico = Ico::divs(*state.size as usize);
        ico_buffer.update(device, queue, &ico);

        let ico_screen: Renderer<IcoRenderer> = Renderer::new(
            &IcoRendererSettings {
                vs: "shader.ico.vert",
                fs: "shader.ico.frag",
            },
            device,
            sc_desc.into(),
            *state.samples as u32,
            ico_buffer.clone(),
        );

        let ico_select: Renderer<IcoRenderer> = Renderer::new(
            &IcoRendererSettings {
                vs: "shader.ico.vert",
                fs: "shader.ico.select.frag",
            },
            device,
            PipelineFormat {
                format: TextureFormat::R32Uint,
            },
            1,
            ico_buffer.clone(),
        );

        let ico_uniform = IcoUniform {
            view_proj: camera.build(*state.perspective).into(),
            view_pos: (camera.rot * -camera.zoom).into(),
            light_pos: glam::vec3(-5.0, -5.0, -10.0).into(),
            selected: 0,
            s1: 0,
            s2: 0,
            s3: 0,
        };

        let modifiers = winit::event::ModifiersState::default();

        let mouse_raw = [0; 2];
        let mouse_last = [0.0; 2].into();
        let mouse_pos = [0.0; 2].into();
        let mouse_pressed = false;

        Self {
            camera,
            size,

            ico,
            ico_screen,
            ico_select,
            ico_uniform,
            ico_buffer,

            msaa,
            depth_texture,
            sampled_depth_texture,

            select,
            select_buffer,
            selected: 0,

            state,
            modifiers,

            delta: std::time::Duration::from_secs_f32(1.0 / 60.0),
            last_frame: std::time::Instant::now(),

            mouse_raw,
            mouse_last,
            mouse_pos,
            mouse_pressed,

            rotating: 0.0,
        }
    }

    pub fn input(&mut self, event: RunnerEvent) -> bool {
        match event {
            RunnerEvent::Window(event) => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos = [
                        (2.0 * position.0 as f32 / self.size.width as f32) - 1.0,
                        (2.0 * position.1 as f32 / self.size.height as f32) - 1.0,
                    ]
                    .into();
                    self.mouse_raw = [position.0 as u32, position.1 as u32];
                    true
                }
                WindowEvent::MouseInput {
                    state: winit::event::ElementState::Pressed,
                    button: winit::event::MouseButton::Left,
                    ..
                } => {
                    self.mouse_last = self.mouse_pos;
                    self.mouse_pressed = true;
                    true
                }
                WindowEvent::MouseWheel {
                    delta: winit::event::MouseScrollDelta::LineDelta(_, scroll),
                    phase: winit::event::TouchPhase::Moved,
                    ..
                } => {
                    *self.state.zoom += scroll * -0.1;
                    if *self.state.zoom < 0.1 {
                        *self.state.zoom = 0.1
                    }
                    if *self.state.zoom > 2.0 {
                        *self.state.zoom = 2.0
                    }
                    true
                }
                WindowEvent::MouseInput {
                    state: winit::event::ElementState::Released,
                    button: winit::event::MouseButton::Left,
                    ..
                } => {
                    self.mouse_pressed = false;
                    true
                }
                WindowEvent::ModifiersChanged(m) => {
                    self.modifiers = m;
                    true
                }
                _ => false,
            },
            RunnerEvent::Device(event) => match event {
                winit::event::DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Q),
                    state: ElementState::Pressed,
                    ..
                }) => {
                    self.rotating = -1.275 * self.delta.as_secs_f32();
                    true
                }
                winit::event::DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::E),
                    state: ElementState::Pressed,
                    ..
                }) => {
                    self.rotating = 1.275 * self.delta.as_secs_f32();
                    true
                }
                winit::event::DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Q),
                    state: ElementState::Released,
                    ..
                }) => {
                    if self.rotating < 0.0 {
                        self.rotating = 0.0;
                    }
                    true
                }
                winit::event::DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::E),
                    state: ElementState::Released,
                    ..
                }) => {
                    if self.rotating > 0.0 {
                        self.rotating = 0.0;
                    }
                    true
                }
                _ => false,
            },
            RunnerEvent::RenderComplete {
                frame_time,
                tick_rate,
            } => {
                self.state.frame_time = frame_time;
                self.state.fps = tick_rate;
                false
            }
            RunnerEvent::None => false,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: Size) {
        self.size = size;
        self.msaa = self.msaa.with_size(device, self.size);
        self.depth_texture = self.depth_texture.with_size(device, self.size);
        self.sampled_depth_texture = self.sampled_depth_texture.with_size(device, self.size);
        self.select = self.select.with_size(device, self.size);

        self.select_buffer = self.select.make_buffer(
            device,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        );
        self.camera.resize(size);
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _window: &winit::window::Window,
    ) {
        if let Some(&samples) = self.state.samples.on_change() {
            self.sampled_depth_texture = self
                .sampled_depth_texture
                .with_samples(device, samples as u32);
            self.msaa = self.msaa.with_samples(device, samples as u32);

            self.ico_screen.invalid(RendererInvalid::Pipeline);
        }

        self.camera.zoom = *self.state.zoom;
        if self.mouse_pressed && !self.state.ui_io.lock().wants_mouse {
            self.camera.pan(self.mouse_pos - self.mouse_last, 2.0);
        }
        self.camera.rotate(self.rotating);
        self.mouse_last = self.mouse_pos;

        let view_proj = self.camera.build(*self.state.perspective);

        if let Some(&d) = self.state.size.on_change() {
            let ico = Ico::divs(d as usize);
            self.ico = ico;
            self.ico_buffer.update(device, queue, &self.ico);
        }

        // let index = ((sc_desc.width * self.mouse_raw[1]) + self.mouse_raw[0])
        //     .min(sc_desc.width * sc_desc.height - 1) as wgpu::BufferAddress;
        // let new = block_on(self.select_buffer.mapped_read(device, index));

        // self.select_buffer.buffer().unmap();

        self.ico_uniform.view_proj = view_proj.into();
        self.ico_uniform.view_pos = (self.camera.rot * -self.camera.zoom).into();
        self.ico_uniform.selected = self.selected;
        self.ico_screen
            .renderer
            .uniform_binding
            .update(queue, self.ico_uniform);
        self.ico_select
            .renderer
            .uniform_binding
            .update(queue, self.ico_uniform);

        self.ico_screen.update(device, *self.state.samples as u32);
        self.ico_select.update(device, 1);
    }

    pub fn render(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        target: &RenderTarget,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        _window: &winit::window::Window,
    ) {
        let color = palette::rgb::Srgb::from_components((0.53, 0.81, 0.92)).into_linear();
        let msaa = if *self.state.samples == 1 {
            None
        } else {
            Some(&self.msaa)
        };
        let size = target.size();
        {
            let mut pass = begin_render_pass(
                encoder,
                &frame,
                Some(&self.sampled_depth_texture),
                color,
                msaa,
                Some("main_render_pass"),
            );
            pass.execute_bundles(vec![&self.ico_screen.bundle].into_iter());
        }

        {
            let mut pass = begin_render_pass(
                encoder,
                &self.select.view,
                Some(&self.depth_texture),
                color,
                None,
                Some("select_render_pass"),
            );
            pass.execute_bundles(vec![&self.ico_select.bundle].into_iter());
        }

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &self.select.texture,
                mip_level: 0,
                origin: Default::default(),
            },
            wgpu::BufferCopyView {
                buffer: &self.select_buffer.buffer(),
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: (self.select_buffer.num_items() * std::mem::size_of::<u32>())
                        as u32
                        / size.height,
                    rows_per_image: size.height,
                },
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth: 1,
            },
        );
    }
}
