use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use cgmath::{EuclideanSpace, InnerSpace, Rad};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::{
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode},
    window::Window,
};

use crate::{
    camera::{Camera, Projection},
    chunk::{Block, BlockType, Chunk},
    cube,
    instance::{Instance, InstanceRaw},
    light::Light,
    texture::Texture,
    uniforms::Uniforms,
    vertex::Vertex,
};

pub struct State {
    render_surface: wgpu::Surface,
    render_device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_groups: HashMap<BlockType, wgpu::BindGroup>,
    uniform_bind_group: wgpu::BindGroup,
    instance_lists: Vec<(BlockType, Vec<Instance>)>,
    instance_buffers: Vec<(BlockType, wgpu::Buffer)>,
    depth_texture: Texture,
    _light: Light,
    _light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    right_speed: f32,
    forward_speed: f32,
    up_speed: f32,
    camera: Camera,
    uniforms: Uniforms,
    projection: Projection,
    uniform_buffer: wgpu::Buffer,
    pub mouse_grabbed: bool,
}

impl State {
    fn create_render_pipeline(
        render_device: &wgpu::Device,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = render_device.create_shader_module(
            &(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            }),
        );

        let render_pipeline_layout =
            render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swap_chain_descriptor.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        })
    }

    async fn create_render_device(
        window: &Window,
    ) -> (wgpu::Surface, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let render_surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&render_surface),
            })
            .await
            .unwrap();

        let (render_device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("render_device"),
                    features: wgpu::Features::NON_FILL_POLYGON_MODE
                        | wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY,
                    limits: wgpu::Limits {
                        max_push_constant_size: 4,
                        ..wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        (render_surface, adapter, render_device, queue)
    }

    fn create_swap_chain(
        window: &Window,
        adapter: &wgpu::Adapter,
        render_device: &wgpu::Device,
        render_surface: &wgpu::Surface,
    ) -> (wgpu::SwapChainDescriptor, wgpu::SwapChain) {
        let size = window.inner_size();

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(render_surface)
                .unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = render_device.create_swap_chain(&render_surface, &swap_chain_descriptor);

        (swap_chain_descriptor, swap_chain)
    }

    fn create_textures(
        render_device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::BindGroupLayout, HashMap<BlockType, wgpu::BindGroup>) {
        let dirt_texture = Texture::from_bytes(
            render_device,
            queue,
            include_bytes!("../assets/block/dirt.png"),
            "dirt",
        )
        .unwrap();

        let cobblestone_texture = Texture::from_bytes(
            render_device,
            queue,
            include_bytes!("../assets/block/cobblestone.png"),
            "cobblestone",
        )
        .unwrap();

        let sampler = render_device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let bind_groups: HashMap<BlockType, wgpu::BindGroup> = [
            (BlockType::Dirt, dirt_texture),
            (BlockType::Cobblestone, cobblestone_texture),
        ]
        .iter()
        .map(|(block_type, texture)| {
            (
                *block_type,
                render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("texture_bind_group"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                }),
            )
        })
        .collect();

        (bind_group_layout, bind_groups)
    }

    fn create_camera(swap_chain_descriptor: &wgpu::SwapChainDescriptor) -> (Camera, Projection) {
        let camera = Camera::new(
            (0.0, 5.0, 10.0).into(),
            cgmath::Deg(0.0).into(),
            cgmath::Deg(-20.0).into(),
        );

        let projection = Projection::new(
            swap_chain_descriptor.width,
            swap_chain_descriptor.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );

        (camera, projection)
    }

    fn create_uniforms(
        camera: &Camera,
        projection: &Projection,
        render_device: &wgpu::Device,
    ) -> (
        Uniforms,
        wgpu::Buffer,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    ) {
        let mut uniforms = Uniforms::new();
        uniforms.update_view_projection(camera, projection);

        let uniform_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        (
            uniforms,
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
        )
    }

    fn create_light(
        render_device: &wgpu::Device,
    ) -> (Light, wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let light = Light {
            position: [5.0, 5.0, 5.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };

        let light_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("light_buffer"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: None,
            });

        let light_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        (
            light,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
        )
    }

    fn create_instances(
        render_device: &wgpu::Device,
        chunk: &Chunk,
    ) -> (
        Vec<(BlockType, Vec<Instance>)>,
        Vec<(BlockType, wgpu::Buffer)>,
    ) {
        let instance_lists = chunk.to_instances();

        let instance_buffers = instance_lists
            .iter()
            .map(|(block_type, instance_list)| {
                let instance_data: Vec<InstanceRaw> =
                    instance_list.iter().map(Instance::to_raw).collect();

                (
                    *block_type,
                    render_device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("instance_buffer"),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsage::VERTEX,
                    }),
                )
            })
            .collect();

        (instance_lists, instance_buffers)
    }

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let mut chunk = Chunk {
            blocks: [
                [[Some(Block {
                    block_type: BlockType::Cobblestone,
                }); 16]; 16],
                [[Some(Block {
                    block_type: BlockType::Dirt,
                }); 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
                [[None; 16]; 16],
            ],
        };

        let (render_surface, adapter, render_device, queue) =
            Self::create_render_device(window).await;

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(window, &adapter, &render_device, &render_surface);

        let (texture_layout, texture_bind_groups) = Self::create_textures(&render_device, &queue);

        let (camera, projection) = Self::create_camera(&swap_chain_descriptor);

        let pointy_at = chunk
            .dda(camera.position.to_vec(), camera.direction())
            .unwrap();

        chunk.blocks[pointy_at.y][pointy_at.z][pointy_at.x] = Some(Block {
            block_type: BlockType::Cobblestone,
        });

        let (uniforms, uniform_buffer, uniform_layout, uniform_bind_group) =
            Self::create_uniforms(&camera, &projection, &render_device);

        let (light, light_buffer, light_layout, light_bind_group) =
            Self::create_light(&render_device);

        let layouts = [&texture_layout, &uniform_layout, &light_layout];
        let render_pipeline =
            Self::create_render_pipeline(&render_device, &swap_chain_descriptor, &layouts);

        let vertex_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(cube::VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(cube::INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let (instance_lists, instance_buffers) = Self::create_instances(&render_device, &chunk);

        let depth_texture =
            Texture::create_depth_texture(&render_device, &swap_chain_descriptor, "depth_texture");

        Self {
            render_surface,
            render_device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_groups,
            camera,
            projection,
            instance_lists,
            instance_buffers,
            depth_texture,
            _light: light,
            _light_buffer: light_buffer,
            light_bind_group,
            mouse_grabbed: false,

            right_speed: 0.0,
            forward_speed: 0.0,
            up_speed: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("resizing to {:?}", new_size);
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.width;
        self.swap_chain_descriptor.height = new_size.height;

        self.projection.resize(new_size.width, new_size.height);

        self.depth_texture = Texture::create_depth_texture(
            &self.render_device,
            &self.swap_chain_descriptor,
            "depth_texture",
        );

        self.swap_chain = self
            .render_device
            .create_swap_chain(&self.render_surface, &self.swap_chain_descriptor);
    }

    fn input_keyboard(&mut self, key_code: &VirtualKeyCode, state: &ElementState) {
        let amount = if state == &ElementState::Pressed {
            1.0
        } else {
            -1.0
        };

        match key_code {
            VirtualKeyCode::W => self.forward_speed += amount,
            VirtualKeyCode::S => self.forward_speed -= amount,
            VirtualKeyCode::A => self.right_speed -= amount,
            VirtualKeyCode::D => self.right_speed += amount,
            VirtualKeyCode::LControl => self.up_speed -= amount,
            VirtualKeyCode::Space => self.up_speed += amount,
            _ => (),
        }
    }

    fn update_camera(&mut self, dx: f64, dy: f64) {
        self.camera.yaw += Rad(dx as f32 * 0.005);
        self.camera.pitch -= Rad(dy as f32 * 0.005);

        if self.camera.pitch < Rad::from(cgmath::Deg(-80.0)) {
            self.camera.pitch = Rad::from(cgmath::Deg(-80.0));
        } else if self.camera.pitch > Rad::from(cgmath::Deg(89.0)) {
            self.camera.pitch = Rad::from(cgmath::Deg(89.0));
        }
    }

    fn update_aim(&mut self) {}

    fn input_mouse(&mut self, dx: f64, dy: f64) {
        if self.mouse_grabbed {
            self.update_camera(dx, dy);
            self.update_aim();
        }
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.input_keyboard(key, state),
            DeviceEvent::MouseMotion { delta: (dx, dy) } => self.input_mouse(*dx, *dy),
            _ => (),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.camera.yaw.0.sin_cos();

        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        self.camera.position += forward * self.forward_speed * 6.0 * dt_secs;

        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.camera.position += right * self.right_speed * 6.0 * dt_secs;

        let up = cgmath::Vector3::new(0.0, 1.0, 0.0).normalize();
        self.camera.position += up * self.up_speed * 6.0 * dt_secs;

        self.uniforms
            .update_view_projection(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut render_encoder =
            self.render_device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        {
            let mut render_pass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
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

            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for (i, (block_type, instance_list)) in self.instance_lists.iter().enumerate() {
                let (_, instance_buffer) = &self.instance_buffers[i];

                let texture_bind_group = &self.texture_bind_groups[block_type];
                render_pass.set_bind_group(0, texture_bind_group, &[]);

                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..cube::INDICES.len() as u32,
                    0,
                    0..instance_list.len() as u32,
                );
            }
        }

        self.queue.submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
