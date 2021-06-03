use std::collections::{HashMap, VecDeque};

use crate::{
    camera::Camera,
    chunk::{Block, Chunk, CHUNK_ISIZE},
    geometry::{Geometry, GeometryBuffers},
    npc::Npc,
    render_context::RenderContext,
    vertex::BlockVertex,
};
use ahash::AHashMap;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::BufferUsage;

pub struct World {
    pub chunks: HashMap<Point3<isize>, Chunk>,
    pub npc: Npc,

    pub chunk_database: sled::Db,
    pub chunk_save_queue: VecDeque<Point3<isize>>,
    pub chunk_load_queue: VecDeque<Point3<isize>>,
    pub chunk_generate_queue: VecDeque<Point3<isize>>,
    pub chunk_buffers: AHashMap<Point3<isize>, GeometryBuffers>,

    pub highlighted: Option<(Point3<isize>, Vector3<i32>)>,
}

pub const RENDER_DISTANCE: isize = 8;
pub const WORLD_HEIGHT: isize = 16 * 16 / CHUNK_ISIZE;

impl World {
    pub fn new() -> Self {
        let chunks = HashMap::new();
        let npc = Npc::load();

        let chunk_database = sled::Config::new()
            .path("chunks")
            .mode(sled::Mode::HighThroughput)
            .use_compression(true)
            .open()
            .unwrap();

        Self {
            chunks,
            npc,

            chunk_database,
            chunk_load_queue: VecDeque::new(),
            chunk_save_queue: VecDeque::new(),
            chunk_generate_queue: VecDeque::new(),
            chunk_buffers: AHashMap::new(),

            highlighted: None,
        }
    }

    pub fn update(&mut self, render_context: &RenderContext, camera: &Camera) {
        if let Some(position) = self.chunk_load_queue.pop_front() {
            let chunk = self.chunks.entry(position).or_default();
            match chunk.load(position, &self.chunk_database) {
                Err(error) => {
                    eprintln!("Failed to load/generate chunk {:?}: {:?}", position, error)
                }
                Ok(true) => {
                    self.update_chunk_geometry(render_context, position);
                    self.chunk_save_queue.push_back(position);
                    // println!("Generated chunk {:?}", position);
                }
                Ok(false) => {
                    self.update_chunk_geometry(render_context, position);
                    // println!("Loaded chunk {:?}", position);
                }
            }
        } else if let Some(position) = self.chunk_save_queue.pop_front() {
            let chunk = self.chunks.get(&position).unwrap();
            if let Err(err) = chunk.save(position, &self.chunk_database) {
                eprintln!("Failed to save chunk {:?}: {:?}", position, err);
            } else {
                // println!("Saved chunk {:?}", position);
            }
        }

        // Load new chunks, if necessary
        let camera_pos: Point3<isize> = camera.position.cast().unwrap();
        let camera_chunk: Point3<isize> = camera_pos.map(|n| n.div_euclid(CHUNK_ISIZE));
        let mut load_queue = Vec::new();
        for (x, y, z) in itertools::iproduct!(
            -RENDER_DISTANCE..RENDER_DISTANCE,
            0..WORLD_HEIGHT,
            -RENDER_DISTANCE..RENDER_DISTANCE
        ) {
            let point: Point3<isize> = Point3::new(x + camera_chunk.x, y, z + camera_chunk.z);
            if !self.chunks.contains_key(&point) && !self.chunk_load_queue.contains(&point) {
                load_queue.push(point);
            }
        }

        // TODO Sort based on where camera is looking
        load_queue.sort_unstable_by_key(|f| {
            (f.x * CHUNK_ISIZE - camera_pos.x).abs() + (f.y * CHUNK_ISIZE - camera_pos.y).abs()
        });

        self.chunk_load_queue.extend(load_queue);
    }

    pub fn update_chunk_geometry(
        &mut self,
        render_context: &RenderContext,
        chunk_position: Point3<isize>,
    ) {
        let chunk = &mut self.chunks.get(&chunk_position).unwrap();
        let offset = chunk_position * CHUNK_ISIZE;
        let geometry = chunk.to_geometry(
            offset,
            World::highlighted_for_chunk(self.highlighted, &chunk_position).as_ref(),
        );

        let buffers =
            GeometryBuffers::from_geometry(render_context, &geometry, BufferUsage::empty());
        self.chunk_buffers.insert(chunk_position, buffers);
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
        let chunk_position = Point3::new(
            x.div_euclid(CHUNK_ISIZE),
            y.div_euclid(CHUNK_ISIZE),
            z.div_euclid(CHUNK_ISIZE),
        );

        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            let bx = x.rem_euclid(CHUNK_ISIZE) as usize;
            let by = y.rem_euclid(CHUNK_ISIZE) as usize;
            let bz = z.rem_euclid(CHUNK_ISIZE) as usize;
            chunk.blocks[by][bz][bx] = block;
        }

        self.chunk_save_queue
            .push_back(chunk_position / CHUNK_ISIZE);
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
