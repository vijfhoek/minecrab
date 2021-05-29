mod aabb;
mod camera;
mod chunk;
mod cube;
mod instance;
mod light;
mod state;
mod texture;
mod uniforms;
mod vertex;
mod world_state;

use std::time::Instant;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::state::State;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("minecrab")
        .with_inner_size(Size::Physical(PhysicalSize {
            width: 1280,
            height: 720,
        }))
        .build(&event_loop)
        .unwrap();

    let mut state = futures::executor::block_on(State::new(&window));

    let mut frames = 0;
    let mut instant = Instant::now();

    let mut last_render_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent { ref event, .. } => state.input(event),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    let _ = window.set_cursor_grab(false);
                    window.set_cursor_visible(true);
                    state.mouse_grabbed = false;
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => {
                    if *button == MouseButton::Left && *mouse_state == ElementState::Pressed {
                        let _ = window.set_cursor_grab(true);
                        window.set_cursor_visible(false);
                        state.mouse_grabbed = true;
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                frames += 1;
                if frames % 1000 == 0 {
                    let frametime = instant.elapsed() / 1000;
                    let fps = 1_000_000 / frametime.as_micros();
                    println!("{:?} - {} fps", frametime, fps);
                    instant = Instant::now();
                }

                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.window_size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
