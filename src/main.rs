mod aabb;
mod camera;
mod chunk;
mod cube;
mod quad;
mod render_context;
mod state;
mod text_renderer;
mod texture;
mod time;
mod vertex;
mod view;
mod world;

use std::time::{Duration, Instant};
use wgpu::SwapChainError;
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
    let mut frame_instant = Instant::now();
    let mut elapsed = Duration::from_secs(0);

    let mut frametime_min = Duration::from_secs(1000);
    let mut frametime_max = Duration::from_secs(0);

    let mut last_render_time = Instant::now();
    let mut triangle_count = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent { ref event, .. } => state.device_event(event),
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

                    state.window_event(event);
                }
                event => { state.window_event(event); }
            },
            Event::RedrawRequested(_) => {
                let frame_elapsed = frame_instant.elapsed();
                frame_instant = Instant::now();

                frametime_min = frametime_min.min(frame_elapsed);
                frametime_max = frametime_max.max(frame_elapsed);
                elapsed += frame_elapsed;

                frames += 1;
                if elapsed.as_secs() >= 1 {
                    let frametime = elapsed / frames;
                    let fps = 1_000_000 / frametime.as_micros();
                    let fps_max = 1_000_000 / frametime_min.as_micros();
                    let fps_min = 1_000_000 / frametime_max.as_micros();

                    println!(
                        "{:>4} frames | frametime avg={:>5.2}ms min={:>5.2}ms max={:>5.2}ms | fps avg={:>5} min={:>5} max={:>5} | {:>8} tris",
                        frames, frametime.as_secs_f32() * 1000.0, frametime_min.as_secs_f32() * 1000.0, frametime_max.as_secs_f32() * 1000.0, fps, fps_min, fps_max, triangle_count,
                    );

                    elapsed = Duration::from_secs(0);
                    frames = 0;
                    frametime_min = Duration::from_secs(1000);
                    frametime_max = Duration::from_secs(0);
                }

                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                match state.render() {
                    Err(root_cause) =>
                        match root_cause.downcast_ref::<SwapChainError>() {
                            // Recreate the swap_chain if lost
                            Some(wgpu::SwapChainError::Lost) => state.resize(state.window_size),
                            // The system is out of memory, we should probably quit
                            Some(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Some(_) | None => eprintln!("{:?}", root_cause),
                        }

                    Ok(v) => triangle_count = v
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
