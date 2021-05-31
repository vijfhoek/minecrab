use crate::{chunk::Chunk, vertex::Vertex};
use cgmath::Vector3;

pub struct World {
    chunks: Vec<Vec<Vec<Chunk>>>,
}

const WORLD_SIZE: usize = 16;

impl World {
    pub fn generate() -> Self {
        let mut chunks = Vec::with_capacity(WORLD_SIZE);
        for y in 0..WORLD_SIZE {
            let mut chunks_z = Vec::with_capacity(WORLD_SIZE);
            for z in 0..WORLD_SIZE {
                let mut chunks_x = Vec::with_capacity(WORLD_SIZE);
                for x in 0..WORLD_SIZE {
                    let chunk = Chunk::generate(x as i32, y as i32, z as i32);
                    chunks_x.push(chunk);
                }
                chunks_z.push(chunks_x);
            }
            chunks.push(chunks_z);
        }

        Self { chunks }
    }

    pub fn to_instances(&self) -> Vec<(Vec<Vertex>, Vec<u16>)> {
        let instant = std::time::Instant::now();
        let mut instances = Vec::new();

        for (y, chunks_y) in self.chunks.iter().enumerate() {
            for (z, chunks_z) in chunks_y.iter().enumerate() {
                for (x, chunk) in chunks_z.iter().enumerate() {
                    let offset = Vector3::new(x as i32 * 16, y as i32 * 16, z as i32 * 16);
                    instances.push(chunk.to_instances(offset));
                }
            }
        }

        let elapsed = instant.elapsed();
        println!("generating world instances took {:?}", elapsed);

        instances
    }
}
