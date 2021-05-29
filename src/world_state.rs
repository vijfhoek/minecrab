use std::collections::HashMap;

use cgmath::EuclideanSpace;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;

use crate::{
    camera::{Camera, Projection},
    chunk::{Block, BlockType, Chunk},
    cube,
    instance::Instance,
    light::Light,
    texture::Texture,
    uniforms::Uniforms,
    vertex::Vertex,
};

pub struct WorldState {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub texture_bind_groups: HashMap<BlockType, wgpu::BindGroup>,
    pub camera: Camera,
    pub projection: Projection,
    pub instance_lists: Vec<(BlockType, Vec<Instance>)>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffers: Vec<(BlockType, wgpu::Buffer)>,
    pub depth_texture: Texture,
    pub light_bind_group: wgpu::BindGroup,
}

impl WorldState {
    fn create_textures(
        render_device: &wgpu::Device,
        render_queue: &wgpu::Queue,
    ) -> (wgpu::BindGroupLayout, HashMap<BlockType, wgpu::BindGroup>) {
        let dirt_texture = Texture::from_bytes(
            render_device,
            render_queue,
            include_bytes!("../assets/block/dirt.png"),
            "dirt",
        )
        .unwrap();

        let cobblestone_texture = Texture::from_bytes(
            render_device,
            render_queue,
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

    fn create_render_pipeline(
        render_device: &wgpu::Device,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = render_device.create_shader_module(
            &(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/world.wgsl").into()),
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
                buffers: &[Vertex::desc(), Instance::desc()],
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
                (
                    *block_type,
                    render_device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("instance_buffer"),
                        contents: bytemuck::cast_slice(&instance_list),
                        usage: wgpu::BufferUsage::VERTEX,
                    }),
                )
            })
            .collect();

        (instance_lists, instance_buffers)
    }

    pub fn new(
        render_device: &wgpu::Device,
        queue: &wgpu::Queue,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
    ) -> WorldState {
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

        let (world_texture_layout, texture_bind_groups) =
            Self::create_textures(&render_device, &queue);

        let (camera, projection) = Self::create_camera(&swap_chain_descriptor);

        let pointy_at = chunk
            .dda(camera.position.to_vec(), camera.direction())
            .unwrap();

        chunk.blocks[pointy_at.y][pointy_at.z][pointy_at.x] = Some(Block {
            block_type: BlockType::Cobblestone,
        });

        let (uniforms, uniform_buffer, world_uniform_layout, uniform_bind_group) =
            Self::create_uniforms(&camera, &projection, &render_device);

        let (_, _, world_light_layout, light_bind_group) = Self::create_light(&render_device);

        let render_pipeline = Self::create_render_pipeline(
            &render_device,
            &swap_chain_descriptor,
            &[
                &world_texture_layout,
                &world_uniform_layout,
                &world_light_layout,
            ],
        );

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

        WorldState {
            render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_groups,
            camera,
            projection,
            light_bind_group,
            vertex_buffer,
            index_buffer,
            instance_lists,
            instance_buffers,
            depth_texture,
        }
    }

    pub fn resize(
        &mut self,
        render_device: &wgpu::Device,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
        new_size: PhysicalSize<u32>,
    ) {
        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture =
            Texture::create_depth_texture(render_device, swap_chain_descriptor, "depth_texture");
    }
}
