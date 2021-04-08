use std::fmt::Display;

use palette::rgb::{Rgb, RgbStandard};
use wgpu::{util::DeviceExt, CommandEncoder, RenderPass, TextureView};

use crate::graphics::{
    common::{ItemBuffer, PipelineFormat, PipelineSettings, TextureLayout, UniformBinding},
    texture::Texture,
};

use super::common::TextureBinding;

pub fn begin_render_pass<'a>(
    encoder: &'a mut CommandEncoder,
    frame: &'a TextureView,
    depth_texture: Option<&'a Texture>,
    color: Rgb<impl RgbStandard, f64>,
    msaa: Option<&'a Texture>,
    name: Option<impl Display>,
) -> RenderPass<'a> {
    let label = name.as_ref().map(|l| format!("{}_render_pass", l));
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: label.as_deref(),
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: if let Some(msaa) = msaa {
                &msaa.view
            } else {
                frame
            },
            resolve_target: if msaa.is_some() { Some(frame) } else { None },
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: color.red,
                    g: color.green,
                    b: color.blue,
                    a: 1.0,
                }),
                store: true,
            },
        }],
        depth_stencil_attachment: depth_texture.map(|depth_texture| {
            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }
        }),
    })
}

pub fn create_pipeline(
    device: &wgpu::Device,
    format: impl Into<PipelineFormat>,
    settings: &PipelineSettings,
    vs: &'static str,
    fs: &'static str,
    name: Option<impl Display>,
) -> wgpu::RenderPipeline {
    let PipelineSettings {
        layouts,
        buffers,
        topology,
        samples,
    } = settings;
    let format = format.into();

    let label = name.as_ref().map(|l| format!("{}_render_layout", l));
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: label.as_deref(),
        bind_group_layouts: layouts,
        push_constant_ranges: &[],
    });

    let shaders = crate::shaders();
    let lock = shaders.lock();
    let vs_module = lock.get(vs).unwrap();
    let fs_module = lock.get(fs).unwrap();

    let label = name.as_ref().map(|l| format!("{}_render_pipeline", l));
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: label.as_deref(),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: format.format,
                color_blend: wgpu::BlendState::REPLACE,
                alpha_blend: wgpu::BlendState::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: *topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            polygon_mode: wgpu::PolygonMode::Fill,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            bias: wgpu::DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
            clamp_depth: false,
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: *samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}

pub fn create_uniform_binding<T>(
    device: &wgpu::Device,
    name: Option<impl Display>,
) -> UniformBinding<T>
where
    T: crevice::std140::AsStd140,
{
    let label = name.as_ref().map(|l| format!("{}_uniform_layout", l));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: label.as_deref(),
    });

    let label = name.as_ref().map(|l| format!("{}_unfiform_buffer", l));
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: label.as_deref(),
        size: T::std140_size_static() as wgpu::BufferAddress,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let label = name.as_ref().map(|l| format!("{}_uniform_binding", l));
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &buffer,
                offset: 0,
                size: None,
            },
        }],
        label: label.as_deref(),
    });

    UniformBinding::new(buffer, layout, binding)
}

pub fn create_texture_binding_layout(
    device: &wgpu::Device,
    texture: &Texture,
    name: Option<impl Display>,
) -> TextureLayout {
    let label = name.as_ref().map(|s| format!("{}_texture_binding", s));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: texture.view_dimension,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: false,
                    comparison: false,
                },
                count: None,
            },
        ],
        label: label.as_deref(),
    });

    TextureLayout { layout }
}

pub fn create_texture_binding(
    device: &wgpu::Device,
    texture: &Texture,
    name: Option<impl Display>,
) -> TextureBinding {
    let label = name.as_ref().map(|s| format!("{}_texture_binding", s));
    let layout = create_texture_binding_layout(device, texture, name);
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: label.as_deref(),
        layout: &layout.layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
    });
    TextureBinding::new(layout, binding)
}

pub fn create_buffer_size<T, L>(
    device: &wgpu::Device,
    items: usize,
    usage: wgpu::BufferUsage,
    label: Option<L>,
) -> wgpu::Buffer
where
    T: bytemuck::Pod,
    L: Display,
{
    let label = label.as_ref().map(|s| format!("{}", s));
    let size = items * std::mem::size_of::<T>();
    device.create_buffer(&wgpu::BufferDescriptor {
        label: label.as_deref(),
        size: size as wgpu::BufferAddress,
        usage,
        mapped_at_creation: false,
    })
}

pub fn create_buffer<T, L>(
    device: &wgpu::Device,
    data: Option<&[T]>,
    usage: wgpu::BufferUsage,
    name: Option<L>,
) -> ItemBuffer<T>
where
    T: bytemuck::Pod,
    L: Display,
{
    let mut num_items = 1;
    let label = name.as_ref().map(|s| format!("{}_buffer", s));
    let buffer = if let Some(data) = data {
        num_items = data.len();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label.as_deref(),
            contents: bytemuck::cast_slice(data),
            usage,
        })
    } else {
        create_buffer_size::<T, _>(device, 1, usage, label.as_ref())
    };
    ItemBuffer::new(buffer, num_items, usage, label)
}

pub fn calc_normal<T>(v1: T, v2: T, v3: T) -> T
where
    T: Into<mint::Vector3<f32>> + From<mint::Vector3<f32>>,
{
    let w = glam::Vec3::from(v1.into());
    let u = glam::Vec3::from(v2.into()) - w;
    let v = glam::Vec3::from(v3.into()) - w;
    let normal = u.cross(v).normalize();
    mint::Vector3::from(normal).into()
}

pub fn calc_tangent(v: [glam::Vec3; 3], t: [glam::Vec2; 3]) -> (glam::Vec3, glam::Vec3) {
    let dp1 = v[1] - v[0];
    let dp2 = v[2] - v[0];
    let duv1 = t[1] - t[0];
    let duv2 = t[2] - t[0];
    let r = 1.0 / (duv1.x*duv2.y - duv1.y*duv2.x);
    let tangent: glam::Vec3 = (dp1*duv2.y - dp2*duv1.y) * r;
    let bitangent: glam::Vec3 = (dp2*duv1.x - dp1*duv2.x) * r;
    (tangent, bitangent)
}