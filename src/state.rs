use std::time::{Duration, Instant};

use winit::{
    dpi::PhysicalSize,
    event::{
        DeviceEvent, ElementState, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    window::Window,
};

use crate::{
    hud::Hud,
    player::Player,
    render_context::RenderContext,
    texture::{Texture, TextureManager},
    world::World,
};

pub struct State {
    pub window_size: PhysicalSize<u32>,
    pub mouse_grabbed: bool,
    render_context: RenderContext,

    pub world: World,
    player: Player,
    hud: Hud,
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
        println!("Using {:?}", adapter.get_info().backend);

        let (render_device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("render_device"),
                    features: wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY,
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
        let (render_surface, render_adapter, render_device, render_queue) =
            Self::create_render_device(window).await;

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(window, &render_adapter, &render_device, &render_surface);

        let mut render_context = RenderContext {
            surface: render_surface,
            device: render_device,
            queue: render_queue,

            swap_chain_descriptor,
            swap_chain,
            texture_manager: None,
        };

        let mut texture_manager = TextureManager::new(&render_context);
        texture_manager.load_all(&render_context).unwrap();
        render_context.texture_manager = Some(texture_manager);

        let hud = Hud::new(&render_context);
        let player = Player::new(&render_context);
        let world = World::new(&render_context, &player.view);

        Self {
            window_size: window.inner_size(),
            mouse_grabbed: false,
            render_context,

            world,
            player,
            hud,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        println!("resizing to {:?}", size);
        self.window_size = size;
        self.render_context.swap_chain_descriptor.width = size.width;
        self.render_context.swap_chain_descriptor.height = size.height;

        self.player.view.projection.resize(size.width, size.height);
        self.world.depth_texture =
            Texture::create_depth_texture(&self.render_context, "depth_texture");

        self.render_context.swap_chain = self.render_context.device.create_swap_chain(
            &self.render_context.surface,
            &self.render_context.swap_chain_descriptor,
        );
    }

    fn set_hotbar_cursor(&mut self, i: usize) {
        self.hud
            .widgets_hud
            .set_hotbar_cursor(&self.render_context, i);
    }

    fn input_keyboard(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        let pressed = state == ElementState::Pressed;

        match key_code {
            VirtualKeyCode::F2 if pressed => self.player.creative ^= true,

            // Hotbar
            VirtualKeyCode::Key1 if pressed => self.set_hotbar_cursor(0),
            VirtualKeyCode::Key2 if pressed => self.set_hotbar_cursor(1),
            VirtualKeyCode::Key3 if pressed => self.set_hotbar_cursor(2),
            VirtualKeyCode::Key4 if pressed => self.set_hotbar_cursor(3),
            VirtualKeyCode::Key5 if pressed => self.set_hotbar_cursor(4),
            VirtualKeyCode::Key6 if pressed => self.set_hotbar_cursor(5),
            VirtualKeyCode::Key7 if pressed => self.set_hotbar_cursor(6),
            VirtualKeyCode::Key8 if pressed => self.set_hotbar_cursor(7),
            VirtualKeyCode::Key9 if pressed => self.set_hotbar_cursor(8),

            // Movement
            VirtualKeyCode::W => self.player.forward_pressed = pressed,
            VirtualKeyCode::S => self.player.backward_pressed = pressed,
            VirtualKeyCode::A => self.player.left_pressed = pressed,
            VirtualKeyCode::D => self.player.right_pressed = pressed,
            VirtualKeyCode::Space => {
                self.player.up_speed = match (pressed, self.player.creative) {
                    // Creative
                    (true, true) => 1.0,
                    (false, true) => 0.0,

                    // Not creative
                    (true, false) if self.player.grounded => 0.6,
                    _ => self.player.up_speed,
                };
            }
            VirtualKeyCode::LShift if self.player.creative => {
                self.player.up_speed = if pressed { -1.0 } else { 0.0 }
            }
            VirtualKeyCode::LControl => self.player.sprinting = pressed,

            _ => (),
        }
    }

    fn input_mouse(&mut self, dx: f64, dy: f64) {
        if self.mouse_grabbed {
            self.player.update_camera(dx, dy);
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } if input.virtual_keycode.is_some() => {
                self.input_keyboard(input.virtual_keycode.unwrap(), input.state)
            }

            WindowEvent::MouseInput {
                button,
                state: ElementState::Pressed,
                ..
            } if self.mouse_grabbed => {
                if button == &MouseButton::Left {
                    self.world
                        .break_at_crosshair(&self.render_context, &self.player.view.camera);
                } else if button == &MouseButton::Right {
                    if let Some(selected) = self.hud.selected_block() {
                        self.world.place_at_crosshair(
                            &self.render_context,
                            &self.player.view.camera,
                            selected,
                        );
                    }
                }
            }

            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, delta),
                ..
            } => self
                .hud
                .widgets_hud
                .move_hotbar_cursor(&self.render_context, -*delta as i32),

            _ => (),
        }
    }

    pub fn device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input_mouse(delta.0, delta.1)
        }
    }

    pub fn update(&mut self, dt: Duration, render_time: Duration) {
        self.player.update_position(dt, &self.world);

        let view = &mut self.player.view;
        view.update_view_projection(&self.render_context);

        self.world
            .update(&self.render_context, dt, render_time, &view.camera);
        self.hud.update(&self.render_context, &view.camera);
    }

    pub fn render(&mut self) -> anyhow::Result<(usize, Duration)> {
        let render_start = Instant::now();

        let frame = self.render_context.swap_chain.get_current_frame()?.output;

        let mut render_encoder = self
            .render_context
            .device
            .create_command_encoder(&Default::default());

        let mut triangle_count = 0;

        triangle_count += self.world.render(
            &self.render_context,
            &mut render_encoder,
            &frame,
            &self.player.view,
        );

        triangle_count += self
            .hud
            .render(&self.render_context, &mut render_encoder, &frame);

        self.render_context
            .queue
            .submit(std::iter::once(render_encoder.finish()));
        let render_time = render_start.elapsed();

        Ok((triangle_count, render_time))
    }
}
