use std::collections::{HashMap, VecDeque};

use crate::{
    camera::Camera,
    chunk::{Block, BlockType, Chunk, CHUNK_ISIZE},
    geometry::GeometryBuffers,
    npc::Npc,
    render_context::RenderContext,
};
use ahash::AHashMap;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector2, Vector3};
use wgpu::{BufferUsage, RenderPass};

pub struct World {
    pub chunks: HashMap<Point3<isize>, Chunk>,
    pub npc: Npc,

    pub chunk_database: sled::Db,
    pub chunk_save_queue: VecDeque<Point3<isize>>,
    pub chunk_load_queue: VecDeque<Point3<isize>>,
    pub chunk_generate_queue: VecDeque<Point3<isize>>,
    pub chunk_buffers: AHashMap<Point3<isize>, GeometryBuffers<u16>>,

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

        self.update_highlight(render_context, camera);

        // Queue up new chunks for loading, if necessary
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

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &Camera) -> usize {
        let camera_pos = camera.position.to_vec();
        let camera_pos = Vector2::new(camera_pos.x, camera_pos.z);
        let mut triangle_count = 0;

        for (position, buffers) in &self.chunk_buffers {
            let pos = (position * CHUNK_ISIZE).cast().unwrap();
            let pos = Vector2::new(pos.x, pos.z);
            if (pos - camera_pos).magnitude() > 300.0 {
                continue;
            }

            buffers.set_buffers(render_pass);
            triangle_count += buffers.draw_indexed(render_pass);
        }

        {
            let buffers = self.npc.geometry_buffers.as_ref().unwrap();
            buffers.set_buffers(render_pass);
            triangle_count += buffers.draw_indexed(render_pass);
        }

        triangle_count
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

    fn update_highlight(&mut self, render_context: &RenderContext, camera: &Camera) {
        let old = self.highlighted;
        let new = self.raycast(camera.position, camera.direction());

        let old_chunk = old.map(|(pos, _)| pos.map(|n| n.div_euclid(CHUNK_ISIZE)));
        let new_chunk = new.map(|(pos, _)| pos.map(|n| n.div_euclid(CHUNK_ISIZE)));

        if old != new {
            self.highlighted = new;

            if let Some(old_chunk_) = old_chunk {
                self.update_chunk_geometry(render_context, old_chunk_);
            }

            if let Some(new_chunk_) = new_chunk {
                // Don't update the same chunk twice
                if old_chunk != new_chunk {
                    self.update_chunk_geometry(render_context, new_chunk_);
                }
            }
        }
    }

    pub fn break_at_crosshair(&mut self, render_context: &RenderContext, camera: &Camera) {
        if let Some((pos, _)) = self.raycast(camera.position, camera.direction()) {
            self.set_block(pos.x as isize, pos.y as isize, pos.z as isize, None);
            self.update_chunk_geometry(render_context, pos / CHUNK_ISIZE);
        }
    }

    pub fn place_at_crosshair(&mut self, render_context: &RenderContext, camera: &Camera) {
        if let Some((pos, face_normal)) = self.raycast(camera.position, camera.direction()) {
            let new_pos = pos.cast().unwrap() + face_normal;

            self.set_block(
                new_pos.x as isize,
                new_pos.y as isize,
                new_pos.z as isize,
                Some(Block {
                    block_type: BlockType::Cobblestone,
                }),
            );

            self.update_chunk_geometry(render_context, pos / CHUNK_ISIZE);
        }
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
