use crate::instance::Instance;
use ahash::AHashMap;
use cgmath::{InnerSpace, Vector3};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Cobblestone,
    Dirt,
    Grass,
    Stone,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

const CHUNK_SIZE: usize = 16;

type ChunkBlocks = [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct Chunk {
    pub blocks: ChunkBlocks,
    pub highlighted: Option<Vector3<usize>>,
}

impl Chunk {
    pub fn generate(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Self {
        let fbm = Fbm::new();

        let builder = PlaneMapBuilder::new(&fbm)
            .set_size(16, 16)
            .set_x_bounds(chunk_x as f64 * 0.2, chunk_x as f64 * 0.2 + 0.2)
            .set_y_bounds(chunk_z as f64 * 0.2, chunk_z as f64 * 0.2 + 0.2)
            .build();

        let mut blocks: ChunkBlocks = Default::default();
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = builder.get_value(x, z) * 10.0 + 64.0;
                let v = v.round() as i32;

                let stone_max = (v - 4 - chunk_y * 16).min(CHUNK_SIZE as i32);
                for y in 0..stone_max {
                    blocks[y as usize][z][x] = Some(Block {
                        block_type: BlockType::Stone,
                    });
                }

                let dirt_max = (v - chunk_y * 16).min(CHUNK_SIZE as i32);
                for y in stone_max.max(0)..dirt_max {
                    blocks[y as usize][z][x] = Some(Block {
                        block_type: BlockType::Dirt,
                    });
                }

                if dirt_max >= 0 && dirt_max < CHUNK_SIZE as i32 {
                    blocks[dirt_max as usize][z][x] = Some(Block {
                        block_type: BlockType::Grass,
                    });
                }
            }
        }

        Self {
            blocks,
            highlighted: None,
        }
    }

    fn check_visible(&self, x: usize, y: usize, z: usize) -> bool {
        (x > 0 && y > 0 && z > 0 && self.get_block(x - 1, y - 1, z - 1).is_some())
            && (y > 0 && z > 0 && self.get_block(x + 1, y - 1, z - 1).is_some())
            && (x > 0 && z > 0 && self.get_block(x - 1, y + 1, z - 1).is_some())
            && (z > 0 && self.get_block(x + 1, y + 1, z - 1).is_some())
            && (x > 0 && y > 0 && self.get_block(x - 1, y - 1, z + 1).is_some())
            && (y > 0 && self.get_block(x + 1, y - 1, z + 1).is_some())
            && (x > 0 && self.get_block(x - 1, y + 1, z + 1).is_some())
            && (x > 0 && y > 0 && z > 0 && self.get_block(x + 1, y + 1, z + 1).is_some())
    }

    pub fn to_instances(&self, offset: Vector3<i32>) -> Vec<(BlockType, Vec<Instance>)> {
        let mut map: AHashMap<BlockType, Vec<Instance>> = AHashMap::new();

        for (y, y_blocks) in self.blocks.iter().enumerate() {
            for (z, z_blocks) in y_blocks.iter().enumerate() {
                for (x, block) in z_blocks.iter().enumerate() {
                    if let Some(block) = block {
                        let position = Vector3::new(
                            (x as i32 + offset.x * 16) as f32,
                            (y as i32 + offset.y * 16) as f32,
                            (z as i32 + offset.z * 16) as f32,
                        );

                        // Don't add the block if it's not visible
                        if self.check_visible(x, y, z) {
                            continue;
                        }

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

            if self
                .get_block(
                    position.x as usize,
                    position.y as usize,
                    position.z as usize,
                )
                .is_some()
            {
                // Intersection occurred
                return Some((position.map(|x| x as usize), face));
            }
        }

        None
    }
}
