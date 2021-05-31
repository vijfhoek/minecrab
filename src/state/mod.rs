mod hud;
mod world;

use std::time::Duration;

use cgmath::{InnerSpace, Rad};
use winit::{
    event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode},
    window::Window,
};

use hud::HudState;
use world::WorldState;

pub struct State {
    pub window_size: winit::dpi::PhysicalSize<u32>,
    render_surface: wgpu::Surface,
    render_device: wgpu::Device,
    render_queue: wgpu::Queue,

    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    world_state: WorldState,
    hud_state: HudState,

    right_speed: f32,
    forward_speed: f32,
    up_speed: f32,
    pub mouse_grabbed: bool,
}

impl State {
    async fn create_render_device(
        window: &Window,
    ) -> (wgpu::Surface, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
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
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = render_device.create_swap_chain(&render_surface, &swap_chain_descriptor);

        (swap_chain_descriptor, swap_chain)
    }

    pub async fn new(window: &Window) -> State {
        let window_size = window.inner_size();

        let (render_surface, render_adapter, render_device, render_queue) =
            Self::create_render_device(window).await;

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(window, &render_adapter, &render_device, &render_surface);

        let world_state = WorldState::new(&render_device, &render_queue, &swap_chain_descriptor);
        let hud_state = HudState::new(&render_device, &render_queue, &swap_chain_descriptor);

        Self {
            window_size,
            render_surface,
            render_device,
            render_queue,

            swap_chain_descriptor,
            swap_chain,

            world_state,
            hud_state,

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
        // let camera = &self.world_state.camera;
        // let chunk = &mut self.world_state.chunk;
        // let position = chunk
        //     .raycast(camera.position.to_vec(), camera.direction())
        //     .map(|(position, _)| position);
        // if position != chunk.highlighted {
        //     chunk.highlighted = position;
        //     self.world_state.update_chunk(&self.render_queue);
        // }
    }

    fn input_mouse(&mut self, dx: f64, dy: f64) {
        if self.mouse_grabbed {
            self.update_camera(dx, dy);
        }
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.input_keyboard(key, state),

            // DeviceEvent::Button {
            //     button,
            //     state: ElementState::Pressed,
            // } if self.mouse_grabbed => {
            //     let camera = &self.world_state.camera;

            //     // if let Some((pos, axis)) = self
            //     //     .world_state
            //     //     .chunk
            //     //     .raycast(camera.position.to_vec(), camera.direction())
            //     // {
            //     //     if *button == 1 {
            //     //         self.world_state.chunk.blocks[pos.y][pos.z][pos.x].take();
            //     //         dbg!(&pos);
            //     //         self.world_state.update_chunk(&self.render_queue);
            //     //     } else if *button == 3 {
            //     //         let new_pos = pos.map(|x| x as i32) - axis;
            //     //         dbg!(&axis, &new_pos);
            //     //         self.world_state.chunk.blocks[new_pos.y as usize][new_pos.z as usize]
            //     //             [new_pos.x as usize] = Some(Block {
            //     //             block_type: BlockType::Cobblestone,
            //     //         });
            //     //         self.world_state.update_chunk(&self.render_queue);
            //     //     }
            //     // }
            // }
            DeviceEvent::MouseMotion { delta: (dx, dy) } => self.input_mouse(*dx, *dy),
            _ => (),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = self.world_state.camera.yaw.0.sin_cos();

        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        self.world_state.camera.position += forward * self.forward_speed * 15.0 * dt_secs;

        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.world_state.camera.position += right * self.right_speed * 15.0 * dt_secs;

        let up = cgmath::Vector3::new(0.0, 1.0, 0.0).normalize();
        self.world_state.camera.position += up * self.up_speed * 15.0 * dt_secs;

        self.update_aim();

        self.world_state
            .uniforms
            .update_view_projection(&self.world_state.camera, &self.world_state.projection);
        self.render_queue.write_buffer(
            &self.world_state.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.world_state.uniforms]),
        );
    }

    pub fn render(&mut self) -> anyhow::Result<usize> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut render_encoder = self
            .render_device
            .create_command_encoder(&Default::default());

        let mut triangle_count = 0;

        {
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
                    view: &self.world_state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.world_state.render_pipeline);

            let tm = &self.world_state.texture_manager;
            render_pass.set_bind_group(0, tm.bind_group.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(1, &self.world_state.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.world_state.light_bind_group, &[]);

            for (chunk_vertices, chunk_indices, index_count) in &self.world_state.chunk_buffers {
                render_pass.set_vertex_buffer(0, chunk_vertices.slice(..));
                render_pass.set_index_buffer(chunk_indices.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*index_count as u32, 0, 0..1);
                triangle_count += index_count / 3;
            }
        }

        triangle_count += self.hud_state.render(&frame, &mut render_encoder)?;

        self.render_queue
            .submit(std::iter::once(render_encoder.finish()));

        Ok(triangle_count)
    }
}
