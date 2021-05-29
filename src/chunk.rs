use crate::instance::Instance;
use cgmath::{InnerSpace, Vector3};
use std::collections::HashMap;

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
        let mut map: HashMap<BlockType, Vec<Instance>> = HashMap::new();

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

    fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Option<&Block>> {
        self.blocks
            .get(y)
            .and_then(|blocks| blocks.get(z))
            .and_then(|blocks| blocks.get(x))
            .map(|block| block.as_ref())
    }

    fn calc_scale(a: Vector3<f32>, b: f32) -> f32 {
        if b == 0.0 {
            f32::INFINITY
        } else {
            (a / b).magnitude()
        }
    }

    pub fn raycast(&self, origin: Vector3<f32>, direction: Vector3<f32>) -> Option<Vector3<usize>> {
        let direction = direction.normalize();

        let scale = Vector3::new(
            Self::calc_scale(direction, direction.x),
            Self::calc_scale(direction, direction.y),
            Self::calc_scale(direction, direction.z),
        );

        let mut position = origin;
        let mut lengths = Vector3::new(0.0, 0.0, 0.0);
        loop {
            let new_lengths = lengths + scale;

            if new_lengths.x < f32::min(new_lengths.y, new_lengths.z) {
                lengths.x = new_lengths.x;
                position += direction * scale.x;
            } else if new_lengths.y < f32::min(new_lengths.x, new_lengths.z) {
                lengths.y = new_lengths.y;
                position += direction * scale.y;
            } else if new_lengths.z < f32::min(new_lengths.x, new_lengths.y) {
                lengths.z = new_lengths.z;
                position += direction * scale.z;
            }

            let position_rounded = position.map(|field| field.round() as usize);
            let block = self.get_block(position_rounded.x, position_rounded.y, position_rounded.z);

            match block {
                None => {
                    // Went outside the chunk, intersection no longer possible
                    break None;
                }
                Some(None) => (),
                Some(Some(_)) => {
                    // Intersected with a block, round position to coordinates and return it.
                    break Some(position_rounded);
                }
            }
        }
    }
}
