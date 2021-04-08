use std::{
    fs::read_to_string,
    io::prelude::*,
    path::{Path, PathBuf},
};

use flate2::{write::ZlibEncoder, Compression};
use image::{EncodableLayout, GenericImageView};

use resources::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let wd = std::env::current_dir()?;
    for res_file in glob::glob(wd.join("data/*.json").to_str().unwrap()).unwrap() {
        match res_file {
            Ok(path) => {
                let mut file = std::fs::File::open(&path)?;
                let output = {
                    let path = path.with_extension("dat");
                    path.file_name().unwrap().to_owned()
                };
                let args: Vec<_> = std::env::args().collect();
                if args.len() > 1
                    && !args.contains(&path.file_stem().unwrap().to_str().unwrap().to_owned())
                {
                    continue;
                }
                let mut input = String::new();
                file.read_to_string(&mut input)?;
                let descriptions: Inputs = serde_json::from_str(&input)?;
                compile(descriptions, output)?;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn compile(
    descriptions: Inputs,
    output: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let wd = std::env::current_dir()?;
    let mut resources = Vec::new();
    let mut compiler = shaderc::Compiler::new().expect("Unable to create shader compiler");

    for InputItem { label, input } in descriptions.inputs {
        match input {
            Input::Image(ImageInput { paths, mipmaps, format }) => {
                let images = paths
                    .iter()
                    .map(|p| wd.join(Path::new("data")).join(p))
                    .map(|p| {
                        log::info!("reading {:?}", p);
                        image::open(&p).map(|i| (p, i))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let size = images.first().unwrap().1.dimensions();
                let depth = images.len() as u32;
                let levels = mipmaps.map(|v| v.get()).unwrap_or(1);

                let mut buffer = Vec::new();
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

                for (path, image) in images {
                    log::info!("processing image {:?}", path);
                    for level in 0..levels {
                        let size = (size.0 / 2u32.pow(level), size.1 / 2u32.pow(level));
                        log::info!("resizeing to {:?}", size);
                        let resized =
                            image.resize(size.0, size.1, image::imageops::FilterType::CatmullRom);
                        let mut encoded = Vec::new();
                        let encoder = image::codecs::dxt::DxtEncoder::new(&mut encoded);
                        encoder.encode(
                            resized.to_rgba8().as_bytes(),
                            size.0,
                            size.1,
                            image::dxt::DXTVariant::DXT5,
                        )?;

                        buffer.extend_from_slice(&encoded);
                    }
                }

                log::info!("compressing texture {:?}", &label);
                e.write_all(&buffer)?;
                let compressed = e.finish()?;
                log::info!("writing texture {:?}", &label);
                resources.push(ResourceItem {
                    label,
                    resource: Resource::Image(ImageRgba {
                        size,
                        depth,
                        levels,
                        data: compressed,
                        format,
                    }),
                });
            }
            Input::Shader(ShaderInput { path }) => {
                let path = Path::new("data").join(path);
                log::info!("compiling shader {:?}", &path);
                let shader_src = ShaderData::load(path)?;
                let compiled = compiler.compile_into_spirv(
                    &shader_src.src,
                    shader_src.kind,
                    &shader_src.src_path.to_str().unwrap(),
                    "main",
                    None,
                )?;
                let shader = Shader {
                    data: Vec::from(compiled.as_binary()),
                };
                resources.push(ResourceItem {
                    label: label.clone(),
                    resource: Resource::Shader(shader),
                });
            }
        }
    }
    let output = wd.join(output);
    let mut out_file = std::fs::File::create(&output)?;
    log::info!("encoding output");
    let data = bincode::serialize(&resources)?;
    log::info!("wrinting output to {:?}", &output);
    out_file.write_all(&data)?;
    out_file.flush()?;
    log::info!("done");
    Ok(())
}

struct ShaderData {
    src: String,
    src_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> std::io::Result<Self> {
        let src = src_path.to_str().expect("invalid filename");
        let kind = {
            if src.ends_with(".vert.glsl") {
                shaderc::ShaderKind::Vertex
            } else if src.ends_with(".frag.glsl") {
                shaderc::ShaderKind::Fragment
            } else if src.ends_with(".comp.glsl") {
                shaderc::ShaderKind::Compute
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Unsupported shader: {}", src_path.display()),
                ));
            }
        };

        let src = read_to_string(src_path.clone())?;

        Ok(Self {
            src,
            src_path,
            kind,
        })
    }
}
