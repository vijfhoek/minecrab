use std::time::Duration;

use cgmath::{InnerSpace, Point3, Rad, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    CommandEncoder, SwapChainTexture,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, VirtualKeyCode},
};

use crate::{
    camera::{Camera, Projection},
    render_context::RenderContext,
    renderable::Renderable,
    texture::{Texture, TextureManager},
    time::Time,
    vertex::{BlockVertex, Vertex},
    view::View,
    world::world::World,
};

pub struct WorldState {
    pub render_pipeline: wgpu::RenderPipeline,
    pub view: View,
    pub view_buffer: wgpu::Buffer,
    pub view_bind_group: wgpu::BindGroup,
    pub texture_manager: TextureManager,
    pub camera: Camera,
    pub projection: Projection,
    pub depth_texture: Texture,
    pub time_bind_group: wgpu::BindGroup,
    pub world: World,

    time: Time,
    time_buffer: wgpu::Buffer,
    wireframe: bool,
    shader: wgpu::ShaderModule,
    render_pipeline_layout: wgpu::PipelineLayout,

    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,

    pub up_speed: f32,
    pub sprinting: bool,
    pub creative: bool,
}

impl WorldState {
    fn create_textures(render_context: &RenderContext) -> TextureManager {
        let mut texture_manager = TextureManager::new(&render_context);
        texture_manager.load_all(render_context).unwrap();
        texture_manager
    }

    fn create_camera(render_context: &RenderContext) -> (Camera, Projection) {
        let camera = Camera::new(
            (10.0, 140.0, 10.0).into(),
            cgmath::Deg(45.0).into(),
            cgmath::Deg(-20.0).into(),
        );

        let projection = Projection::new(
            render_context.swap_chain_descriptor.width,
            render_context.swap_chain_descriptor.height,
            cgmath::Deg(45.0),
            0.1,
            300.0,
        );

        (camera, projection)
    }

    fn create_view(
        camera: &Camera,
        projection: &Projection,
        render_context: &RenderContext,
    ) -> (View, wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mut view = View::new();
        view.update_view_projection(camera, projection);

        let view_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("view_buffer"),
                contents: bytemuck::cast_slice(&[view.to_raw()]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let view_bind_group_layout =
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
                    label: Some("view_bind_group_layout"),
                });

        let view_bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &view_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                }],
                label: Some("view_bind_group"),
            });

        (view, view_buffer, view_bind_group_layout, view_bind_group)
    }

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
        wireframe: bool,
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

    pub fn toggle_wireframe(&mut self, render_context: &RenderContext) {
        self.wireframe = !self.wireframe;
        self.render_pipeline = Self::create_render_pipeline(
            render_context,
            &self.shader,
            &self.render_pipeline_layout,
            self.wireframe,
        )
    }

    pub fn new(render_context: &RenderContext) -> WorldState {
        let texture_manager = Self::create_textures(render_context);

        let (camera, projection) = Self::create_camera(render_context);

        let (view, view_buffer, view_bind_group_layout, view_bind_group) =
            Self::create_view(&camera, &projection, render_context);

        let (time, time_buffer, time_layout, time_bind_group) = Self::create_time(render_context);

        let mut world = World::new();
        world.npc.load_geometry(render_context);

        let shader = render_context.device.create_shader_module(
            &(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/world.wgsl").into()),
            }),
        );

        let render_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[
                        &texture_manager.bind_group_layout,
                        &view_bind_group_layout,
                        &time_layout,
                    ],
                });
        let render_pipeline =
            Self::create_render_pipeline(&render_context, &shader, &render_pipeline_layout, false);
        let depth_texture = Texture::create_depth_texture(render_context, "depth_texture");

        Self {
            render_pipeline,
            view,
            view_buffer,
            view_bind_group,
            texture_manager,
            camera,
            projection,
            depth_texture,
            shader,
            render_pipeline_layout,

            time,
            time_buffer,
            time_bind_group,

            world,

            wireframe: false,

            up_speed: 0.0,
            sprinting: false,
            forward_pressed: false,
            backward_pressed: false,
            left_pressed: false,
            right_pressed: false,
            creative: true,
        }
    }

    pub fn render(&self, frame: &SwapChainTexture, render_encoder: &mut CommandEncoder) -> usize {
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

        let tm = &self.texture_manager;
        render_pass.set_bind_group(0, tm.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_bind_group(1, &self.view_bind_group, &[]);
        render_pass.set_bind_group(2, &self.time_bind_group, &[]);

        triangle_count += self.world.render(&mut render_pass, &self.view);

        triangle_count
    }

    pub fn update_camera(&mut self, dx: f64, dy: f64) {
        let camera = &mut self.camera;
        camera.yaw += Rad(dx as f32 * 0.003);
        camera.pitch -= Rad(dy as f32 * 0.003);

        if camera.pitch < Rad::from(cgmath::Deg(-80.0)) {
            camera.pitch = Rad::from(cgmath::Deg(-80.0));
        } else if camera.pitch > Rad::from(cgmath::Deg(89.9)) {
            camera.pitch = Rad::from(cgmath::Deg(89.9));
        }
    }

    pub fn input_mouse_button(&mut self, button: &MouseButton, render_context: &RenderContext) {
        if button == &MouseButton::Left {
            self.world.break_at_crosshair(render_context, &self.camera);
        } else if button == &MouseButton::Right {
            self.world.place_at_crosshair(render_context, &self.camera);
        }
    }

    pub fn input_keyboard(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        match key_code {
            VirtualKeyCode::W => self.forward_pressed = pressed,
            VirtualKeyCode::S => self.backward_pressed = pressed,
            VirtualKeyCode::A => self.left_pressed = pressed,
            VirtualKeyCode::D => self.right_pressed = pressed,
            VirtualKeyCode::F2 if pressed => self.creative = !self.creative,
            VirtualKeyCode::Space => {
                self.up_speed = if pressed {
                    if self.creative {
                        1.0
                    } else {
                        0.6
                    }
                } else {
                    0.0
                }
            }
            VirtualKeyCode::LShift if self.creative => {
                self.up_speed = if pressed { -1.0 } else { 0.0 }
            }
            VirtualKeyCode::LControl => self.sprinting = pressed,
            _ => (),
        }
    }

    fn check_collision(&self, position: Point3<f32>) -> bool {
        self.world
            .get_block(
                position.x as isize,
                (position.y - 1.8) as isize,
                position.z as isize,
            )
            .is_some()
    }

    fn update_position(&mut self, dt: Duration) {
        let dt_seconds = dt.as_secs_f32();
        let (yaw_sin, yaw_cos) = self.camera.yaw.0.sin_cos();

        let speed = 10.0 * (self.sprinting as i32 * 2 + 1) as f32;

        let mut new_position = self.camera.position;

        let up = Vector3::unit_y() * self.up_speed * speed * dt_seconds;
        new_position += up;
        if !self.creative && self.check_collision(new_position) {
            new_position -= up;
            self.up_speed = 0.0;
        }

        let forward_speed = self.forward_pressed as i32 - self.backward_pressed as i32;
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let forward = forward * forward_speed as f32 * speed * dt_seconds;
        new_position += forward;
        if !self.creative && self.check_collision(new_position) {
            new_position -= forward;
        }

        let right_speed = self.right_pressed as i32 - self.left_pressed as i32;
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let right = right * right_speed as f32 * speed * dt_seconds;
        new_position += right;
        if !self.creative && self.check_collision(new_position) {
            new_position -= right;
        }

        self.camera.position = new_position;

        if !self.creative {
            self.up_speed -= 1.6 * dt.as_secs_f32();
            self.up_speed *= 0.98_f32.powf(dt.as_secs_f32() / 20.0);
        }
    }

    pub fn update(&mut self, dt: Duration, render_context: &RenderContext) {
        self.update_position(dt);

        self.world.update(render_context, dt, &self.camera);

        self.view
            .update_view_projection(&self.camera, &self.projection);
        render_context.queue.write_buffer(
            &self.view_buffer,
            0,
            bytemuck::cast_slice(&[self.view.to_raw()]),
        );

        self.time.time += dt.as_secs_f32();
        render_context.queue.write_buffer(
            &self.time_buffer,
            0,
            &bytemuck::cast_slice(&[self.time]),
        );
    }

    pub fn resize(&mut self, render_context: &RenderContext, new_size: PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = Texture::create_depth_texture(render_context, "depth_texture");
    }
}
