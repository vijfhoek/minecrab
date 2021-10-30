use wgpu::{CommandEncoder, RenderPipeline, SwapChainTexture};

use crate::{
    render_context::RenderContext,
    vertex::{HudVertex, Vertex},
    world::block::BlockType,
};

use self::{debug_hud::DebugHud, hotbar_hud::HotbarHud, widgets_hud::WidgetsHud};
use winit::dpi::PhysicalSize;

pub mod debug_hud;
pub mod hotbar_hud;
pub mod widgets_hud;

// TODO update aspect ratio when resizing
pub const UI_SCALE_X: f32 = 0.0045;
pub const UI_SCALE_Y: f32 = 0.008;

pub struct Hud {
    pub widgets_hud: WidgetsHud,
    pub debug_hud: DebugHud,
    pub hotbar_hud: HotbarHud,

    pub pipeline: RenderPipeline,
}

impl Hud {
    pub fn new(render_context: &RenderContext) -> Self {
        Self {
            widgets_hud: WidgetsHud::new(render_context),
            debug_hud: DebugHud::new(render_context),
            hotbar_hud: HotbarHud::new(render_context),

            pipeline: Self::create_render_pipeline(render_context),
        }
    }

    fn create_render_pipeline(render_context: &RenderContext) -> wgpu::RenderPipeline {
        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("GUI texture bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let module = &render_context
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("UI shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui.wgsl").into()),
            });

        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("UI render pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        render_context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("UI render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "main",
                    buffers: &[HudVertex::descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: render_context.swap_chain_descriptor.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    clamp_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: Default::default(),
            })
    }

    pub fn update(
        &mut self,
        render_context: &crate::render_context::RenderContext,
        camera: &crate::camera::Camera,
    ) {
        self.debug_hud.update(render_context, &camera.position);
        self.hotbar_hud.update(render_context);
    }

    pub fn render<'a>(
        &'a self,
        render_context: &RenderContext,
        encoder: &mut CommandEncoder,
        frame: &SwapChainTexture,
    ) -> usize {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            ..Default::default()
        });
        render_pass.set_pipeline(&self.pipeline);

        self.widgets_hud.render(&mut render_pass)
            + self.debug_hud.render(&mut render_pass)
            + self.hotbar_hud.render(render_context, &mut render_pass)
    }

    pub fn selected_block(&self) -> Option<BlockType> {
        // TODO The hotbar widget should be rendered by HotbarHud
        self.hotbar_hud.blocks[self.widgets_hud.hotbar_cursor_position]
    }

    pub fn resize(&mut self, render_context: &RenderContext, size: PhysicalSize<u32>) {
        let ratio = size.width as f32 / size.height as f32;

        const MAX_SCALE_X_BEFORE_HOTBAR_CLIPS: f32 = 0.0108;

        let mut ui_scale_y = 0.008;
        let mut ui_scale_x = ui_scale_y / ratio;

        if ui_scale_x > MAX_SCALE_X_BEFORE_HOTBAR_CLIPS {
            ui_scale_x = MAX_SCALE_X_BEFORE_HOTBAR_CLIPS;
            ui_scale_y = ui_scale_x * ratio;
        }

        self.debug_hud.set_ratio(ratio);
        self.hotbar_hud.set_scale(ui_scale_x, ui_scale_y);
        self.widgets_hud.set_scale(render_context, ui_scale_x, ui_scale_y);
    }

}
