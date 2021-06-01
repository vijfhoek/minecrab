use crate::{
    chunk::{Block, Chunk, CHUNK_SIZE},
    vertex::Vertex,
};
use cgmath::{InnerSpace, Vector3};
use rayon::prelude::*;

pub struct World {
    pub chunks: Vec<Vec<Vec<Chunk>>>,
}

const WORLD_SIZE: Vector3<usize> = Vector3::new(
    32 * 16 / CHUNK_SIZE,
    16 * 16 / CHUNK_SIZE,
    32 * 16 / CHUNK_SIZE,
);

impl World {
    pub fn generate() -> Self {
        let mut chunks = Vec::new();

        (0..WORLD_SIZE.y)
            .into_par_iter()
            .map(|y| {
                let mut chunks_z = Vec::new();
                for z in 0..WORLD_SIZE.z {
                    let mut chunks_x = Vec::new();
                    for x in 0..WORLD_SIZE.x {
                        let chunk = Chunk::generate(x as i32, y as i32, z as i32);
                        chunks_x.push(chunk);
                    }
                    chunks_z.push(chunks_x);
                }
                chunks_z
            })
            .collect_into_vec(&mut chunks);

        Self { chunks }
    }

    pub fn highlighted_for_chunk(
        highlighted: Option<(Vector3<usize>, Vector3<i32>)>,
        chunk_position: Vector3<usize>,
    ) -> Option<(Vector3<usize>, Vector3<i32>)> {
        let position = chunk_position * CHUNK_SIZE;
        if let Some((pos, face)) = highlighted {
            if pos.x >= position.x
                && pos.x < position.x + CHUNK_SIZE
                && pos.y >= position.y
                && pos.y < position.y + CHUNK_SIZE
                && pos.z >= position.z
                && pos.z < position.z + CHUNK_SIZE
            {
                Some((pos - position, face))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_geometry(
        &self,
        highlighted: Option<(Vector3<usize>, Vector3<i32>)>,
    ) -> Vec<(Vector3<usize>, Vec<Vertex>, Vec<u16>)> {
        let instant = std::time::Instant::now();

        let chunks = &self.chunks;
        let geometry = chunks
            .par_iter()
            .enumerate()
            .flat_map(|(y, chunks_y)| {
                let mut geometry = Vec::new();
                for (z, chunks_z) in chunks_y.iter().enumerate() {
                    for (x, chunk) in chunks_z.iter().enumerate() {
                        let chunk_position = Vector3::new(x as usize, y as usize, z as usize);
                        let offset = (chunk_position * CHUNK_SIZE).cast().unwrap();
                        let h = Self::highlighted_for_chunk(highlighted, chunk_position);
                        let (vertices, indices) = chunk.to_geometry(offset, h.as_ref());
                        geometry.push((Vector3::new(x, y, z), vertices, indices));
                    }
                }
                geometry
            })
            .collect();

        let elapsed = instant.elapsed();
        println!("Generating world geometry took {:?}", elapsed);

        geometry
    }

    pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<&Block> {
        if x < 0 || y < 0 || z < 0 {
            return None;
        }

        let chunk = match self
            .chunks
            .get(y as usize / CHUNK_SIZE)
            .and_then(|chunk_layer| chunk_layer.get(z as usize / CHUNK_SIZE))
            .and_then(|chunk_row| chunk_row.get(x as usize / CHUNK_SIZE))
        {
            Some(v) => v,
            None => return None,
        };

        chunk.blocks[y as usize % CHUNK_SIZE][z as usize % CHUNK_SIZE][x as usize % CHUNK_SIZE]
            .as_ref()
    }

    pub fn set_block(&mut self, x: isize, y: isize, z: isize, block: Option<Block>) {
        if x < 0 || y < 0 || z < 0 {
            return;
        }

        let chunk = match self
            .chunks
            .get_mut(y as usize / CHUNK_SIZE)
            .and_then(|chunk_layer| chunk_layer.get_mut(z as usize / CHUNK_SIZE))
            .and_then(|chunk_row| chunk_row.get_mut(x as usize / CHUNK_SIZE))
        {
            Some(v) => v,
            None => return,
        };

        chunk.blocks[y as usize % CHUNK_SIZE][z as usize % CHUNK_SIZE][x as usize % CHUNK_SIZE] =
            block;
    }

    fn calc_scale(vector: Vector3<f32>, scalar: f32) -> f32 {
        if scalar == 0.0 {
            f32::INFINITY
        } else {
            (vector / scalar).magnitude()
        }
    }

    #[allow(dead_code)]
    pub fn raycast(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
    ) -> Option<(Vector3<usize>, Vector3<i32>)> {
        let direction = direction.normalize();
        let scale = Vector3::new(
            Self::calc_scale(direction, direction.x),
            Self::calc_scale(direction, direction.y),
            Self::calc_scale(direction, direction.z),
        );

        let mut position: Vector3<i32> = origin.cast().unwrap();
        let step = direction.map(|x| x.signum() as i32);

        // Truncate the origin
        let mut lengths = Vector3 {
            x: if direction.x < 0.0 {
                (origin.x - position.x as f32) * scale.x
            } else {
                (position.x as f32 + 1.0 - origin.x) * scale.x
            },
            y: if direction.y < 0.0 {
                (origin.y - position.y as f32) * scale.y
            } else {
                (position.y as f32 + 1.0 - origin.y) * scale.y
            },
            z: if direction.z < 0.0 {
                (origin.z - position.z as f32) * scale.z
            } else {
                (position.z as f32 + 1.0 - origin.z) * scale.z
            },
        };

        let mut face;

        while lengths.magnitude() < 100.0 {
            if lengths.x < lengths.y && lengths.x < lengths.z {
                lengths.x += scale.x;
                position.x += step.x;
                face = Vector3::unit_x() * step.x;
            } else if lengths.y < lengths.x && lengths.y < lengths.z {
                lengths.y += scale.y;
                position.y += step.y;
                face = Vector3::unit_y() * step.y;
            } else if lengths.z < lengths.x && lengths.z < lengths.y {
                lengths.z += scale.z;
                position.z += step.z;
                face = Vector3::unit_z() * step.z;
            } else {
                return None;
            }

            if self
                .get_block(
                    position.x as isize,
                    position.y as isize,
                    position.z as isize,
                )
                .is_some()
            {
                // Intersection occurred
                return Some((position.cast().unwrap(), face));
            }
        }

        None
    }
}
