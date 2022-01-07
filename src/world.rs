use crate::region::Region;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

pub struct World {
    world_dir: PathBuf,
}

impl World {
    pub fn new(world_dir: impl Into<PathBuf>) -> Self {
        Self {
            world_dir: world_dir.into(),
        }
    }

    pub fn entity_regions(&self, dimension: &str) -> Result<impl Iterator<Item = Result<Region>>> {
        let mut entity_dir = self.world_dir.clone();
        match dimension {
            "overworld" => (),
            "nether" => entity_dir.push("DIM-1"),
            "end" => entity_dir.push("DIM1"),
            _ => return Err(anyhow!("Invalid dimension: {}", dimension)),
        }
        entity_dir.push("entities");
        Ok(fs::read_dir(entity_dir)?
            .map(|de| de.unwrap().path().as_os_str().to_str().unwrap().to_string())
            .filter(|path| fs::metadata(path).unwrap().len() > 0)
            .map(|path| Region::new(&path)))
    }
}
