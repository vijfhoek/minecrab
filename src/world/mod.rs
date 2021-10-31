pub mod block;
pub mod chunk;
pub mod face_flags;
pub mod npc;
pub mod quad;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::{
    camera::Camera,
    render_context::RenderContext,
    texture::Texture,
    time::Time,
    vertex::{BlockVertex, Vertex},
    view::View,
    world::{
        block::{Block, BlockType},
        chunk::{Chunk, CHUNK_ISIZE, CHUNK_SIZE},
        npc::Npc,
    },
};
use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};
use fxhash::FxHashMap;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, CommandEncoder, RenderPipeline, SwapChainTexture,
};
use cgmath::num_traits::Inv;

pub struct World {
    pub render_pipeline: RenderPipeline,
    pub depth_texture: Texture,

    pub time: Time,
    pub time_buffer: Buffer,
    pub time_bind_group: BindGroup,

    pub npc: Npc,

    pub chunks: FxHashMap<Point3<isize>, Chunk>,
    pub chunk_database: sled::Db,
    pub chunk_save_queue: VecDeque<(Point3<isize>, bool)>,
    pub chunk_load_queue: VecDeque<Point3<isize>>,
    pub chunk_generate_queue: VecDeque<Point3<isize>>,
    pub chunk_occlusion_position: Option<Point3<isize>>,
    pub chunks_visible: Option<Vec<Point3<isize>>>,

    pub highlighted: Option<(Point3<isize>, Vector3<i32>)>,

    pub unload_timer: Duration,
}

pub const RENDER_DISTANCE: isize = 8;
pub const WORLD_HEIGHT: isize = 16 * 16 / CHUNK_ISIZE;

const DEBUG_IO: bool = false;

impl World {
    #[allow(clippy::collapsible_else_if)]
    pub fn update(
        &mut self,
        render_context: &RenderContext,
        dt: Duration,
        render_time: Duration,
        camera: &Camera,
    ) {
        self.time.time += dt.as_secs_f32();
        render_context.queue.write_buffer(
            &self.time_buffer,
            0,
            &bytemuck::cast_slice(&[self.time]),
        );

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

        // Unload chunks that are far away
        self.unload_timer += dt;
        if self.unload_timer.as_secs() >= 10 {
            self.unload_timer = Duration::ZERO;

            let camera_pos = camera.position.to_vec();
            let unload_distance = (RENDER_DISTANCE * CHUNK_ISIZE) as f32 * 1.5;

            let mut unload_chunks = Vec::new();
            for point in self.chunks.keys() {
                let pos: Point3<f32> = (point * CHUNK_ISIZE).cast().unwrap();
                if (pos.x - camera_pos.x).abs() > unload_distance
                    || (pos.z - camera_pos.z).abs() > unload_distance
                {
                    unload_chunks.push(*point);
                }
            }
            for point in unload_chunks {
                self.enqueue_chunk_save(point, true);
            }
        }

        let start = Instant::now() - render_time;
        let mut chunk_updates = 0;
        while chunk_updates == 0 || start.elapsed() < Duration::from_millis(15) {
            if let Some(position) = self.chunk_load_queue.pop_front() {
                let chunk = self.chunks.entry(position).or_default();
                match chunk.load(position, &self.chunk_database) {
                    Err(error) => {
                        eprintln!("Failed to load/generate chunk {:?}: {:?}", position, error)
                    }
                    Ok(true) => {
                        self.update_chunk_geometry(render_context, position);
                        self.enqueue_chunk_save(position, false);
                        if DEBUG_IO {
                            println!("Generated chunk {:?}", position);
                        }
                    }
                    Ok(false) => {
                        self.update_chunk_geometry(render_context, position);
                        if DEBUG_IO {
                            println!("Loaded chunk {:?}", position);
                        }
                    }
                }
            } else if let Some((position, unload)) = self.chunk_save_queue.pop_front() {
                if let Some(chunk) = self.chunks.get(&position) {
                    if let Err(err) = chunk.save(position, &self.chunk_database) {
                        eprintln!("Failed to save chunk {:?}: {:?}", position, err);
                    } else {
                        if unload {
                            self.chunks.remove(&position);

                            if DEBUG_IO {
                                println!("Saved and unloaded chunk {:?}", position);
                            }
                        } else {
                            if DEBUG_IO {
                                println!("Saved chunk {:?}", position);
                            }
                        }
                    }
                } else {
                    eprintln!("Tried to save unloaded chunk {:?}", position);
                }
            } else {
                break;
            }

            chunk_updates += 1;
        }

        if chunk_updates > 0 {
            self.chunk_occlusion_position = None;
        }
    }

    pub fn render<'a>(
        &'a mut self,
        render_context: &RenderContext,
        render_encoder: &mut CommandEncoder,
        frame: &SwapChainTexture,
        view: &View,
    ) -> usize {
        // TODO Move this to update
        self.update_occlusion(view);

        let mut render_pass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.502,
                        g: 0.663,
                        b: 0.965,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipeline);

        let texture_manager = render_context.texture_manager.as_ref().unwrap();
        render_pass.set_bind_group(0, texture_manager.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_bind_group(1, &view.bind_group, &[]);
        render_pass.set_bind_group(2, &self.time_bind_group, &[]);

        let visible = self.chunks_visible.as_ref().unwrap();
        let mut triangle_count = 0;
        for position in visible {
            let chunk = self.chunks.get(position).unwrap();
            triangle_count += chunk.render(&mut render_pass, &position, view);
        }
        triangle_count += self.npc.render(&mut render_pass);
        triangle_count
    }

    pub fn new(render_context: &RenderContext, view: &View) -> Self {
        let chunks = FxHashMap::default();
        let mut npc = Npc::new();
        npc.load_geometry(render_context);

        let chunk_database = sled::Config::new()
            .path("chunks")
            .mode(sled::Mode::HighThroughput)
            .use_compression(true)
            .open()
            .unwrap();

        let time = Time::new();

        let time_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("time_buffer"),
                contents: bytemuck::cast_slice(&[time]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let time_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("time_bind_group_layout"),
                });

        let time_bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &time_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                }],
                label: Some("time_bind_group"),
            });

        let texture_manager = render_context.texture_manager.as_ref().unwrap();
        let render_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[
                        &texture_manager.bind_group_layout,
                        &view.bind_group_layout,
                        &time_bind_group_layout,
                    ],
                });

        let shader = render_context.device.create_shader_module(
            &(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/world.wgsl").into()),
            }),
        );

        let render_pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "main",
                        buffers: &[BlockVertex::descriptor()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: render_context.swap_chain_descriptor.format,
                            blend: Some(wgpu::BlendState {
                                alpha: wgpu::BlendComponent::REPLACE,
                                color: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrite::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                });

        let depth_texture = Texture::create_depth_texture(render_context, "depth_texture");

        Self {
            render_pipeline,

            time,
            time_buffer,
            time_bind_group,

            depth_texture,

            npc,

            chunks,
            chunk_database,
            chunk_load_queue: VecDeque::new(),
            chunk_save_queue: VecDeque::new(),
            chunk_generate_queue: VecDeque::new(),
            chunk_occlusion_position: None,
            chunks_visible: None,

            highlighted: None,

            unload_timer: Duration::ZERO,
        }
    }

    pub fn update_occlusion(&mut self, view: &View) {
        let initial_position = view
            .camera
            .position
            .map(|x| (x.floor() as isize).div_euclid(CHUNK_ISIZE));

        if self.chunk_occlusion_position == Some(initial_position) {
            return;
        }

        self.chunk_occlusion_position = Some(initial_position);
        let mut queue = VecDeque::from(vec![initial_position]);

        assert_eq!(CHUNK_SIZE, 32);
        let mut visited = [0u32; CHUNK_SIZE * CHUNK_SIZE];
        let mut render_queue = Vec::new();

        while !queue.is_empty() {
            let position = queue.pop_front().unwrap();

            let b = position.map(|x| x.rem_euclid(CHUNK_ISIZE) as usize);
            if (visited[b.x * CHUNK_SIZE + b.y] >> b.z) & 1 == 1 {
                continue;
            }
            visited[b.x * CHUNK_SIZE + b.y] |= 1 << b.z;

            if let Some(chunk) = self.chunks.get(&position) {
                render_queue.push(position);
                if !chunk.full {
                    queue.extend([
                        position + Vector3::unit_x(),
                        position - Vector3::unit_x(),
                        position + Vector3::unit_y(),
                        position - Vector3::unit_y(),
                        position + Vector3::unit_z(),
                        position - Vector3::unit_z(),
                    ]);
                }
            }
        }

        self.chunks_visible = Some(render_queue);
    }

    pub fn enqueue_chunk_save(&mut self, position: Point3<isize>, unload: bool) {
        if let Some((_, unload_)) = self
            .chunk_save_queue
            .iter_mut()
            .find(|(pos, _)| pos == &position)
        {
            *unload_ = *unload_ || unload;
        } else {
            self.chunk_save_queue.push_back((position, unload));
        }
    }

    pub fn update_chunk_geometry(
        &mut self,
        render_context: &RenderContext,
        chunk_position: Point3<isize>,
    ) {
        let chunk = self.chunks.get_mut(&chunk_position).unwrap();
        chunk.update_geometry(render_context, chunk_position, self.highlighted);
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

    pub fn place_at_crosshair(
        &mut self,
        render_context: &RenderContext,
        camera: &Camera,
        block_type: BlockType,
    ) {
        if let Some((pos, face_normal)) = self.raycast(camera.position, camera.direction()) {
            let new_pos = (pos.cast().unwrap() + face_normal).cast().unwrap();
            self.set_block(new_pos.x, new_pos.y, new_pos.z, Some(Block { block_type }));
            self.update_chunk_geometry(render_context, pos / CHUNK_ISIZE);
        }
    }

    pub fn get_block(&self, point: Point3<isize>) -> Option<&Block> {
        let chunk = match self.chunks.get(&point.map(|x| x.div_euclid(CHUNK_ISIZE))) {
            Some(chunk) => chunk,
            None => return None,
        };

        let b = point.map(|x| x.rem_euclid(CHUNK_ISIZE) as usize);
        chunk.blocks[b.y][b.z][b.x].as_ref()
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

        self.enqueue_chunk_save(chunk_position, false);
    }

    #[allow(dead_code)]
    pub fn raycast(
        &self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
    ) -> Option<(Point3<isize>, Vector3<i32>)> {
        let direction = direction.normalize();
        let mut position: Point3<i32> = origin.map(|x| x.floor() as i32);
        let step = direction.map(|x| x.signum() as i32);

        // Algorithm from: http://www.cse.yorku.ca/%7Eamana/research/grid.pdf
        fn dif_from_next(n: f32, n_step: i32) -> f32 {
            if n_step < 0 {
                // Difference between the next smallest integer and n
                (n).floor() - n
            } else {
                // Difference between the next biggest integer and n
                (n + 1.0).floor() - n
            }
        }

        let mut t_max_x = dif_from_next(origin.x, step.x) / direction.x;
        let mut t_max_y = dif_from_next(origin.y, step.y) / direction.y;
        let mut t_max_z = dif_from_next(origin.z, step.z) / direction.z;

        let t_delta_x = direction.x.abs().inv();
        let t_delta_y = direction.y.abs().inv();
        let t_delta_z = direction.z.abs().inv();

        let mut face;

        while t_max_x.min(t_max_y).min(t_max_z) < 100.0 {
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    t_max_x += t_delta_x;
                    position.x += step.x;
                    face = Vector3::unit_x() * -step.x;
                } else {
                    t_max_z += t_delta_z;
                    position.z += step.z;
                    face = Vector3::unit_z() * -step.z;
                }
            } else {
                if t_max_y < t_max_z {
                    t_max_y += t_delta_y;
                    position.y += step.y;
                    face = Vector3::unit_y() * -step.y;
                } else {
                    t_max_z += t_delta_z;
                    position.z += step.z;
                    face = Vector3::unit_z() * -step.z;
                }
            }

            if self.get_block(position.cast().unwrap()).is_some() {
                // Intersection occurred
                return Some((position.cast().unwrap(), face));
            }
        }

        None
    }
}
