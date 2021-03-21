use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;

lazy_static! {
    static ref SHADERS: Mutex<HashMap<&'static str, wgpu::ShaderModule>> =
        Mutex::new(HashMap::new());
}

pub struct Shaders<'a> {
    mutex: MutexGuard<'a, HashMap<&'static str, wgpu::ShaderModule>>,
}

impl Shaders<'_> {
    pub fn get(&self, key: &'static str) -> Option<&wgpu::ShaderModule> {
        self.mutex.get(key)
    }
}

pub fn shader_add(device: &wgpu::Device, key: &'static str, shader: &wgpu::ShaderModuleDescriptor) {
    let shader = device.create_shader_module(shader);
    SHADERS.lock().unwrap().insert(key, shader);
}

pub fn shaders() -> Shaders<'static> {
    let map = SHADERS.lock().unwrap();
    Shaders { mutex: map }
}
