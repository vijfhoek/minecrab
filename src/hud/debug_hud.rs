use std::time::{Duration, Instant};

use cgmath::Point3;
use wgpu::RenderPass;

use crate::{
    geometry::GeometryBuffers,
    render_context::RenderContext,
    text_renderer::{self, TextRenderer},
};

pub struct DebugHud {
    text_renderer: TextRenderer,

    fps_instant: Instant,
    fps_elapsed: Duration,
    fps_frames: u32,
    fps_geometry_buffers: GeometryBuffers<u16>,

    coordinates_last: Point3<f32>,
    coordinates_geometry_buffers: GeometryBuffers<u16>,
}

impl DebugHud {
    pub fn new(render_context: &RenderContext) -> Self {
        let text_renderer = TextRenderer::new(render_context).unwrap();
        let fps_geometry_buffers =
            text_renderer.string_to_buffers(&render_context, -0.98, 0.97, "");
        let coordinates_geometry_buffers =
            text_renderer.string_to_buffers(&render_context, -0.98, 0.97 - text_renderer::DY, "");

        Self {
            text_renderer,

            fps_instant: Instant::now(),
            fps_elapsed: Duration::default(),
            fps_frames: 0,
            fps_geometry_buffers,

            coordinates_last: Point3::new(0.0, 0.0, 0.0),
            coordinates_geometry_buffers,
        }
    }

    pub fn update(&mut self, render_context: &RenderContext, position: &Point3<f32>) {
        let elapsed = self.fps_instant.elapsed();
        self.fps_instant = Instant::now();
        self.fps_elapsed += elapsed;
        self.fps_frames += 1;

        if self.fps_elapsed.as_millis() >= 500 {
            let frametime = self.fps_elapsed / self.fps_frames;
            let fps = 1.0 / frametime.as_secs_f32();

            let string = format!("{:<5.0} fps", fps);
            self.fps_geometry_buffers =
                self.text_renderer
                    .string_to_buffers(render_context, -0.98, 0.97, &string);

            self.fps_elapsed = Duration::from_secs(0);
            self.fps_frames = 0;
        }

        if position != &self.coordinates_last {
            let string = format!("({:.1},{:.1},{:.1})", position.x, position.y, position.z,);
            self.coordinates_geometry_buffers = self.text_renderer.string_to_buffers(
                render_context,
                -0.98,
                0.97 - text_renderer::DY * 1.3,
                &string,
            );
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) -> usize {
        let mut triangle_count = 0;

        // Render the FPS text
        self.fps_geometry_buffers.apply_buffers(render_pass);
        render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        triangle_count += self.fps_geometry_buffers.draw_indexed(render_pass);

        // Render the coordinates text
        self.coordinates_geometry_buffers.apply_buffers(render_pass);
        render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        triangle_count += self.coordinates_geometry_buffers.draw_indexed(render_pass);

        triangle_count
    }
}
