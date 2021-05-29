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

        let scale_x = Self::calc_scale(direction, direction.x);
        let scale_y = Self::calc_scale(direction, direction.y);
        let scale_z = Self::calc_scale(direction, direction.z);
        dbg!(direction, scale_x, scale_y, scale_z);

        let mut new_position = position;

        let mut x_length = 0.0;
        let mut y_length = 0.0;
        let mut z_length = 0.0;
        loop {
            let new_x_length = x_length + scale_x;
            let new_y_length = y_length + scale_y;
            let new_z_length = z_length + scale_z;

            if new_x_length < f32::min(new_y_length, new_z_length) {
                x_length = new_x_length;
                new_position += direction * scale_x;
            } else if new_y_length < f32::min(new_x_length, new_z_length) {
                y_length = new_y_length;
                new_position += direction * scale_y;
            } else if new_z_length < f32::min(new_x_length, new_y_length) {
                z_length = new_z_length;
                new_position += direction * scale_z;
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
