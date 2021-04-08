use std::{io::prelude::*, num::NonZeroU32, path::PathBuf};

use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ImageFormat {
    LinearRgb,
    Srgb,
}

#[derive(Serialize, Deserialize)]
pub struct ImageRgba {
    pub size: (u32, u32),
    pub depth: u32,
    pub levels: u32,
    pub data: Vec<u8>,
    pub format: ImageFormat,
}

impl ImageRgba {
    pub fn read(&self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut decoder = ZlibDecoder::new(&self.data[..]);
        let read = decoder.read_to_end(buf)?;
        Ok(read)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Shader {
    pub data: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub enum Resource {
    Image(ImageRgba),
    Shader(Shader),
}

#[derive(Serialize, Deserialize)]
pub struct ResourceItem {
    pub label: String,
    pub resource: Resource,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Input {
    Image(ImageInput),
    Shader(ShaderInput),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInput {
    pub paths: Vec<PathBuf>,
    pub mipmaps: Option<NonZeroU32>,
    pub format: ImageFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderInput {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputItem {
    pub label: String,
    pub input: Input,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
    pub inputs: Vec<InputItem>,
}
