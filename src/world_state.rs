use std::time::Instant;

use cgmath::Vector3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;

use crate::{
    camera::{Camera, Projection},
    light::Light,
    texture::{Texture, TextureManager},
    uniforms::Uniforms,
    vertex::Vertex,
    world::World,
};

pub struct WorldState {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub texture_manager: TextureManager,
    pub camera: Camera,
    pub projection: Projection,
    pub depth_texture: Texture,
    pub light_bind_group: wgpu::BindGroup,
    pub world: World,

    pub chunk_buffers: Vec<(
        wgpu::Buffer,
        wgpu::Buffer,
        Vec<Vec<(usize, usize, usize)>>,
        usize,
    )>,
}

impl WorldState {
    fn create_textures(render_device: &wgpu::Device, render_queue: &wgpu::Queue) -> TextureManager {
        let mut texture_manager = TextureManager::new(render_device);
        texture_manager
            .load_all(render_device, render_queue)
            .unwrap();
        texture_manager
    }

    fn create_camera(swap_chain_descriptor: &wgpu::SwapChainDescriptor) -> (Camera, Projection) {
        let camera = Camera::new(
            (0.0, 80.0, 0.0).into(),
            cgmath::Deg(0.0).into(),
            cgmath::Deg(-20.0).into(),
        );

        let projection = Projection::new(
            swap_chain_descriptor.width,
            swap_chain_descriptor.height,
            cgmath::Deg(45.0),
            0.1,
            500.0,
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
        let light = Light::new(
            Vector3::new(256.0, 500.0, 200.0),
            Vector3::new(1.0, 1.0, 1.0),
        );

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
                label: Some("light_bind_group_layout"),
            });

        let light_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
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

        let wireframe = false;

        render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
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
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: if wireframe {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
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
        })
    }

    pub fn update_chunk(&mut self, render_device: &wgpu::Device) {
        let instant = Instant::now();

        let world_geometry = self.world.to_instances();
        self.chunk_buffers.clear();
        for (chunk_vertices, chunk_indices, index_textures) in world_geometry {
            self.chunk_buffers.push((
                render_device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(&chunk_vertices),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
                render_device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(&chunk_indices),
                    usage: wgpu::BufferUsage::INDEX,
                }),
                index_textures,
                chunk_indices.len(),
            ));
        }

        let elapsed = instant.elapsed();
        println!("World update took {:?}", elapsed);
    }

    pub fn new(
        render_device: &wgpu::Device,
        render_queue: &wgpu::Queue,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
    ) -> WorldState {
        let world = World::generate();

        let texture_manager = Self::create_textures(render_device, render_queue);

        let (camera, projection) = Self::create_camera(swap_chain_descriptor);

        let (uniforms, uniform_buffer, world_uniform_layout, uniform_bind_group) =
            Self::create_uniforms(&camera, &projection, render_device);

        let (_, _, world_light_layout, light_bind_group) = Self::create_light(&render_device);

        let render_pipeline = Self::create_render_pipeline(
            &render_device,
            &swap_chain_descriptor,
            &[
                &texture_manager.bind_group_layout,
                &world_uniform_layout,
                &world_light_layout,
            ],
        );

        let depth_texture =
            Texture::create_depth_texture(&render_device, &swap_chain_descriptor, "depth_texture");

        let mut world_state = Self {
            render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            texture_manager,
            camera,
            projection,
            depth_texture,
            light_bind_group,
            world,
            chunk_buffers: Vec::new(),
        };

        world_state.update_chunk(render_device);

        world_state
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
