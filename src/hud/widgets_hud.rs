// TODO Might want to move the hotbar outside
use wgpu::{BindGroup, BufferUsage, RenderPass};

use crate::{
    geometry::Geometry,
    geometry_buffers::GeometryBuffers,
    render_context::RenderContext,
    texture::Texture,
    vertex::{HudVertex, Vertex},
    hud::{DEFAULT_UI_SCALE_X, DEFAULT_UI_SCALE_Y},
};

pub struct WidgetsHud {
    texture_bind_group: BindGroup,
    geometry_buffers: GeometryBuffers<u16>,
    pub hotbar_cursor_position: usize,

    ui_scale_x: f32,
    ui_scale_y: f32,
}

impl WidgetsHud {
    pub fn new(render_context: &RenderContext) -> Self {
        let (_, texture_bind_group) = Self::create_textures(render_context);
        let geometry_buffers = Self::generate_geometry_buffer(render_context, 0.0, 0.0);

        Self {
            texture_bind_group,
            geometry_buffers,
            hotbar_cursor_position: 0,
            ui_scale_x: DEFAULT_UI_SCALE_X,
            ui_scale_y: DEFAULT_UI_SCALE_Y,
        }
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
                                view_dimension: wgpu::TextureViewDimension::D2Array,
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

    pub fn set_hotbar_cursor(&mut self, render_context: &RenderContext, i: usize) {
        self.hotbar_cursor_position = i;
        self.redraw_hotbar_cursor(render_context);
    }

    pub fn move_hotbar_cursor(&mut self, render_context: &RenderContext, delta: i32) {
        self.hotbar_cursor_position =
            (self.hotbar_cursor_position as i32 + delta).rem_euclid(9) as usize;
        self.redraw_hotbar_cursor(render_context);
    }

    pub fn redraw_hotbar_cursor(&self, render_context: &RenderContext) {
        let x = (-92 + 20 * self.hotbar_cursor_position as i32) as f32;
        let texture_index = 0;
        let color = [1.0; 4];

        #[rustfmt::skip]
        let vertices = [
            HudVertex { position: [self.ui_scale_x * (x       ), -1.0 + self.ui_scale_y * 23.0], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0], texture_index, color },
            HudVertex { position: [self.ui_scale_x * (x + 24.0), -1.0 + self.ui_scale_y * 23.0], texture_coordinates: [ 24.0 / 256.0,  22.0 / 256.0], texture_index, color },
            HudVertex { position: [self.ui_scale_x * (x + 24.0), -1.0 + self.ui_scale_y * -1.0], texture_coordinates: [ 24.0 / 256.0,  46.0 / 256.0], texture_index, color },
            HudVertex { position: [self.ui_scale_x * (x       ), -1.0 + self.ui_scale_y * -1.0], texture_coordinates: [  0.0 / 256.0,  46.0 / 256.0], texture_index, color },
        ];

        render_context.queue.write_buffer(
            &self.geometry_buffers.vertices,
            HudVertex::descriptor().array_stride * 8,
            bytemuck::cast_slice(&vertices),
        );
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) -> usize {
        // Render the HUD elements
        self.geometry_buffers.apply_buffers(render_pass);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        self.geometry_buffers.draw_indexed(render_pass);
        render_pass.draw_indexed(0..self.geometry_buffers.index_count as u32, 0, 0..1);

        INDICES.len() / 3
    }

    pub fn set_scale(&mut self, render_context: &RenderContext, scale_x: f32, scale_y: f32) {
        self.ui_scale_x = scale_x;
        self.ui_scale_y = scale_y;
        self.geometry_buffers = Self::generate_geometry_buffer(render_context, scale_x, scale_y);
    }

    fn generate_geometry_buffer(render_context: &RenderContext, ui_scale_x: f32, ui_scale_y: f32) -> GeometryBuffers<u16> {
        #[rustfmt::skip]
        let vertices: [HudVertex; 12] = [
            // Crosshair
            HudVertex { position: [ui_scale_x *  -8.0,        ui_scale_y *  8.0], texture_coordinates: [240.0 / 256.0,   0.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x *   8.0,        ui_scale_y *  8.0], texture_coordinates: [  1.0,           0.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x *   8.0,        ui_scale_y * -8.0], texture_coordinates: [  1.0,          16.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x *  -8.0,        ui_scale_y * -8.0], texture_coordinates: [240.0 / 256.0,  16.0 / 256.0], texture_index: 0, color: [1.0; 4] },

            // Hotbar
            HudVertex { position: [ui_scale_x * -91.0, -1.0 + ui_scale_y * 22.0], texture_coordinates: [  0.0 / 256.0,   0.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x *  91.0, -1.0 + ui_scale_y * 22.0], texture_coordinates: [182.0 / 256.0,   0.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x *  91.0, -1.0                    ], texture_coordinates: [182.0 / 256.0,  22.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x * -91.0, -1.0                    ], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0], texture_index: 0, color: [1.0; 4] },

            // Hotbar cursor
            HudVertex { position: [ui_scale_x * -92.0, -1.0 + ui_scale_y * 23.0], texture_coordinates: [  0.0 / 256.0,  22.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x * -68.0, -1.0 + ui_scale_y * 23.0], texture_coordinates: [ 24.0 / 256.0,  22.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x * -68.0, -1.0 + ui_scale_y * -1.0], texture_coordinates: [ 24.0 / 256.0,  46.0 / 256.0], texture_index: 0, color: [1.0; 4] },
            HudVertex { position: [ui_scale_x * -92.0, -1.0 + ui_scale_y * -1.0], texture_coordinates: [  0.0 / 256.0,  46.0 / 256.0], texture_index: 0, color: [1.0; 4] },
        ];

        let geometry = Geometry {
            vertices: vertices.to_vec(),
            indices: INDICES.to_vec(),
        };

        GeometryBuffers::from_geometry(render_context, &geometry, BufferUsage::COPY_DST)
    }
}


#[rustfmt::skip]
pub const INDICES: [u16; 18] = [
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
