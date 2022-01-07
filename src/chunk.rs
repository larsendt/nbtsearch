use anyhow::{anyhow, Context, Result};

#[derive(Debug)]
enum ChunkCompression {
    Zlib,
    Gzip,
}

#[derive(Debug)]
pub struct Chunk {
    pub x: i64,
    pub z: i64,
    pub nbt: nbt::Blob,
}

impl Chunk {
    pub fn new(x: i64, z: i64, mut buf: Vec<u8>) -> Result<Self> {
        // TODO: need this?
        let _header_size = ((buf[0] as usize) << 24)
            + ((buf[1] as usize) << 16)
            + ((buf[2] as usize) << 8)
            + ((buf[3] as usize) << 0);

        let compression = match buf[4] {
            1 => ChunkCompression::Gzip,
            2 => ChunkCompression::Zlib,
            c => {
                return Err(anyhow!(
                    "For chunk {},{} expected chunk compression 1 or 2 but got {}",
                    x,
                    z,
                    c
                ))
            }
        };

        // Take the header from the buffer
        buf.drain(0..5);
        let nbt = Self::get_nbt(buf, &compression, x, z)?;

        Ok(Chunk { x, z, nbt })
    }

    fn get_nbt(data: Vec<u8>, compression: &ChunkCompression, x: i64, z: i64) -> Result<nbt::Blob> {
        match compression {
            ChunkCompression::Zlib => nbt::from_zlib_reader(&data[..])
                .with_context(|| format!("Error decoding Zlib NBT for chunk {},{}", x, z)),
            ChunkCompression::Gzip => nbt::from_gzip_reader(&data[..])
                .with_context(|| format!("Error decoding Gzip NBT for chunk {},{}", x, z)),
        }
    }
}
