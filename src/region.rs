use crate::chunk::Chunk;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{ErrorKind, Read},
    os::unix::prelude::FileExt,
};

#[derive(Debug)]
pub struct Region {
    coords: (i64, i64),
    location_table: LocationTable,
    // TODO
    _mtime_table: MtimeTable,
    file: File,
}

#[derive(Debug)]
pub struct LocationTable {
    table: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct MtimeTable {
    // TODO
    _buf: [u8; 4096],
}

impl Region {
    pub fn new(filename: &str) -> Result<Self> {
        let mut file = File::open(filename)?;

        Ok(Self {
            coords: Self::read_region_coords(filename)?,
            location_table: Self::read_location_table(&mut file, filename)?,
            _mtime_table: Self::read_mtime_table(&mut file, filename)?,
            file,
        })
    }

    pub fn all_chunk_coords(&self) -> impl Iterator<Item = (i64, i64)> {
        let min_x = self.coords.0 * 32;
        let max_x = min_x + 32;
        let min_z = self.coords.1 * 32;
        let max_z = min_z + 32;

        (min_x..max_x)
            .into_iter()
            .flat_map(move |x| (min_z..max_z).into_iter().map(move |z| (x, z)))
    }

    pub fn get_chunk(&mut self, x: i64, z: i64) -> Result<Option<Chunk>> {
        let loc_table_idx = (x.rem_euclid(32) + z.rem_euclid(32) * 32) as usize;
        let loc_table_offset = self.location_table.table[loc_table_idx].0;
        let loc_table_size = self.location_table.table[loc_table_idx].1;

        if loc_table_size == 0 {
            return Ok(None);
        }

        let mut buf: Vec<u8> = vec![0u8; loc_table_size];
        let r = self.file.read_exact_at(&mut buf, loc_table_offset as u64);
        if let Err(e) = r {
            match e.kind() {
                ErrorKind::UnexpectedEof => (),
                _ => return Err(e).with_context(|| format!("Failed to read chunk at {},{}", x, z)),
            }
        }

        if loc_table_size > 0 {
            Ok(Some(Chunk::new(x, z, buf)?))
        } else {
            panic!(
                "Chunk {},{} had loc_table_offset {} and loc_table_size {}",
                x, z, loc_table_offset, loc_table_size
            );
        }
    }

    pub fn all_chunks(&mut self) -> impl Iterator<Item = Result<Chunk>> + '_ {
        self.all_chunk_coords()
            .map(|(x, z)| self.get_chunk(x, z))
            .map(|c| {
                if let Ok(Some(c)) = c {
                    Some(Ok(c))
                } else if let Ok(None) = c {
                    None
                } else if let Err(e) = c {
                    Some(Err(e))
                } else {
                    panic!("Don't know what to do: {:?}", c)
                }
            })
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
    }

    fn read_region_coords(filename: &str) -> Result<(i64, i64)> {
        let coord_regex = Regex::new(r".*/r\.(?P<x>-?\d+)\.(?P<z>-?\d+)\.mca")?;
        let caps = match coord_regex.captures(filename) {
            Some(c) => c,
            None => {
                return Err(anyhow!(format!(
                    "Unable to extract coords from region filename '{}'",
                    filename
                )))
            }
        };

        let x: i64 = (&caps["x"]).parse()?;
        let z: i64 = (&caps["z"]).parse()?;
        Ok((x, z))
    }

    fn read_location_table(file: &mut File, path: &str) -> Result<LocationTable> {
        let mut location_table = [0u8; 4096];
        file.read_exact(&mut location_table)
            .with_context(|| format!("Failed to read location table from {}", path))?;

        let mut table = Vec::new();

        for entry_idx in (0..4096).step_by(4) {
            let offset_segments = ((location_table[entry_idx] as usize) << 16)
                + ((location_table[entry_idx + 1] as usize) << 8)
                + location_table[entry_idx + 2] as usize;
            let size_segments = location_table[entry_idx + 3] as usize;
            let offset_bytes = offset_segments * 4096;
            let size_bytes = size_segments * 4096;
            table.push((offset_bytes, size_bytes));
        }

        Ok(LocationTable { table })
    }

    fn read_mtime_table(file: &mut File, path: &str) -> Result<MtimeTable> {
        let mut mtime_table = [0u8; 4096];
        file.read_exact(&mut mtime_table)
            .with_context(|| format!("Failed to read mtime table from {}", path))?;
        Ok(MtimeTable { _buf: mtime_table })
    }
}
