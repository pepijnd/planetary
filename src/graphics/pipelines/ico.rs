use crevice::std140::AsStd140;

use crate::{
    graphics::{
        common::{
            BundleData, ItemBuffer, Pipeline, PipelineFormat, PipelineSettings, UniformBinding,
        },
        helper::{create_buffer, create_pipeline, create_uniform_binding},
        texture::Texture,
    },
    structures::ico::Ico,
};

#[derive(Debug)]
pub struct IcoPipeline {
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub uniform_binding: UniformBinding<IcoUniform>,
}

#[derive(Debug, Clone)]
pub struct IcoBuffer {
    pub vertex_buffer: ItemBuffer<IcoVertex>,
}

impl IcoBuffer {
    pub fn build(device: &wgpu::Device) -> IcoBuffer {
        let vertex_buffer = create_buffer(
            device,
            None,
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            Some("ico_vertices"),
        );
        IcoBuffer { vertex_buffer }
    }
}

impl BundleData for IcoBuffer {
    type Data = Ico;
    type Id = usize;

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, ico: &Self::Data) {
        let data = ico.vertex_data();
        self.vertex_buffer.update(device, queue, data.as_slice());
    }

    fn id(&self) -> Self::Id {
        self.vertex_buffer.id()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsStd140)]
pub struct IcoUniform {
    pub view_proj: mint::ColumnMatrix4<f32>,
    pub view_angle: mint::Vector3<f32>,
    pub light_dir: mint::Vector3<f32>,
    pub selected: u32,
    pub s1: u32,
    pub s2: u32,
    pub s3: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IcoVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub index: u32,
}

impl IcoVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct IcoRendererSettings {
    pub vs: &'static str,
    pub fs: &'static str,
}

pub struct IcoRenderer {
    pub uniform_binding: UniformBinding<IcoUniform>,
    pub vs: &'static str,
    pub fs: &'static str,
}

impl Pipeline for IcoRenderer {
    type Settings = IcoRendererSettings;
    type Data = IcoBuffer;

    fn build_pipeline(
        &self,
        device: &wgpu::Device,
        format: PipelineFormat,
        samples: u32,
    ) -> wgpu::RenderPipeline {
        let settings = PipelineSettings {
            layouts: &[&self.uniform_binding.layout],
            buffers: &[IcoVertex::desc()],
            samples,
            ..Default::default()
        };

        create_pipeline(device, format, &settings, self.vs, self.fs, Some("ico"))
    }

    fn build_bundle(
        &self,
        device: &wgpu::Device,
        pipeline: &wgpu::RenderPipeline,
        format: PipelineFormat,
        samples: u32,
        data: &IcoBuffer,
    ) -> wgpu::RenderBundle {
        let mut bundle =
            device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: Some("ico_render_bundle"),
                color_formats: &[format.format],
                depth_stencil_format: Some(Texture::DEPTH_FORMAT),
                sample_count: samples,
            });

        let vb = data.vertex_buffer.buffer();

        bundle.set_pipeline(pipeline);
        bundle.set_bind_group(0, &self.uniform_binding.binding, &[]);
        bundle.set_vertex_buffer(0, vb.slice(..));
        bundle.draw(0..data.vertex_buffer.num_items() as u32, 0..1);
        bundle.finish(&wgpu::RenderBundleDescriptor {
            label: Some("ico_render_bundle"),
        })
    }

    fn build(device: &wgpu::Device, settings: &IcoRendererSettings) -> Self {
        let uniform_binding: UniformBinding<IcoUniform> =
            create_uniform_binding(device, 0, Some("ico"));
        let IcoRendererSettings { vs, fs } = settings.clone();
        Self {
            uniform_binding,
            vs,
            fs,
        }
    }
}
