use std::sync::Arc;

use engine::{event::RunnerEvent, graphics::texture::Texture};
use parking_lot::Mutex;

pub struct UiValue<T>
where
    T: PartialEq + Copy,
{
    old: T,
    new: T,
}

impl<T> UiValue<T>
where
    T: PartialEq + Copy,
{
    fn new(value: T) -> Self {
        Self {
            old: value,
            new: value,
        }
    }

    pub fn on_change(&mut self) -> Option<&T> {
        if self.new != self.old {
            self.old = self.new;
            Some(&self.new)
        } else {
            None
        }
    }
}

impl<T> std::ops::Deref for UiValue<T>
where
    T: PartialEq + Copy,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.new
    }
}

impl<T> std::ops::DerefMut for UiValue<T>
where
    T: PartialEq + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.new
    }
}

impl<T> Default for UiValue<T>
where
    T: Default + PartialEq + Copy,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct UiIo {
    pub wants_mouse: bool,
    pub wants_keyboard: bool,
}

impl UiIo {
    pub fn new(wants_mouse: bool, wants_keyboard: bool) -> Self {
        Self {
            wants_mouse,
            wants_keyboard,
        }
    }
}

pub struct EditorState {
    pub size: UiValue<i32>,
    pub zoom: UiValue<f32>,
    pub perspective: UiValue<bool>,
    pub light_mix: UiValue<f32>,
    pub samples: UiValue<i32>,
    pub samples_select: i32,

    pub frame_times: Vec<std::time::Duration>,
    pub fps: f32,

    pub image_id: Option<imgui::TextureId>,

    pub ui_io: Arc<Mutex<UiIo>>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            size: UiValue::new(1),
            zoom: UiValue::new(1.0),
            perspective: UiValue::new(true),
            light_mix: UiValue::new(0.5),
            samples: UiValue::new(1),
            samples_select: 0,
            frame_times: Vec::with_capacity(60),
            fps: 0.0,
            image_id: None,
            ui_io: Arc::new(Mutex::new(UiIo::new(false, false))),
        }
    }
}

pub struct EditorUi {
    pub context: imgui::Context,
    pub renderer: imgui_wgpu::Renderer,
    pub platform: imgui_winit_support::WinitPlatform,
    pub ui_io: Arc<Mutex<UiIo>>,
}

impl EditorUi {
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        state: &mut EditorState,
    ) -> EditorUi {
        let mut context = imgui::Context::create();

        context.set_ini_filename(None);

        let mut io = context.io_mut();
        io.display_size = [sc_desc.width as f32, sc_desc.height as f32];
        io.font_global_scale = (1.0 / window.scale_factor()) as f32;

        let mut renderer = imgui_wgpu::Renderer::new(
            &mut context,
            device,
            queue,
            imgui_wgpu::RendererConfig {
                texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
                depth_format: None,
                sample_count: 1,
                ..Default::default()
            },
        );

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(
            context.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let image = imgui_wgpu::Texture::new(
            device,
            &renderer,
            imgui_wgpu::TextureConfig {
                size: wgpu::Extent3d {
                    width: sc_desc.width,
                    height: sc_desc.height,
                    depth: 1,
                },
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT
                    | wgpu::TextureUsage::SAMPLED
                    | wgpu::TextureUsage::COPY_DST,
                format: Some(Texture::DEPTH_FORMAT),
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                label: None,
            },
        );

        state.image_id = Some(renderer.textures.insert(image));

        EditorUi {
            context,
            renderer,
            platform,
            ui_io: Arc::clone(&state.ui_io),
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<RunnerEvent>,
    ) {
        self.platform
            .handle_event(self.context.io_mut(), &window, &event)
    }

    pub fn update(&mut self, window: &winit::window::Window, delta: std::time::Duration) {
        self.context.io_mut().update_delta_time(delta);
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .unwrap();
        let mut ui_io = self.ui_io.lock();
        ui_io.wants_keyboard = self.context.io().want_capture_keyboard;
        ui_io.wants_mouse = self.context.io().want_capture_mouse;
    }

    pub fn has_mouse(&self) -> bool {
        self.context.io().want_capture_mouse
    }

    pub fn has_keyboard(&self) -> bool {
        self.context.io().want_capture_keyboard
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32), state: &mut EditorState) {
        let mut io = self.context.io_mut();
        io.display_size = [size.0 as f32, size.1 as f32];

        if let Some(image_id) = state.image_id {
            let image = imgui_wgpu::Texture::new(
                device,
                &self.renderer,
                imgui_wgpu::TextureConfig {
                    size: wgpu::Extent3d {
                        width: size.0,
                        height: size.1,
                        depth: 1,
                    },
                    usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
                    format: Some(Texture::DEPTH_FORMAT),
                    ..Default::default()
                },
            );

            self.renderer.textures.replace(image_id, image);
        }
    }

    pub fn draw(frame: &imgui::Ui, window: &winit::window::Window, state: &mut EditorState) {
        let total: std::time::Duration = state.frame_times.iter().sum();
        if total > std::time::Duration::from_millis(250) {
            state.fps = state.frame_times.len() as f32 / total.as_secs_f32();
            state.frame_times.clear();
        }
        imgui::Window::new(imgui::im_str!("Xerograph"))
            .size(
                [400.0, window.inner_size().height as f32 - 30.0],
                imgui::Condition::Always,
            )
            .resizable(false)
            .movable(false)
            .position([15.0, 15.0], imgui::Condition::Always)
            .build(frame, || {
                frame.label_text(
                    imgui::im_str!("fps"),
                    &imgui::ImString::new(format!("{:.2}", state.fps)),
                );
                imgui::Slider::new(imgui::im_str!("Size"))
                    .range(0..=5)
                    .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                    .build(frame, &mut state.size);
                imgui::Slider::new(imgui::im_str!("Zoom"))
                    .range(0.1..=2.0)
                    .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                    .build(frame, &mut state.zoom);
                frame.checkbox(imgui::im_str!("Perspective"), &mut state.perspective);
                imgui::Slider::new(imgui::im_str!("Shading"))
                    .range(0.0..=1.0)
                    .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                    .build(frame, &mut state.light_mix);
                let values = vec![1, 2, 4, 8];
                let items = values
                    .iter()
                    .map(|v| imgui::ImString::new(format!("{}", v)))
                    .collect::<Vec<_>>();
                if frame.list_box(
                    imgui::im_str!("Samples"),
                    &mut state.samples_select,
                    items.iter().collect::<Vec<_>>().as_slice(),
                    4,
                ) {
                    *state.samples = values[state.samples_select as usize]
                }
                if let Some(image_id) = state.image_id {
                    imgui::Image::new(image_id, [380.0, 214.0])
                        .border_col([1.0, 1.0, 1.0, 1.0])
                        .build(frame);
                }
            });
    }

    pub fn render(
        &mut self,
        state: &mut EditorState,
        frame: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        window: &winit::window::Window,
    ) {
        // if let Some(image_id) = self.state.image_id {
        //     if let Some(texture) = self.ui.renderer.textures.get(image_id) {
        //         encoder.copy_texture_to_texture(
        //             wgpu::TextureCopyView {
        //                 texture: &self.sampled_depth_texture.texture,
        //                 mip_level: 0,
        //                 origin: Default::default(),
        //             },
        //             wgpu::TextureCopyView {
        //                 texture: &texture.texture(),
        //                 mip_level: 0,
        //                 origin: Default::default(),
        //             },
        //             wgpu::Extent3d {
        //                 width: size.0,
        //                 height: size.1,
        //                 depth: 1,
        //             },
        //         )
        //     }
        // }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("block_pipeline_render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        let frame = self.context.frame();
        Self::draw(&frame, window, state);
        self.platform.prepare_render(&frame, window);
        let draw_data = frame.render();

        self.renderer
            .render(&draw_data, queue, device, &mut render_pass)
            .unwrap();
    }
}
