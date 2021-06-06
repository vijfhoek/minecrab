use std::time::Duration;

use wgpu::RenderPass;

use crate::{camera::Camera, render_context::RenderContext, view::View};

pub trait Renderable {
    fn update(
        &mut self,
        render_context: &RenderContext,
        dt: Duration,
        render_time: Duration,
        camera: &Camera,
    );
    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, view: &View) -> usize;
}
