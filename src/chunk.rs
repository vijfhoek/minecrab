use crate::instance::Instance;
use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Dirt,
    Cobblestone,
}

impl BlockType {
    pub fn texture_index(self) -> u32 {
        match self {
            Self::Dirt => 0,
            Self::Cobblestone => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

const CHUNK_SIZE: usize = 16;

pub struct Chunk {
    pub blocks: [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn to_instances(&self) -> Vec<(BlockType, Vec<Instance>)> {
        let mut map: HashMap<BlockType, Vec<Instance>> = HashMap::new();

        for (y, y_blocks) in self.blocks.iter().enumerate() {
            for (z, z_blocks) in y_blocks.iter().enumerate() {
                for (x, block) in z_blocks.iter().enumerate() {
                    if let Some(block) = block {
                        let position = Vector3 {
                            x: x as f32,
                            y: y as f32,
                            z: z as f32,
                        };

                        let rotation = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0));

                        let instances = map.entry(block.block_type).or_default();

                        instances.push(Instance {
                            position,
                            rotation,
                            block_type: block.block_type,
                        });
                    }
                }
            }
        }

        map.drain().collect()
    }

    fn calc_scale(a: Vector3<f32>, b: f32) -> f32 {
        if b == 0.0 {
            f32::INFINITY
        } else {
            (a / b).magnitude()
        }
    }

    pub fn dda(&self, position: Vector3<f32>, direction: Vector3<f32>) -> Option<Vector3<usize>> {
        assert!(f32::abs(direction.magnitude() - 1.0) < f32::EPSILON);

        let scale = Vector3::new(
            Self::calc_scale(direction, direction.x),
            Self::calc_scale(direction, direction.y),
            Self::calc_scale(direction, direction.z),
        );

        let mut new_position = position;
        let mut lengths = Vector3::new(0.0, 0.0, 0.0);
        loop {
            let new_lengths = lengths + scale;

            if new_lengths.x < f32::min(new_lengths.y, new_lengths.z) {
                lengths.x = new_lengths.x;
                new_position += direction * scale.x;
            } else if new_lengths.y < f32::min(new_lengths.x, new_lengths.z) {
                lengths.y = new_lengths.y;
                new_position += direction * scale.y;
            } else if new_lengths.z < f32::min(new_lengths.x, new_lengths.y) {
                lengths.z = new_lengths.z;
                new_position += direction * scale.z;
            }

            let pos_usize = new_position.map(|field| field.round() as usize);
            let block = self
                .blocks
                .get(pos_usize.y)
                .and_then(|a| a.get(pos_usize.z))
                .and_then(|a| a.get(pos_usize.x));

            match block {
                None => {
                    // Went outside the chunk, intersection no longer possible
                    break None;
                }
                Some(None) => (),
                Some(Some(_)) => {
                    // Intersected with a block, round position to coordinates and return it.
                    break Some(pos_usize);
                }
            }
        }
    }
}
