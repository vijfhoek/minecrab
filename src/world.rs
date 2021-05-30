use cgmath::Vector3;

use crate::{
    chunk::{BlockType, Chunk},
    instance::Instance,
};

pub struct World {
    chunks: Vec<Vec<Vec<Chunk>>>,
}

impl World {
    pub fn generate() -> Self {
        let mut chunks = Vec::with_capacity(16);
        for y in 0..16 {
            let mut chunks_z = Vec::with_capacity(16);
            for z in 0..16 {
                let mut chunks_x = Vec::with_capacity(16);
                for x in 0..16 {
                    let chunk = Chunk::generate(x, y, z);
                    chunks_x.push(chunk);
                }
                chunks_z.push(chunks_x);
            }
            chunks.push(chunks_z);
        }

        Self { chunks }
    }

    pub fn to_instances(&self) -> Vec<(BlockType, Vector3<i32>, Vec<Instance>)> {
        let instant = std::time::Instant::now();

        let mut instance_lists = Vec::new();

        for (y, chunks_y) in self.chunks.iter().enumerate() {
            for (z, chunks_z) in chunks_y.iter().enumerate() {
                for (x, chunk) in chunks_z.iter().enumerate() {
                    let offset = Vector3::new(x as i32, y as i32, z as i32);
                    for (block_type, instances) in chunk.to_instances(offset) {
                        instance_lists.push((block_type, offset, instances));
                    }
                }
            }
        }

        let elapsed = instant.elapsed();
        println!("generating world instances took {:?}", elapsed);

        instance_lists
    }
}
