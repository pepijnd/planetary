use futures::executor::block_on;
use num_traits::float::FloatConst;
use wgpu::TextureFormat;

use engine::{
    camera::Camera,
    graphics::common::{BundleData, PipelineFormat, Renderer, RendererInvalid},
    graphics::helper::begin_render_pass,
    graphics::{common::ItemBuffer, texture::Texture},
};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    pipelines::ico::{IcoBuffer, IcoRenderer, IcoRendererSettings, IcoUniform},
    structures::ico::Ico,
    ui::{EditorState, EditorUi},
};

pub struct Editor {
    pub camera: Camera,
    pub size: [u32; 2],

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
    pub ui: EditorUi,
    pub state: EditorState,

    pub delta: std::time::Duration,

    pub mouse_raw: [u32; 2],
    pub mouse_last: glam::Vec2,
    pub mouse_pos: glam::Vec2,
    pub mouse_pressed: bool,

    pub rotating: f32,
}

impl Editor {
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let mut state = EditorState::default();
        let ui = EditorUi::new(&window, &device, &queue, &sc_desc, &mut state);

        let camera = Camera::new(sc_desc, f32::FRAC_PI_2() / 2.0, *state.zoom as f32);
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let depth_texture = Texture::depth(device, size, 1, Some("depth_texture"));
        let sampled_depth_texture = Texture::depth(device, size, *state.samples as u32, Some("depth_texture_sampled"));
        let msaa = Texture::msaa(device, size, *state.samples as u32, Some("depth_texture_sampled"));
        let select = Texture::select(device, size, Some("depth_texture"));

        let select_buffer = select.make_buffer(
            device,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        );

        let size = [sc_desc.width, sc_desc.height];

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
            view_angle: camera.rot.into(),
            light_dir: glam::vec3(-0.5, -0.5, -1.0).normalize().into(),
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

            ui,
            state,
            modifiers,

            delta: Default::default(),

            mouse_raw,
            mouse_last,
            mouse_pos,
            mouse_pressed,

            rotating: 0.0
        }
    }

    pub fn input(&mut self, event: engine::RunnerEvent) -> bool {
        match event {
            engine::RunnerEvent::Window(event) => {
                match event {
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        self.mouse_pos = [
                            (2.0 * position.x as f32 / self.size[0] as f32) - 1.0,
                            (2.0 * position.y as f32 / self.size[1] as f32) - 1.0,
                        ]
                        .into();
                        self.mouse_raw = [position.x as u32, position.y as u32];
                        true
                    }
                    winit::event::WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        self.mouse_last = self.mouse_pos;
                        self.mouse_pressed = true;
                        true
                    }
                    winit::event::WindowEvent::MouseWheel {
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
                    winit::event::WindowEvent::MouseInput {
                        state: winit::event::ElementState::Released,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        self.mouse_pressed = false;
                        true
                    }
                    winit::event::WindowEvent::ModifiersChanged(m) => {
                        self.modifiers = *m;
                        true
                    }
                    _ => false,
                }
            }
            engine::RunnerEvent::Device(event) => {
                match event {
                    winit::event::DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Q), state: ElementState::Pressed, .. }) => {
                        self.rotating = -1.275 * self.delta.as_secs_f32();
                        true
                    },
                    winit::event::DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(VirtualKeyCode::E), state: ElementState::Pressed, .. }) => {
                        self.rotating = 1.275 * self.delta.as_secs_f32();
                        true
                    },
                    winit::event::DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Q), state: ElementState::Released, .. }) => {
                        if self.rotating < 0.0 {
                            self.rotating = 0.0;
                        }
                        true
                    },
                    winit::event::DeviceEvent::Key(KeyboardInput { virtual_keycode: Some(VirtualKeyCode::E), state: ElementState::Released, .. }) => {
                        if self.rotating > 0.0 {
                            self.rotating = 0.0;
                        }
                        true
                    },
                    _ => { false }
                }
            }
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.size = [sc_desc.width, sc_desc.height];
        self.msaa = self.msaa.with_size(device, self.size);
        self.depth_texture = self.depth_texture.with_size(device, self.size);
        self.sampled_depth_texture = self.sampled_depth_texture.with_size(device, self.size);
        self.select = self.select.with_size(device, self.size);

        self.select_buffer = self.select.make_buffer(
            device,
            wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
        );
        self.ui.resize(&device, &sc_desc, &mut self.state);
        self.camera.resize(&sc_desc);
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        window: &winit::window::Window,
        delta: std::time::Duration,
    ) {
        self.delta = delta;

        if let Some(&samples) = self.state.samples.on_change() {
            self.sampled_depth_texture = self.sampled_depth_texture
                .with_samples(device, samples as u32);
            self.msaa = self.msaa.with_samples(device, samples as u32);

            self.ico_screen.invalid(RendererInvalid::Pipeline);
        }

        self.state.frame_times.push(delta);
        self.ui.update(window, delta);

        self.camera.zoom = *self.state.zoom;
        if self.mouse_pressed && !self.ui.has_mouse() {
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

        let index = ((sc_desc.width * self.mouse_raw[1]) + self.mouse_raw[0])
            .min(sc_desc.width * sc_desc.height - 1) as wgpu::BufferAddress;
        let new = block_on(self.select_buffer.mapped_read(device, index));
        if self.selected != new {
            if let Some(face) = self.ico.face(new) {
                self.ico_uniform.s1 = face.siblings[0].map(|s| s.get()).unwrap_or(0);
                self.ico_uniform.s2 = face.siblings[1].map(|s| s.get()).unwrap_or(0);
                self.ico_uniform.s3 = face.siblings[2].map(|s| s.get()).unwrap_or(0);
            } else {
                self.ico_uniform.s1 = 0;
                self.ico_uniform.s2 = 0;
                self.ico_uniform.s3 = 0;
            }
            self.selected = new;
        }

        self.select_buffer.buffer().unmap();

        self.ico_uniform.view_proj = view_proj.into();
        self.ico_uniform.view_angle = self.camera.rot.into();
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
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
    ) {
        let color = palette::rgb::Srgb::from_components((0.53, 0.81, 0.92)).into_linear();
        let msaa = if *self.state.samples == 1 {
            None
        } else {
            Some(&self.msaa)
        };
        {
            let mut pass = begin_render_pass(
                encoder,
                &frame.view,
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
                    bytes_per_row: 4 * sc_desc.width,
                    rows_per_image: sc_desc.height,
                },
            },
            wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
        );

        if let Some(image_id) = self.state.image_id {
            if let Some(target) = self.ui.renderer.textures.get(image_id) {
                encoder.copy_texture_to_texture(
                    wgpu::TextureCopyView {
                        texture: &self.sampled_depth_texture.texture,
                        mip_level: 0,
                        origin: Default::default(),
                    },
                    wgpu::TextureCopyView {
                        texture: &target.texture(),
                        mip_level: 0,
                        origin: Default::default(),
                    },
                    wgpu::Extent3d {
                        width: sc_desc.width,
                        height: sc_desc.height,
                        depth: 1,
                    },
                )
            }
        }

        self.ui
            .render(&mut self.state, &frame.view, encoder, queue, device, window);
    }
}
