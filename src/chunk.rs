use std::{collections::VecDeque, usize};

use crate::{geometry::Geometry, quad::Quad, vertex::BlockVertex};
use ahash::{AHashMap, AHashSet};
use cgmath::{Point3, Vector3};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use serde::{
    de::{SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BlockType {
    Cobblestone = 1,
    Dirt = 2,
    Stone = 3,
    Grass = 4,
    Bedrock = 5,
    Sand = 6,
    Gravel = 7,
    Water = 8,
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
pub const FACE_ALL: FaceFlags =
    FACE_LEFT | FACE_RIGHT | FACE_BOTTOM | FACE_TOP | FACE_BACK | FACE_FRONT;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Block {
    pub block_type: BlockType,
}

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_ISIZE: isize = CHUNK_SIZE as isize;

type ChunkBlocks = [[[Option<Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(Clone, Default)]
pub struct Chunk {
    pub blocks: ChunkBlocks,
}

impl Serialize for Chunk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(CHUNK_SIZE.pow(3)))?;
        for layer in self.blocks.iter() {
            for row in layer {
                for block in row {
                    seq.serialize_element(block)?;
                }
            }
        }
        seq.end()
    }
}

struct ChunkVisitor;

impl<'de> Visitor<'de> for ChunkVisitor {
    type Value = Chunk;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a chunk")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut chunk = Chunk::default();
        for layer in chunk.blocks.iter_mut() {
            for row in layer {
                for block in row {
                    *block = seq.next_element()?.unwrap();
                }
            }
        }

        Ok(chunk)
    }
}

impl<'de> Deserialize<'de> for Chunk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChunkVisitor)
    }
}

impl Chunk {
    pub fn generate(&mut self, chunk_x: isize, chunk_y: isize, chunk_z: isize) {
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

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = terrain_noise.get_value(x, z) * 20.0 + 128.0;
                let v = v.round() as isize;

                let s = stone_noise.get_value(x, z) * 20.0 + 4.5;
                let s = (s.round() as isize).min(10).max(3);

                let stone_max = (v - s - chunk_y * CHUNK_ISIZE).min(CHUNK_ISIZE);
                for y in 0..stone_max {
                    self.blocks[y as usize][z][x] = Some(Block {
                        block_type: BlockType::Stone,
                    });
                }

                let dirt_max = (v - chunk_y * CHUNK_ISIZE).min(CHUNK_ISIZE);
                for y in stone_max.max(0)..dirt_max {
                    self.blocks[y as usize][z][x] = Some(Block {
                        block_type: BlockType::Dirt,
                    });
                }

                if dirt_max >= 0 && dirt_max < CHUNK_ISIZE {
                    self.blocks[dirt_max as usize][z][x] = Some(Block {
                        block_type: BlockType::Grass,
                    });
                }

                if chunk_y == 0 {
                    self.blocks[0][z][x] = Some(Block {
                        block_type: BlockType::Bedrock,
                    });
                }
                if chunk_y < 128 / CHUNK_ISIZE {
                    for layer in self.blocks.iter_mut() {
                        if layer[z][x].is_none() {
                            layer[z][x] = Some(Block {
                                block_type: BlockType::Water,
                            });
                        }
                    }
                }
            }
        }
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
        offset: Point3<isize>,
        culled: AHashMap<(usize, usize), (BlockType, FaceFlags)>,
        queue: &mut VecDeque<(usize, usize)>,
        highlighted: Option<&(Point3<usize>, Vector3<i32>)>,
    ) -> Vec<Quad> {
        let mut quads: Vec<Quad> = Vec::new();
        let mut visited = AHashSet::new();
        let hl = highlighted.map(|h| h.0);
        while let Some((x, z)) = queue.pop_front() {
            let position = offset + Vector3::new(x, y, z).cast().unwrap();

            if visited.contains(&(x, z)) {
                continue;
            }
            visited.insert((x, z));

            if let Some(&(block_type, visible_faces)) = &culled.get(&(x, z)) {
                let mut quad_faces = visible_faces;

                if hl == Some(Point3::new(x, y, z)) {
                    let mut quad = Quad::new(position, 1, 1);
                    quad.highlighted_normal = highlighted.unwrap().1;
                    quad.visible_faces = quad_faces;
                    quad.block_type = Some(block_type);
                    quads.push(quad);
                    continue;
                }

                if block_type == BlockType::Water {
                    let mut quad = Quad::new(position, 1, 1);
                    quad.visible_faces = quad_faces;
                    quad.block_type = Some(block_type);
                    quads.push(quad);
                    continue;
                }

                // Extend along the X axis
                let mut xmax = x + 1;
                for x_ in x..CHUNK_SIZE {
                    xmax = x_ + 1;

                    if visited.contains(&(xmax, z)) || hl == Some(Point3::new(xmax, y, z)) {
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
                        if visited.contains(&(x_, zmax)) || hl == Some(Point3::new(x_, y, zmax)) {
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

                let mut quad = Quad::new(position, (xmax - x) as isize, (zmax - z) as isize);
                quad.visible_faces = quad_faces;
                quad.block_type = Some(block_type);
                quads.push(quad);
            }
        }

        quads
    }

    fn quads_to_geometry(quads: Vec<Quad>) -> Geometry<BlockVertex> {
        let mut geometry: Geometry<BlockVertex> = Default::default();
        for quad in quads {
            geometry.append(&mut quad.to_geometry(geometry.vertices.len() as u16));
        }
        geometry
    }

    pub fn to_geometry(
        &self,
        position: Point3<isize>,
        highlighted: Option<&(Point3<usize>, Vector3<i32>)>,
    ) -> Geometry<BlockVertex> {
        let quads: Vec<Quad> = (0..CHUNK_SIZE)
            .into_par_iter()
            .flat_map(|y| {
                let (culled, mut queue) = self.cull_layer(y);
                self.layer_to_quads(y, position, culled, &mut queue, highlighted)
            })
            .collect();

        Self::quads_to_geometry(quads)
    }

    pub fn save(&self, position: Point3<isize>, store: &sled::Db) -> anyhow::Result<()> {
        let data = rmp_serde::encode::to_vec_named(self)?;
        let key = format!("{}_{}_{}", position.x, position.y, position.z);
        store.insert(key, data)?;
        Ok(())
    }

    pub fn load(&mut self, position: Point3<isize>, store: &sled::Db) -> anyhow::Result<bool> {
        let key = format!("{}_{}_{}", position.x, position.y, position.z);

        if let Some(data) = store.get(key)? {
            *self = rmp_serde::decode::from_slice(&data)?;
            Ok(false)
        } else {
            self.generate(position.x, position.y, position.z);
            Ok(true)
        }
    }
}
