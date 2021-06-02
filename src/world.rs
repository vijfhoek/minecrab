use std::collections::HashMap;

use crate::{
    chunk::{Block, Chunk, CHUNK_ISIZE, CHUNK_SIZE},
    geometry::Geometry,
    npc::Npc,
    vertex::BlockVertex,
};
use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};
use rayon::prelude::*;

pub struct World {
    pub chunks: HashMap<Point3<isize>, Chunk>,
    pub npc: Npc,
}

const WORLD_SIZE: Vector3<usize> = Vector3::new(
    8 * 16 / CHUNK_SIZE,
    16 * 16 / CHUNK_SIZE,
    8 * 16 / CHUNK_SIZE,
);

impl World {
    pub fn generate() -> Self {
        let npc = Npc::load();
        let half: Vector3<isize> = WORLD_SIZE.cast().unwrap() / 2;

        let coords: Vec<_> =
            itertools::iproduct!(-half.x..half.x, 0..WORLD_SIZE.y as isize, -half.z..half.z)
                .collect();

        let chunks: HashMap<_, _> = coords
            .par_iter()
            .map(|&(x, y, z)| {
                (
                    Point3::new(x, y, z),
                    Chunk::generate(x as i32, y as i32, z as i32),
                )
            })
            .collect();

        Self { chunks, npc }
    }

    pub fn highlighted_for_chunk(
        highlighted: Option<(Point3<isize>, Vector3<i32>)>,
        chunk_position: &Point3<isize>,
    ) -> Option<(Point3<usize>, Vector3<i32>)> {
        let position = chunk_position * CHUNK_ISIZE;
        if let Some((pos, face)) = highlighted {
            if pos.x >= position.x
                && pos.x < position.x + CHUNK_ISIZE
                && pos.y >= position.y
                && pos.y < position.y + CHUNK_ISIZE
                && pos.z >= position.z
                && pos.z < position.z + CHUNK_ISIZE
            {
                let point: Point3<isize> = EuclideanSpace::from_vec(pos - position);
                Some((point.cast().unwrap(), face))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_geometry(
        &self,
        highlighted: Option<(Point3<isize>, Vector3<i32>)>,
    ) -> Vec<(Point3<isize>, Geometry<BlockVertex>)> {
        let instant = std::time::Instant::now();

        let chunks = &self.chunks;
        let geometry = chunks
            .par_iter()
            .map(|(chunk_position, chunk)| {
                let position = (chunk_position * CHUNK_ISIZE).cast().unwrap();
                let h = Self::highlighted_for_chunk(highlighted, chunk_position);
                let geometry = chunk.to_geometry(position, h.as_ref());
                (*chunk_position, geometry)
            })
            .collect();

        let elapsed = instant.elapsed();
        println!("Generating world geometry took {:?}", elapsed);

        geometry
    }

    pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<&Block> {
        let chunk = match self.chunks.get(&Point3::new(
            x.div_euclid(CHUNK_ISIZE),
            y.div_euclid(CHUNK_ISIZE),
            z.div_euclid(CHUNK_ISIZE),
        )) {
            Some(chunk) => chunk,
            None => return None,
        };

        let bx = x.rem_euclid(CHUNK_ISIZE) as usize;
        let by = y.rem_euclid(CHUNK_ISIZE) as usize;
        let bz = z.rem_euclid(CHUNK_ISIZE) as usize;
        chunk.blocks[by][bz][bx].as_ref()
    }

    pub fn set_block(&mut self, x: isize, y: isize, z: isize, block: Option<Block>) {
        if let Some(chunk) = self.chunks.get_mut(&Point3::new(
            x.div_euclid(CHUNK_ISIZE),
            y.div_euclid(CHUNK_ISIZE),
            z.div_euclid(CHUNK_ISIZE),
        )) {
            let bx = x.rem_euclid(CHUNK_ISIZE) as usize;
            let by = y.rem_euclid(CHUNK_ISIZE) as usize;
            let bz = z.rem_euclid(CHUNK_ISIZE) as usize;
            chunk.blocks[by][bz][bx] = block;
        }
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
        origin: Point3<f32>,
        direction: Vector3<f32>,
    ) -> Option<(Point3<isize>, Vector3<i32>)> {
        let direction = direction.normalize();
        let scale = Vector3::new(
            Self::calc_scale(direction, direction.x),
            Self::calc_scale(direction, direction.y),
            Self::calc_scale(direction, direction.z),
        );

        let mut position: Point3<i32> = origin.map(|x| x.floor() as i32);
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
                face = Vector3::unit_x() * -step.x;
            } else if lengths.y < lengths.x && lengths.y < lengths.z {
                lengths.y += scale.y;
                position.y += step.y;
                face = Vector3::unit_y() * -step.y;
            } else if lengths.z < lengths.x && lengths.z < lengths.y {
                lengths.z += scale.z;
                position.z += step.z;
                face = Vector3::unit_z() * -step.z;
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
