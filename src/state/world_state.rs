use std::time::Duration;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    CommandEncoder, SwapChainTexture,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, VirtualKeyCode},
};

use crate::{
    player::Player,
    render_context::RenderContext,
    renderable::Renderable,
    texture::Texture,
    time::Time,
    vertex::{BlockVertex, Vertex},
    world::{block::BlockType, World},
};

pub struct WorldState {
    pub render_pipeline: wgpu::RenderPipeline,
    pub depth_texture: Texture,

    time: Time,
    time_buffer: wgpu::Buffer,
    pub time_bind_group: wgpu::BindGroup,

    pub world: World,
    pub player: Player,
}

impl WorldState {
    fn create_time(
        render_context: &RenderContext,
    ) -> (Time, wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let time = Time::new();

        let buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("time_buffer"),
                contents: bytemuck::cast_slice(&[time]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let bind_group_layout =
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

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("time_bind_group"),
            });

        (time, buffer, bind_group_layout, bind_group)
    }

    fn create_render_pipeline(
        render_context: &RenderContext,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        render_context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
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
            })
    }

    pub fn new(render_context: &RenderContext) -> WorldState {
        let (time, time_buffer, time_layout, time_bind_group) = Self::create_time(render_context);
        let player = Player::new(render_context);

        let mut world = World::new();
        world.npc.load_geometry(render_context);

        let shader = render_context.device.create_shader_module(
            &(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/world.wgsl").into()),
            }),
        );

        let texture_manager = render_context.texture_manager.as_ref().unwrap();
        let render_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[
                        &texture_manager.bind_group_layout,
                        &player.view.bind_group_layout,
                        &time_layout,
                    ],
                });
        let render_pipeline =
            Self::create_render_pipeline(render_context, &shader, &render_pipeline_layout);
        let depth_texture = Texture::create_depth_texture(render_context, "depth_texture");

        Self {
            render_pipeline,
            depth_texture,

            time,
            time_buffer,
            time_bind_group,

            world,
            player,
        }
    }

    pub fn render(
        &self,
        render_context: &RenderContext,
        frame: &SwapChainTexture,
        render_encoder: &mut CommandEncoder,
    ) -> usize {
        let mut triangle_count = 0;

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
        render_pass.set_bind_group(1, &self.player.view.bind_group, &[]);
        render_pass.set_bind_group(2, &self.time_bind_group, &[]);

        triangle_count += self.world.render(&mut render_pass, &self.player.view);

        triangle_count
    }

    pub fn input_mouse_button(
        &mut self,
        button: &MouseButton,
        render_context: &RenderContext,
        selected: Option<BlockType>,
    ) {
        if button == &MouseButton::Left {
            self.world
                .break_at_crosshair(render_context, &self.player.view.camera);
        } else if button == &MouseButton::Right {
            if let Some(selected) = selected {
                self.world
                    .place_at_crosshair(render_context, &self.player.view.camera, selected);
            }
        }
    }

    #[allow(clippy::collapsible_else_if)]
    pub fn input_keyboard(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        match key_code {
            VirtualKeyCode::W => self.player.forward_pressed = pressed,
            VirtualKeyCode::S => self.player.backward_pressed = pressed,
            VirtualKeyCode::A => self.player.left_pressed = pressed,
            VirtualKeyCode::D => self.player.right_pressed = pressed,
            VirtualKeyCode::F2 if pressed => self.player.creative ^= true,
            VirtualKeyCode::Space => {
                // TODO aaaaaaaaaaaaaaaaaa
                self.player.up_speed = if pressed {
                    if self.player.creative {
                        1.0
                    } else {
                        if self.player.up_speed.abs() < 0.05 {
                            0.6
                        } else {
                            self.player.up_speed
                        }
                    }
                } else {
                    if self.player.creative {
                        0.0
                    } else {
                        self.player.up_speed
                    }
                }
            }
            VirtualKeyCode::LShift if self.player.creative => {
                self.player.up_speed = if pressed { -1.0 } else { 0.0 }
            }
            VirtualKeyCode::LControl => self.player.sprinting = pressed,
            _ => (),
        }
    }

    pub fn update(&mut self, dt: Duration, render_time: Duration, render_context: &RenderContext) {
        self.player.update_position(dt, &self.world);

        self.world
            .update(render_context, dt, render_time, &self.player.view.camera);

        self.player.view.update_view_projection(render_context);

        self.time.time += dt.as_secs_f32();
        render_context.queue.write_buffer(
            &self.time_buffer,
            0,
            &bytemuck::cast_slice(&[self.time]),
        );
    }

    pub fn resize(&mut self, render_context: &RenderContext, new_size: PhysicalSize<u32>) {
        // TODO Move this to View
        self.player
            .view
            .projection
            .resize(new_size.width, new_size.height);

        self.depth_texture = Texture::create_depth_texture(render_context, "depth_texture");
    }
}
