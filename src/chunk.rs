use std::{collections::VecDeque, convert::TryInto, usize};

use crate::{cube, quad::Quad, vertex::Vertex};
use ahash::{AHashMap, AHashSet};
use cgmath::{Vector3, Zero};
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
    Water,
}

impl BlockType {
    #[rustfmt::skip]
    pub const fn texture_indices(self) -> (usize, usize, usize, usize, usize, usize) {
        match self {
            BlockType::Cobblestone => ( 0,  0,  0,  0,  0,  0),
            BlockType::Dirt        => ( 1,  1,  1,  1,  1,  1),
            BlockType::Stone       => ( 2,  2,  2,  2,  2,  2),
            BlockType::Grass       => ( 4,  4,  4,  4,  2,  3),
            BlockType::Bedrock     => ( 5,  5,  5,  5,  5,  5),
            BlockType::Sand        => ( 6,  6,  6,  6,  6,  6),
            BlockType::Gravel      => ( 7,  7,  7,  7,  7,  7),
            BlockType::Water       => ( 8,  8,  8,  8,  8,  8), // up to 71
        }
    }

    pub const fn is_transparent(self) -> bool {
        matches!(self, BlockType::Water)
    }
}

pub type FaceFlags = usize;
pub const FACE_NONE: FaceFlags = 0;
pub const FACE_LEFT: FaceFlags = 1;
pub const FACE_RIGHT: FaceFlags = 2;
pub const FACE_BOTTOM: FaceFlags = 4;
pub const FACE_TOP: FaceFlags = 8;
pub const FACE_BACK: FaceFlags = 16;
pub const FACE_FRONT: FaceFlags = 32;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

pub const CHUNK_SIZE: usize = 64;

type ChunkBlocks = [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct Chunk {
    pub blocks: ChunkBlocks,
}

impl Chunk {
    pub fn generate(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Self {
        let fbm = noise::Fbm::new();

        const TERRAIN_NOISE_SCALE: f64 = 0.1 / 16.0 * CHUNK_SIZE as f64;
        const TERRAIN_NOISE_OFFSET: f64 = 0.0 / 16.0 * CHUNK_SIZE as f64;
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

        const STONE_NOISE_SCALE: f64 = 0.07 / 16.0 * CHUNK_SIZE as f64;
        const STONE_NOISE_OFFSET: f64 = 11239.0 / 16.0 * CHUNK_SIZE as f64;
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

        let mut blocks: ChunkBlocks = [[[Default::default(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = terrain_noise.get_value(x, z) * 20.0 + 128.0;
                let v = v.round() as i32;

                let s = stone_noise.get_value(x, z) * 20.0 + 4.5;
                let s = (s.round() as i32).min(10).max(3);

                let stone_max = (v - s - chunk_y * CHUNK_SIZE as i32).min(CHUNK_SIZE as i32);
                for y in 0..stone_max {
                    blocks[y as usize][z][x] = Some(Block {
                        block_type: BlockType::Stone,
                    });
                }

                let dirt_max = (v - chunk_y * CHUNK_SIZE as i32).min(CHUNK_SIZE as i32);
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
                if chunk_y < 128 / CHUNK_SIZE as i32 {
                    for layer in blocks.iter_mut() {
                        if layer[z][x].is_none() {
                            layer[z][x] = Some(Block {
                                block_type: BlockType::Water,
                            });
                        }
                    }
                }
            }
        }

        Self { blocks }
    }

    #[rustfmt::skip]
    fn check_visible_faces(&self, x: usize, y: usize, z: usize) -> FaceFlags {
        let mut visible_faces = FACE_NONE;
        let transparent = self.blocks[y][z][x].unwrap().block_type.is_transparent();

        if x == 0 || self.blocks[y][z][x - 1].is_none()
            || transparent != self.blocks[y][z][x - 1].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_LEFT;
        }
        if x == CHUNK_SIZE - 1 || self.blocks[y][z][x + 1].is_none()
            || transparent != self.blocks[y][z][x + 1].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_RIGHT;
        }

        if y == 0 || self.blocks[y - 1][z][x].is_none()
            || transparent != self.blocks[y - 1][z][x].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_BOTTOM;
        }
        if y == CHUNK_SIZE - 1 || self.blocks[y + 1][z][x].is_none()
            || transparent != self.blocks[y + 1][z][x].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_TOP;
        }

        if z == 0 || self.blocks[y][z - 1][x].is_none()
            || transparent != self.blocks[y][z - 1][x].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_BACK;
        }
        if z == CHUNK_SIZE - 1 || self.blocks[y][z + 1][x].is_none()
            || transparent != self.blocks[y][z + 1][x].unwrap().block_type.is_transparent()
        {
            visible_faces |= FACE_FRONT;
        }

        visible_faces
    }

    fn cull_layer(
        &self,
        y: usize,
    ) -> (
        AHashMap<(usize, usize), (BlockType, FaceFlags)>,
        VecDeque<(usize, usize)>,
    ) {
        let mut culled = AHashMap::new();
        let mut queue = VecDeque::new();

        let y_blocks = &self.blocks[y];
        for (z, z_blocks) in y_blocks.iter().enumerate() {
            for (x, block) in z_blocks.iter().enumerate() {
                if let Some(block) = block {
                    // Don't add the block if it's not visible
                    let visible_faces = self.check_visible_faces(x, y, z);
                    if visible_faces == FACE_NONE {
                        continue;
                    }

                    culled.insert((x, z), (block.block_type, visible_faces));
                    queue.push_back((x, z));
                }
            }
        }

        (culled, queue)
    }

    fn layer_to_quads(
        &self,
        y: usize,
        offset: Vector3<i32>,
        culled: AHashMap<(usize, usize), (BlockType, FaceFlags)>,
        queue: &mut VecDeque<(usize, usize)>,
        highlighted: Option<&(Vector3<usize>, Vector3<i32>)>,
    ) -> Vec<(BlockType, i32, Vector3<i32>, Quad, Vector3<i32>, FaceFlags)> {
        let mut quads: Vec<(BlockType, i32, Vector3<i32>, Quad, Vector3<i32>, FaceFlags)> =
            Vec::new();
        let mut visited = AHashSet::new();
        let hl = highlighted.map(|h| h.0);
        while let Some((x, z)) = queue.pop_front() {
            if visited.contains(&(x, z)) {
                continue;
            }
            visited.insert((x, z));

            if let Some(&(block_type, visible_faces)) = &culled.get(&(x, z)) {
                let mut quad_faces = visible_faces;

                if hl == Some(Vector3::new(x, y, z)) {
                    let quad = Quad::new(x as i32, z as i32, 1, 1);
                    quads.push((
                        block_type,
                        y as i32,
                        offset,
                        quad,
                        highlighted.unwrap().1,
                        quad_faces,
                    ));
                    continue;
                }

                if block_type == BlockType::Water {
                    let quad = Quad::new(x as i32, z as i32, 1, 1);
                    quads.push((
                        block_type,
                        y as i32,
                        offset,
                        quad,
                        Vector3::zero(),
                        quad_faces,
                    ));
                    continue;
                }

                // Extend along the X axis
                let mut xmax = x + 1;
                for x_ in x..CHUNK_SIZE {
                    xmax = x_ + 1;

                    if visited.contains(&(xmax, z)) || hl == Some(Vector3::new(xmax, y, z)) {
                        break;
                    }

                    if let Some(&(block_type_, visible_faces_)) = culled.get(&(xmax, z)) {
                        quad_faces |= visible_faces_;
                        if block_type != block_type_ {
                            break;
                        }
                    } else {
                        break;
                    }

                    visited.insert((xmax, z));
                }

                // Extend along the Z axis
                let mut zmax = z + 1;
                'z: for z_ in z..CHUNK_SIZE {
                    zmax = z_ + 1;

                    for x_ in x..xmax {
                        if visited.contains(&(x_, zmax)) || hl == Some(Vector3::new(x_, y, zmax)) {
                            break 'z;
                        }

                        if let Some(&(block_type_, visible_faces_)) = culled.get(&(x_, zmax)) {
                            quad_faces |= visible_faces_;
                            if block_type != block_type_ {
                                break 'z;
                            }
                        } else {
                            break 'z;
                        }
                    }

                    for x_ in x..xmax {
                        visited.insert((x_, zmax));
                    }
                }

                let quad = Quad::new(x as i32, z as i32, (xmax - x) as i32, (zmax - z) as i32);
                quads.push((
                    block_type,
                    y as i32,
                    offset,
                    quad,
                    Vector3::zero(),
                    quad_faces,
                ));
            }
        }

        quads
    }

    fn quads_to_geometry(
        quads: Vec<(BlockType, i32, Vector3<i32>, Quad, Vector3<i32>, FaceFlags)>,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (block_type, y, offset, quad, highlighted, visible_faces) in quads {
            let texture_indices = block_type.texture_indices();

            let (quad_vertices, quad_indices) = &cube::vertices(
                &quad,
                y,
                1.0,
                offset,
                texture_indices,
                highlighted,
                visible_faces,
                vertices.len().try_into().unwrap(),
            );

            vertices.extend(quad_vertices);
            indices.extend(quad_indices);
        }

        (vertices, indices)
    }

    pub fn to_geometry(
        &self,
        offset: Vector3<i32>,
        highlighted: Option<&(Vector3<usize>, Vector3<i32>)>,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut quads: Vec<(BlockType, i32, Vector3<i32>, Quad, Vector3<i32>, FaceFlags)> =
            Vec::new();

        for y in 0..CHUNK_SIZE {
            let (culled, mut queue) = self.cull_layer(y);
            let mut layer_quads = self.layer_to_quads(y, offset, culled, &mut queue, highlighted);
            quads.append(&mut layer_quads);
        }

        Self::quads_to_geometry(quads)
    }
}
