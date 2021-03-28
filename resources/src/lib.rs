mod resources;
use std::io::Read;

pub use resources::*;



pub fn read() -> Result<Vec<ResourceItem>, Box<dyn std::error::Error>> {
    let path = std::env::current_exe()?
        .parent()
        .unwrap()
        .join("resources.dat");
    let mut file = if let Ok(file) = std::fs::File::open(path) {
        file
    } else if let Ok(file) = std::fs::File::open(std::env::current_dir()?.join("resources.dat")) {
        file
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "cannot find file 'resources.dat'",
        )
        .into());
    };
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let resources: Vec<ResourceItem> = bincode::deserialize(&data)?;
    Ok(resources)
}