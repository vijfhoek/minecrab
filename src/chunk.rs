use std::{collections::VecDeque, usize};

use crate::{cube, quad::Quad, vertex::Vertex};
use ahash::{AHashMap, AHashSet};
use cgmath::{InnerSpace, Vector3};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Cobblestone,
    Dirt,
    Stone,
    Grass,
    Bedrock,
    Sand,
    Gravel,
}

impl BlockType {
    pub const fn texture_indices(self) -> (usize, usize, usize, usize, usize, usize) {
        #[rustfmt::skip]
        let indices = match self {
            BlockType::Cobblestone => ( 0,  0,  0,  0,  0,  0),
            BlockType::Dirt        => ( 1,  1,  1,  1,  1,  1),
            BlockType::Stone       => ( 2,  2,  2,  2,  2,  2),
            BlockType::Grass       => ( 4,  4,  4,  4,  2,  3),
            BlockType::Bedrock     => ( 5,  5,  5,  5,  5,  5),
            BlockType::Sand        => ( 6,  6,  6,  6,  6,  6),
            BlockType::Gravel      => ( 7,  7,  7,  7,  7,  7),
        };
        indices
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

pub(crate) const CHUNK_SIZE: usize = 16;

type ChunkBlocks = [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct Chunk {
    pub blocks: ChunkBlocks,
    pub highlighted: Option<Vector3<usize>>,
}

impl Chunk {
    pub fn generate(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Self {
        let fbm = noise::Fbm::new();

        const TERRAIN_NOISE_SCALE: f64 = 0.1;
        const TERRAIN_NOISE_OFFSET: f64 = 0.0;
        let terrain_noise = PlaneMapBuilder::new(&fbm)
            .set_size(CHUNK_SIZE, CHUNK_SIZE)
            .set_x_bounds(
                chunk_x as f64 * TERRAIN_NOISE_SCALE + TERRAIN_NOISE_OFFSET,
                chunk_x as f64 * TERRAIN_NOISE_SCALE + TERRAIN_NOISE_SCALE + TERRAIN_NOISE_OFFSET,
            )
            .set_y_bounds(
                chunk_z as f64 * TERRAIN_NOISE_SCALE + TERRAIN_NOISE_OFFSET,
                chunk_z as f64 * TERRAIN_NOISE_SCALE + TERRAIN_NOISE_SCALE + TERRAIN_NOISE_OFFSET,
            )
            .build();

        const STONE_NOISE_SCALE: f64 = 0.07;
        const STONE_NOISE_OFFSET: f64 = 11239.0;
        let stone_noise = PlaneMapBuilder::new(&fbm)
            .set_size(CHUNK_SIZE, CHUNK_SIZE)
            .set_x_bounds(
                chunk_x as f64 * STONE_NOISE_SCALE + STONE_NOISE_OFFSET,
                chunk_x as f64 * STONE_NOISE_SCALE + STONE_NOISE_SCALE + STONE_NOISE_OFFSET,
            )
            .set_y_bounds(
                chunk_z as f64 * STONE_NOISE_SCALE + STONE_NOISE_OFFSET,
                chunk_z as f64 * STONE_NOISE_SCALE + STONE_NOISE_SCALE + STONE_NOISE_OFFSET,
            )
            .build();

        let mut blocks: ChunkBlocks = Default::default();
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = terrain_noise.get_value(x, z) * 20.0 + 64.0;
                let v = v.round() as i32;

                let s = stone_noise.get_value(x, z) * 20.0 + 4.5;
                let s = (s.round() as i32).min(10).max(3);

                let stone_max = (v - s - chunk_y * 16).min(CHUNK_SIZE as i32);
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

                if chunk_y == 0 {
                    blocks[0][z][x] = Some(Block {
                        block_type: BlockType::Bedrock,
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
        self.get_block(x - 1, y - 1, z - 1).is_some()
            && self.get_block(x + 1, y - 1, z - 1).is_some()
            && self.get_block(x - 1, y + 1, z - 1).is_some()
            && self.get_block(x + 1, y + 1, z - 1).is_some()
            && self.get_block(x - 1, y - 1, z + 1).is_some()
            && self.get_block(x + 1, y - 1, z + 1).is_some()
            && self.get_block(x - 1, y + 1, z + 1).is_some()
            && self.get_block(x + 1, y + 1, z + 1).is_some()
    }

    fn cull_layer(
        &self,
        y: usize,
    ) -> (
        AHashMap<(usize, usize), BlockType>,
        VecDeque<(usize, usize)>,
    ) {
        let mut output = AHashMap::new();
        let mut queue = VecDeque::new();

        let y_blocks = &self.blocks[y];
        for (z, z_blocks) in y_blocks.iter().enumerate() {
            for (x, block) in z_blocks.iter().enumerate() {
                if let Some(block) = block {
                    // Don't add the block if it's not visible
                    if self.check_visible(x, y, z) {
                        continue;
                    }

                    output.insert((x, z), block.block_type);
                    queue.push_back((x, z));
                }
            }
        }

        (output, queue)
    }

    fn layer_to_quads(
        y: usize,
        offset: Vector3<i32>,
        culled: AHashMap<(usize, usize), BlockType>,
        queue: &mut VecDeque<(usize, usize)>,
    ) -> Vec<(BlockType, i32, Vector3<i32>, Quad)> {
        let mut quads: Vec<(BlockType, i32, Vector3<i32>, Quad)> = Vec::new();
        let mut visited = AHashSet::new();
        while let Some((x, z)) = queue.pop_front() {
            if visited.contains(&(x, z)) {
                continue;
            }
            visited.insert((x, z));

            if let Some(&block_type) = &culled.get(&(x, z)) {
                // Extend horizontally
                let mut xmax = x + 1;
                for x_ in x..CHUNK_SIZE {
                    xmax = x_ + 1;
                    if culled.get(&(x_ + 1, z)) != Some(&block_type)
                        || visited.contains(&(x_ + 1, z))
                    {
                        break;
                    }
                    visited.insert((x_ + 1, z));
                }

                // Extend vertically
                let mut zmax = z + 1;
                'z: for z_ in z..CHUNK_SIZE {
                    zmax = z_ + 1;
                    for x_ in x..xmax {
                        if culled.get(&(x_, z_ + 1)) != Some(&block_type)
                            || visited.contains(&(x_, z_ + 1))
                        {
                            break 'z;
                        }
                    }
                    for x_ in x..xmax {
                        visited.insert((x_, z_ + 1));
                    }
                }

                let quad = Quad::new(x as i32, z as i32, (xmax - x) as i32, (zmax - z) as i32);
                quads.push((block_type, y as i32, offset, quad));
            }
        }

        quads
    }

    fn quads_to_geometry(
        quads: Vec<(BlockType, i32, Vector3<i32>, Quad)>,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (quad_index, (block_type, y, offset, quad)) in quads.iter().enumerate() {
            #[rustfmt::skip]
            let v = cube::vertices(quad, *y, *offset, block_type.texture_indices());
            vertices.extend(&v);

            for index in cube::INDICES {
                indices.push(index + quad_index as u16 * 24);
            }
        }

        (vertices, indices)
    }

    pub fn to_geometry(&self, offset: Vector3<i32>) -> (Vec<Vertex>, Vec<u16>) {
        let mut quads: Vec<(BlockType, i32, Vector3<i32>, Quad)> = Vec::new();

        for y in 0..CHUNK_SIZE {
            let (culled, mut queue) = self.cull_layer(y);
            let mut layer_quads = Self::layer_to_quads(y, offset, culled, &mut queue);
            quads.append(&mut layer_quads);
        }

        Self::quads_to_geometry(quads)
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

    #[allow(dead_code)]
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
