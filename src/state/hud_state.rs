use std::time::{Duration, Instant};

use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    CommandEncoder, SwapChainTexture,
};

use crate::{
    render_context::RenderContext,
    text_renderer::{self, TextRenderer},
    texture::Texture,
    vertex::HudVertex,
};

const UI_SCALE_X: f32 = 0.0045;
const UI_SCALE_Y: f32 = 0.008;

pub struct HudState {
    texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    hud_vertex_buffer: wgpu::Buffer,
    hud_index_buffer: wgpu::Buffer,

    text_renderer: TextRenderer,

    fps_vertex_buffer: wgpu::Buffer,
    fps_index_buffer: wgpu::Buffer,
    fps_index_count: usize,
    fps_instant: Instant,
    fps_frames: u32,
    fps_elapsed: Duration,

    coordinates_vertex_buffer: wgpu::Buffer,
    coordinates_index_buffer: wgpu::Buffer,
    coordinates_index_count: usize,
    coordinates_last: Vector3<f32>,
    pub hotbar_cursor_position: i32,
}

impl HudState {
    pub fn new(render_context: &RenderContext) -> Self {
        let (texture_bind_group_layout, texture_bind_group) = Self::create_textures(render_context);

        let render_pipeline =
            Self::create_render_pipeline(render_context, &[&texture_bind_group_layout]);

        // HUD buffers
        let hud_vertex_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("HUD crosshair vertex buffer"),
                contents: bytemuck::cast_slice(HUD_VERTICES),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });
        let hud_index_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("HUD crosshair index buffer"),
                contents: bytemuck::cast_slice(HUD_INDICES),
                usage: wgpu::BufferUsage::INDEX,
            });

        // Text buffers
        let text_renderer = TextRenderer::new(render_context).unwrap();
        let (fps_vertex_buffer, fps_index_buffer, fps_index_count) =
            text_renderer.string_to_buffers(&render_context, -0.98, 0.97, "");
        let (coordinates_vertex_buffer, coordinates_index_buffer, coordinates_index_count) =
            text_renderer.string_to_buffers(&render_context, -0.98, 0.97 - text_renderer::DY, "");

        Self {
            texture_bind_group,
            render_pipeline,
            hud_vertex_buffer,
            hud_index_buffer,
            text_renderer,

            fps_vertex_buffer,
            fps_index_buffer,
            fps_index_count,
            fps_instant: Instant::now(),
            fps_frames: 0,
            fps_elapsed: Duration::from_secs(0),

            coordinates_vertex_buffer,
            coordinates_index_buffer,
            coordinates_index_count,
            coordinates_last: Vector3::new(0.0, 0.0, 0.0),

            hotbar_cursor_position: 0,
        }
    }

    pub fn update(&mut self, render_context: &RenderContext, position: &Vector3<f32>) {
        let elapsed = self.fps_instant.elapsed();
        self.fps_instant = Instant::now();
        self.fps_elapsed += elapsed;
        self.fps_frames += 1;

        if self.fps_elapsed.as_millis() >= 500 {
            let frametime = self.fps_elapsed / self.fps_frames;
            let fps = 1.0 / frametime.as_secs_f32();

            let string = format!("{:<5.0} fps", fps);
            let (vertices, indices, index_count) =
                self.text_renderer
                    .string_to_buffers(render_context, -0.98, 0.97, &string);
            self.fps_vertex_buffer = vertices;
            self.fps_index_buffer = indices;
            self.fps_index_count = index_count;

            self.fps_elapsed = Duration::from_secs(0);
            self.fps_frames = 0;
        }

        if position != &self.coordinates_last {
            let string = format!("({:.1},{:.1},{:.1})", position.x, position.y, position.z,);
            let (vertices, indices, index_count) = self.text_renderer.string_to_buffers(
                render_context,
                -0.98,
                0.97 - text_renderer::DY * 1.3,
                &string,
            );
            self.coordinates_vertex_buffer = vertices;
            self.coordinates_index_buffer = indices;
            self.coordinates_index_count = index_count;
        }
    }

    pub fn render(
        &self,
        frame: &SwapChainTexture,
        render_encoder: &mut CommandEncoder,
    ) -> anyhow::Result<usize> {
        let mut render_pass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        // Render the crosshair and hotbar
        render_pass.set_vertex_buffer(0, self.hud_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.hud_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.draw_indexed(0..HUD_INDICES.len() as u32, 0, 0..1);

        // Render the FPS text
        render_pass.set_vertex_buffer(0, self.fps_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.fps_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        render_pass.draw_indexed(0..self.fps_index_count as u32, 0, 0..1);

        // Render the coordinates text
        render_pass.set_vertex_buffer(0, self.coordinates_vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.coordinates_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        render_pass.draw_indexed(0..self.coordinates_index_count as u32, 0, 0..1);

        Ok(HUD_INDICES.len() / 3)
    }

    pub fn update_hotbar_cursor(&self, render_context: &RenderContext) {
        let x = (-92 + 20 * self.hotbar_cursor_position) as f32;

        #[rustfmt::skip]
        let vertices = [
            HudVertex { position: [UI_SCALE_X * (x       ), -1.0 + UI_SCALE_Y * 23.0], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0] },
            HudVertex { position: [UI_SCALE_X * (x + 24.0), -1.0 + UI_SCALE_Y * 23.0], texture_coordinates: [ 24.0 / 256.0,  22.0 / 256.0] },
            HudVertex { position: [UI_SCALE_X * (x + 24.0), -1.0 + UI_SCALE_Y * -1.0], texture_coordinates: [ 24.0 / 256.0,  46.0 / 256.0] },
            HudVertex { position: [UI_SCALE_X * (x       ), -1.0 + UI_SCALE_Y * -1.0], texture_coordinates: [  0.0 / 256.0,  46.0 / 256.0] },
        ];

        render_context.queue.write_buffer(
            &self.hud_vertex_buffer,
            HudVertex::descriptor().array_stride * 8,
            bytemuck::cast_slice(&vertices),
        );
    }

    pub fn set_hotbar_cursor(&mut self, render_context: &RenderContext, i: i32) {
        self.hotbar_cursor_position = i;
        self.update_hotbar_cursor(render_context);
    }

    pub fn move_hotbar_cursor(&mut self, render_context: &RenderContext, delta: i32) {
        self.hotbar_cursor_position = (self.hotbar_cursor_position + delta).rem_euclid(9);
        self.update_hotbar_cursor(render_context);
    }

    fn create_textures(render_context: &RenderContext) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let texture = Texture::from_bytes(
            render_context,
            include_bytes!("../../assets/gui/widgets.png"),
            "Texture GUI widgets",
        )
        .unwrap();

        let sampler = render_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                ..wgpu::SamplerDescriptor::default()
            });

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
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GUI texture bind group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                ],
            });

        (bind_group_layout, bind_group)
    }

    fn create_render_pipeline(
        render_context: &RenderContext,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
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
                    bind_group_layouts,
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
                    polygon_mode: wgpu::PolygonMode::Fill,
                    clamp_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            })
    }
}

#[rustfmt::skip]
pub const HUD_VERTICES: &[HudVertex] = &[
    // Crosshair
    HudVertex { position: [UI_SCALE_X *  -8.0,        UI_SCALE_Y *  8.0], texture_coordinates: [240.0 / 256.0,   0.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X *   8.0,        UI_SCALE_Y *  8.0], texture_coordinates: [256.0 / 256.0,   0.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X *   8.0,        UI_SCALE_Y * -8.0], texture_coordinates: [256.0 / 256.0,  16.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X *  -8.0,        UI_SCALE_Y * -8.0], texture_coordinates: [240.0 / 256.0,  16.0 / 256.0] },

    // Hotbar
    HudVertex { position: [UI_SCALE_X * -91.0, -1.0 + UI_SCALE_Y * 22.0], texture_coordinates: [  0.0 / 256.0,   0.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X *  91.0, -1.0 + UI_SCALE_Y * 22.0], texture_coordinates: [182.0 / 256.0,   0.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X *  91.0, -1.0                    ], texture_coordinates: [182.0 / 256.0,  22.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X * -91.0, -1.0                    ], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0] },

    // Hotbar cursor
    HudVertex { position: [UI_SCALE_X * -92.0, -1.0 + UI_SCALE_Y * 23.0], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X * -68.0, -1.0 + UI_SCALE_Y * 23.0], texture_coordinates: [ 24.0 / 256.0,  22.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X * -68.0, -1.0 + UI_SCALE_Y * -1.0], texture_coordinates: [ 24.0 / 256.0,  46.0 / 256.0] },
    HudVertex { position: [UI_SCALE_X * -92.0, -1.0 + UI_SCALE_Y * -1.0], texture_coordinates: [  0.0 / 256.0,  46.0 / 256.0] },
];

#[rustfmt::skip]
pub const HUD_INDICES: &[u16] = &[
    // Crosshair
    1, 0, 3,
    1, 3, 2,

    // Hotbar
    5, 4, 7,
    5, 7, 6,

    // Hotbar cursor
    9, 8, 11,
    9, 11, 10,
];
