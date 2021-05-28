mod camera;
mod cube;
mod instance;
mod light;
mod state;
mod texture;
mod uniforms;
mod vertex;

use std::time::Instant;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::state::State;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(r#"minecrab"#)
        .with_inner_size(Size::Physical(PhysicalSize {
            width: 1280,
            height: 720,
        }))
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));

    let mut frames = 0;
    let mut instant = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                frames += 1;
                if frames % 1000 == 0 {
                    let frametime = instant.elapsed() / 1000;
                    let fps = 1_000_000 / frametime.as_micros();
                    println!("{:?} - {} fps", frametime, fps);
                    instant = Instant::now();
                }

                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
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
