use crate::instance::Instance;
use ahash::AHashMap;
use cgmath::{InnerSpace, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Dirt,
    Cobblestone,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    pub blocks: [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub highlighted: Option<Vector3<usize>>,
}

impl Chunk {
    pub fn to_instances(&self) -> Vec<(BlockType, Vec<Instance>)> {
        let mut map: AHashMap<BlockType, Vec<Instance>> = AHashMap::new();

        for (y, y_blocks) in self.blocks.iter().enumerate() {
            for (z, z_blocks) in y_blocks.iter().enumerate() {
                for (x, block) in z_blocks.iter().enumerate() {
                    if let Some(block) = block {
                        let position = Vector3::new(x as f32, y as f32, z as f32);
                        let instances = map.entry(block.block_type).or_default();

                        instances.push(Instance {
                            position: position.into(),
                            highlighted: (self.highlighted == Some(Vector3::new(x, y, z))) as i32,
                        });
                    }
                }
            }
        }

        map.drain().collect()
    }

    pub fn get_mut_block(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        self.blocks
            .get_mut(y)
            .and_then(|blocks| blocks.get_mut(z))
            .and_then(|blocks| blocks.get_mut(x))
            .and_then(|block| block.as_mut())
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        self.blocks
            .get(y)
            .and_then(|blocks| blocks.get(z))
            .and_then(|blocks| blocks.get(x))
            .and_then(|block| block.as_ref())
    }

    fn calc_scale(vector: Vector3<f32>, scalar: f32) -> f32 {
        if scalar == 0.0 {
            f32::INFINITY
        } else {
            (vector / scalar).magnitude()
        }
    }

    pub fn raycast(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
    ) -> Option<(Vector3<usize>, Vector3<i32>)> {
        let scale = Vector3::new(
            Self::calc_scale(direction, direction.x),
            Self::calc_scale(direction, direction.y),
            Self::calc_scale(direction, direction.z),
        );
        let direction = direction.normalize();

        let mut position = origin.map(|x| x as i32);
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
            if lengths.x <= lengths.y && lengths.x <= lengths.z {
                lengths.x += scale.x;
                position.x += step.x;
                face = Vector3::unit_x() * step.x;
            } else if lengths.y <= lengths.x && lengths.y <= lengths.z {
                lengths.y += scale.y;
                position.y += step.y;
                face = Vector3::unit_y() * step.y;
            } else if lengths.z <= lengths.x && lengths.z <= lengths.y {
                lengths.z += scale.z;
                position.z += step.z;
                face = Vector3::unit_z() * step.z;
            } else {
                return None;
            }

            if let Some(_) = self.get_block(
                position.x as usize,
                position.y as usize,
                position.z as usize,
            ) {
                // Intersection occurred
                return Some((position.map(|x| x as usize), face));
            }
        }

        None
    }
}
