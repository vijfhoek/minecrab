pub mod hud_state;
pub mod world_state;

use std::time::{Duration, Instant};

use cgmath::EuclideanSpace;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    window::Window,
};

use hud_state::HudState;
use world_state::WorldState;

use crate::{render_context::RenderContext, texture::TextureManager};

pub const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    topology: wgpu::PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: None,
    clamp_depth: false,
    polygon_mode: wgpu::PolygonMode::Fill,
    conservative: false,
};

pub struct State {
    pub window_size: PhysicalSize<u32>,
    render_context: RenderContext,
    pub world_state: WorldState,
    hud_state: HudState,

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
        println!("Using {:?}", adapter.get_info().backend);

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

        let world_state = WorldState::new(&render_context);
        let hud_state = HudState::new(&render_context);

        Self {
            window_size,
            render_context,

            world_state,
            hud_state,

            mouse_grabbed: false,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        println!("resizing to {:?}", new_size);
        self.window_size = new_size;
        self.render_context.swap_chain_descriptor.width = new_size.width;
        self.render_context.swap_chain_descriptor.height = new_size.height;

        self.world_state.resize(&self.render_context, new_size);

        self.render_context.swap_chain = self.render_context.device.create_swap_chain(
            &self.render_context.surface,
            &self.render_context.swap_chain_descriptor,
        );
    }

    fn input_keyboard(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        if state == ElementState::Pressed {
            match key_code {
                VirtualKeyCode::F1 => self.world_state.toggle_wireframe(&self.render_context),
                VirtualKeyCode::Key1 => self.hud_state.set_hotbar_cursor(&self.render_context, 0),
                VirtualKeyCode::Key2 => self.hud_state.set_hotbar_cursor(&self.render_context, 1),
                VirtualKeyCode::Key3 => self.hud_state.set_hotbar_cursor(&self.render_context, 2),
                VirtualKeyCode::Key4 => self.hud_state.set_hotbar_cursor(&self.render_context, 3),
                VirtualKeyCode::Key5 => self.hud_state.set_hotbar_cursor(&self.render_context, 4),
                VirtualKeyCode::Key6 => self.hud_state.set_hotbar_cursor(&self.render_context, 5),
                VirtualKeyCode::Key7 => self.hud_state.set_hotbar_cursor(&self.render_context, 6),
                VirtualKeyCode::Key8 => self.hud_state.set_hotbar_cursor(&self.render_context, 7),
                VirtualKeyCode::Key9 => self.hud_state.set_hotbar_cursor(&self.render_context, 8),
                _ => self.world_state.input_keyboard(key_code, state),
            }
        } else {
            self.world_state.input_keyboard(key_code, state)
        }
    }

    fn input_mouse(&mut self, dx: f64, dy: f64) {
        if self.mouse_grabbed {
            self.world_state.update_camera(dx, dy);
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
            } if self.mouse_grabbed => self.world_state.input_mouse_button(
                button,
                &self.render_context,
                self.hud_state.selected_block_type(),
            ),

            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, delta),
                ..
            } => self
                .hud_state
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
        self.world_state
            .update(dt, render_time, &self.render_context);
        self.hud_state.update(
            &self.render_context,
            &self.world_state.camera.position.to_vec(),
        );
    }

    pub fn render(&mut self) -> anyhow::Result<(usize, Duration)> {
        let render_start = Instant::now();

        Ok({
            let frame = self.render_context.swap_chain.get_current_frame()?.output;

            let mut render_encoder = self
                .render_context
                .device
                .create_command_encoder(&Default::default());

            let mut triangle_count = 0;
            triangle_count +=
                self.world_state
                    .render(&self.render_context, &frame, &mut render_encoder);
            triangle_count +=
                self.hud_state
                    .render(&self.render_context, &frame, &mut render_encoder)?;

            self.render_context
                .queue
                .submit(std::iter::once(render_encoder.finish()));
            let render_time = render_start.elapsed();

            (triangle_count, render_time)
        })
    }
}
