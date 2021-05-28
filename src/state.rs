use cgmath::Rotation3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::Camera,
    cube,
    instance::{Instance, InstanceRaw},
    light::Light,
    texture::Texture,
    uniforms::Uniforms,
    vertex::Vertex,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;

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
    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    light: Light,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
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
                    features: wgpu::Features::NON_FILL_POLYGON_MODE,
                    limits: wgpu::Limits::default(),
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
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let texture = Texture::from_bytes(
            render_device,
            queue,
            include_bytes!("../assets/block/cobblestone.png"),
            "dirt_diffuse",
        )
        .unwrap();

        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
            });

        let texture_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dirt_diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        (texture_bind_group_layout, texture_bind_group)
    }

    fn create_camera(swap_chain_descriptor: &wgpu::SwapChainDescriptor) -> Camera {
        Camera {
            eye: (0.0, 5.0, 10.0).into(), // position the camera one unit up and 2 units back
            target: (0.0, 0.0, 0.0).into(), // have it look at the origin
            up: cgmath::Vector3::unit_y(),
            aspect: swap_chain_descriptor.width as f32 / swap_chain_descriptor.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn create_uniforms(
        camera: &Camera,
        render_device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(camera);

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

        (uniform_bind_group_layout, uniform_bind_group)
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

    fn create_instances(render_device: &wgpu::Device) -> (Vec<Instance>, wgpu::Buffer) {
        let instances = (0..NUM_INSTANCES_PER_ROW as i32)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW as i32).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: (x - NUM_INSTANCES_PER_ROW as i32 / 2) as f32 * 2.0,
                        y: 0.0,
                        z: (z - NUM_INSTANCES_PER_ROW as i32 / 2) as f32 * 2.0,
                    };

                    let rotation = cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    );

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        (instances, instance_buffer)
    }

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let (render_surface, adapter, render_device, queue) =
            Self::create_render_device(window).await;

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(window, &adapter, &render_device, &render_surface);

        let (texture_layout, texture_bind_group) = Self::create_textures(&render_device, &queue);

        let camera = Self::create_camera(&swap_chain_descriptor);

        let (uniform_layout, uniform_bind_group) = Self::create_uniforms(&camera, &render_device);

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

        let (instances, instance_buffer) = Self::create_instances(&render_device);

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
            uniform_bind_group,
            texture_bind_group,
            instances,
            instance_buffer,
            depth_texture,
            light,
            light_buffer,
            light_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("resizing to {:?}", new_size);
        self.size = new_size;
        self.swap_chain_descriptor.width = new_size.width;
        self.swap_chain_descriptor.height = new_size.height;

        self.depth_texture = Texture::create_depth_texture(
            &self.render_device,
            &self.swap_chain_descriptor,
            "depth_texture",
        );

        self.swap_chain = self
            .render_device
            .create_swap_chain(&self.render_surface, &self.swap_chain_descriptor);
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        let old_position: cgmath::Vector3<_> = self.light.position.into();
        self.light.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
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

            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(
                0..cube::INDICES.len() as u32,
                0,
                0..self.instances.len() as u32,
            );
        }

        self.queue.submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
