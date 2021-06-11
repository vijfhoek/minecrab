use std::convert::TryInto;

use crate::{
    geometry::Geometry, geometry_buffers::GeometryBuffers, render_context::RenderContext,
    texture::Texture, vertex::HudVertex,
};

pub const DX: f32 = 20.0 / 640.0;
pub const DY: f32 = 20.0 / 360.0;

#[rustfmt::skip]
const CHARACTER_WIDTHS: [i32; 16 * 8] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    4, 2, 4, 6, 6, 6, 6, 2, 4, 4, 4, 6, 2, 6, 2, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 2, 2, 5, 6, 5, 6,
    7, 6, 6, 6, 6, 6, 6, 6, 6, 4, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 4, 6, 4, 6, 6,
    3, 6, 6, 6, 6, 6, 6, 6, 6, 2, 6, 5, 3, 6, 6, 6,
    6, 6, 6, 6, 4, 6, 6, 6, 6, 6, 6, 4, 2, 4, 7, 0,
];

pub struct TextRenderer {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl TextRenderer {
    pub fn new(render_context: &RenderContext) -> anyhow::Result<Self> {
        let bytes = std::fs::read("assets/font/ascii_shadow.png")?;
        let texture = Texture::from_bytes(render_context, &bytes, "font")?;

        let sampler = render_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..wgpu::SamplerDescriptor::default()
            });

        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Font texture bind group layout"),
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
                label: Some(&("Font texture bind group")),
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

        Ok(Self {
            texture,
            bind_group,
        })
    }

    fn char_uv(c: u8) -> (f32, f32) {
        let row = (c / 16) as f32;
        let column = (c % 16) as f32;
        (column / 16.0, row / 16.0)
    }

    pub fn char_geometry(
        &self,
        x: f32,
        y: f32,
        c: u8,
        index_offset: u16,
    ) -> ([HudVertex; 4], [u16; 6]) {
        let (tx, ty) = Self::char_uv(c);
        let s = 1.0 / 16.0;

        #[rustfmt::skip]
        let vertices = [
            HudVertex { position: [x,      y     ], texture_coordinates: [tx,     ty    ], texture_index: 0, value: 1.0 },
            HudVertex { position: [x + DX, y     ], texture_coordinates: [tx + s, ty    ], texture_index: 0, value: 1.0 },
            HudVertex { position: [x + DX, y - DY], texture_coordinates: [tx + s, ty + s], texture_index: 0, value: 1.0 },
            HudVertex { position: [x,      y - DY], texture_coordinates: [tx,     ty + s], texture_index: 0, value: 1.0 },
        ];

        #[rustfmt::skip]
        let indices = [
            1 + index_offset, index_offset, 2 + index_offset,
            2 + index_offset, index_offset, 3 + index_offset,
        ];

        (vertices, indices)
    }

    pub fn string_geometry(
        &self,
        mut x: f32,
        mut y: f32,
        string: &str,
    ) -> Geometry<HudVertex, u16> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // TODO unicode?? ? ???
        let ascii = string.replace(|c: char| !c.is_ascii(), "");

        for &c in ascii.as_bytes() {
            let index_offset = vertices.len().try_into().unwrap();
            let (v, i) = self.char_geometry(x, y, c, index_offset);
            vertices.extend(&v);
            indices.extend(&i);

            x += DX * (CHARACTER_WIDTHS[c as usize] as f32 / 8.0);
            if x >= 1.0 {
                x = 0.0;
                y -= DY;
            }
        }

        Geometry::new(vertices, indices)
    }

    pub fn string_to_buffers(
        &self,
        render_context: &RenderContext,
        x: f32,
        y: f32,
        string: &str,
    ) -> GeometryBuffers<u16> {
        let geometry = self.string_geometry(x, y, string);
        GeometryBuffers::from_geometry(render_context, &geometry, wgpu::BufferUsage::empty())
    }
}
