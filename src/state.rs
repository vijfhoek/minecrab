use std::time::Duration;

use cgmath::{EuclideanSpace, InnerSpace, Rad};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::{
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode},
    window::Window,
};

use crate::{cube, texture::Texture, vertex::Vertex, world_state::WorldState};

const UI_SCALE_X: f32 = 0.0045;
const UI_SCALE_Y: f32 = 0.008;

pub const CROSSHAIR_VERTICES: &[Vertex] = &[
    // Crosshair
    Vertex {
        position: [-UI_SCALE_X * 8.0, UI_SCALE_Y * 8.0, 0.0],
        texture_coordinates: [240.0 / 256.0, 0.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [UI_SCALE_X * 8.0, UI_SCALE_Y * 8.0, 0.0],
        texture_coordinates: [1.0, 0.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [UI_SCALE_X * 8.0, -UI_SCALE_Y * 8.0, 0.0],
        texture_coordinates: [1.0, 16.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-UI_SCALE_X * 8.0, -UI_SCALE_Y * 8.0, 0.0],
        texture_coordinates: [240.0 / 256.0, 16.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    // Hotbar
    Vertex {
        position: [-UI_SCALE_X * 91.0, -1.0 + UI_SCALE_Y * 22.0, 0.0],
        texture_coordinates: [0.0 / 256.0, 0.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [UI_SCALE_X * 91.0, -1.0 + UI_SCALE_Y * 22.0, 0.0],
        texture_coordinates: [182.0 / 256.0, 0.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [UI_SCALE_X * 91.0, -1.0, 0.0],
        texture_coordinates: [182.0 / 256.0, 22.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-UI_SCALE_X * 91.0, -1.0, 0.0],
        texture_coordinates: [0.0 / 256.0, 22.0 / 256.0],
        normal: [0.0, 0.0, 0.0],
    },
];

#[rustfmt::skip]
pub const CROSSHAIR_INDICES: &[u16] = &[
    1, 0, 3,
    1, 3, 2,

    5, 4, 7,
    5, 7, 6,
];

pub struct State {
    pub window_size: winit::dpi::PhysicalSize<u32>,
    render_surface: wgpu::Surface,
    render_device: wgpu::Device,
    render_queue: wgpu::Queue,

    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    world_state: WorldState,

    ui_texture_bind_group: wgpu::BindGroup,
    ui_render_pipeline: wgpu::RenderPipeline,
    ui_crosshair_vertex_buffer: wgpu::Buffer,
    ui_crosshair_index_buffer: wgpu::Buffer,

    right_speed: f32,
    forward_speed: f32,
    up_speed: f32,
    pub mouse_grabbed: bool,
}

impl State {
    fn create_ui_textures(
        render_device: &wgpu::Device,
        render_queue: &wgpu::Queue,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let texture = Texture::from_bytes(
            render_device,
            render_queue,
            include_bytes!("../assets/gui/widgets.png"),
            "Texture GUI widgets",
        )
        .unwrap();

        let sampler = render_device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        let bind_group_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GUI texture bind group layout"),
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

        let bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GUI texture bind group"),
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
        });

        (bind_group_layout, bind_group)
    }

    fn create_ui_render_pipeline(
        render_device: &wgpu::Device,
        swap_chain_descriptor: &wgpu::SwapChainDescriptor,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let module = &render_device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("UI shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        let pipeline_layout =
            render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("UI render pipeline layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swap_chain_descriptor.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
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
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = render_device.create_swap_chain(&render_surface, &swap_chain_descriptor);

        (swap_chain_descriptor, swap_chain)
    }

    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();

        let (render_surface, render_adapter, render_device, render_queue) =
            Self::create_render_device(window).await;

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(window, &render_adapter, &render_device, &render_surface);

        let world_state = WorldState::new(&render_device, &render_queue, &swap_chain_descriptor);

        let (ui_texture_bind_group_layout, ui_texture_bind_group) =
            Self::create_ui_textures(&render_device, &render_queue);
        let ui_render_pipeline = Self::create_ui_render_pipeline(
            &render_device,
            &swap_chain_descriptor,
            &[&ui_texture_bind_group_layout],
        );

        let ui_crosshair_vertex_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("GUI crosshair vertex buffer"),
            contents: bytemuck::cast_slice(CROSSHAIR_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let ui_crosshair_index_buffer = render_device.create_buffer_init(&BufferInitDescriptor {
            label: Some("GUI crosshair index buffer"),
            contents: bytemuck::cast_slice(CROSSHAIR_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        Self {
            window_size,
            render_surface,
            render_device,
            render_queue,

            swap_chain_descriptor,
            swap_chain,

            world_state,

            ui_render_pipeline,
            ui_texture_bind_group,
            ui_crosshair_vertex_buffer,
            ui_crosshair_index_buffer,

            right_speed: 0.0,
            forward_speed: 0.0,
            up_speed: 0.0,
            mouse_grabbed: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("resizing to {:?}", new_size);
        self.window_size = new_size;
        self.swap_chain_descriptor.width = new_size.width;
        self.swap_chain_descriptor.height = new_size.height;

        self.world_state
            .resize(&self.render_device, &self.swap_chain_descriptor, new_size);

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
        let camera = &mut self.world_state.camera;
        camera.yaw += Rad(dx as f32 * 0.003);
        camera.pitch -= Rad(dy as f32 * 0.003);

        if camera.pitch < Rad::from(cgmath::Deg(-80.0)) {
            camera.pitch = Rad::from(cgmath::Deg(-80.0));
        } else if camera.pitch > Rad::from(cgmath::Deg(89.9)) {
            camera.pitch = Rad::from(cgmath::Deg(89.9));
        }
    }

    fn update_aim(&mut self) {
        let camera = &self.world_state.camera;
        let chunk = &mut self.world_state.chunk;

        let coords = chunk.dda(camera.position.to_vec(), camera.direction());
        if coords != chunk.highlighted {
            chunk.highlighted = coords;
            self.world_state.update_chunk(&self.render_queue);
        }
    }

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
        let (yaw_sin, yaw_cos) = self.world_state.camera.yaw.0.sin_cos();

        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        self.world_state.camera.position += forward * self.forward_speed * 6.0 * dt_secs;

        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.world_state.camera.position += right * self.right_speed * 6.0 * dt_secs;

        let up = cgmath::Vector3::new(0.0, 1.0, 0.0).normalize();
        self.world_state.camera.position += up * self.up_speed * 6.0 * dt_secs;

        self.world_state
            .uniforms
            .update_view_projection(&self.world_state.camera, &self.world_state.projection);
        self.render_queue.write_buffer(
            &self.world_state.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.world_state.uniforms]),
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
                    view: &self.world_state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.world_state.render_pipeline);

            render_pass.set_bind_group(1, &self.world_state.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.world_state.light_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.world_state.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.world_state.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            for (block_type, instance_list) in &self.world_state.instance_lists {
                let instance_buffer = &self.world_state.instance_buffers[block_type];

                let texture_bind_group = &self.world_state.texture_bind_groups[block_type];
                render_pass.set_bind_group(0, texture_bind_group, &[]);

                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..cube::INDICES.len() as u32,
                    0,
                    0..instance_list.len() as u32,
                );
            }
        }

        {
            let mut render_pass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.ui_render_pipeline);
            render_pass.set_vertex_buffer(0, self.ui_crosshair_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.ui_crosshair_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.set_bind_group(0, &self.ui_texture_bind_group, &[]);
            render_pass.draw_indexed(0..CROSSHAIR_INDICES.len() as u32, 0, 0..1);
        }

        self.render_queue
            .submit(std::iter::once(render_encoder.finish()));

        Ok(())
    }
}
