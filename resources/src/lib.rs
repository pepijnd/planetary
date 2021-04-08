mod resources;
use std::{io::Read, path::Path};

pub use resources::*;

pub fn read(inputs: &[impl AsRef<Path>]) -> Result<Vec<ResourceItem>, Box<dyn std::error::Error>> {
    let mut resources = Vec::new();
    for file in inputs {
        let path = std::env::current_exe()?.parent().unwrap().join(file);
        let mut file = if let Ok(file) = std::fs::File::open(path) {
            file
        } else if let Ok(file) = std::fs::File::open(std::env::current_dir()?.join(file))
        {
            file
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("cannot find file '{}'", file.as_ref().to_string_lossy()),
            )
            .into());
        };
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let mut file_resources: Vec<ResourceItem> = bincode::deserialize(&data)?;
        resources.append(&mut file_resources);
    }
    Ok(resources)
}
