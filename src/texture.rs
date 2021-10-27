use std::{num::NonZeroU32, ops::Range};

use anyhow::Context;
use cgmath::{Vector2, Zero};
use image::{EncodableLayout, ImageBuffer, Rgba};
use wgpu::Origin3d;

use crate::render_context::RenderContext;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub sampler: Option<wgpu::Sampler>,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(render_context: &RenderContext, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: render_context.swap_chain_descriptor.width,
            height: render_context.swap_chain_descriptor.height,
            depth_or_array_layers: 1,
        };

        let texture = render_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

        Self {
            texture,
            sampler: Some(sampler),
            view,
        }
    }

    fn from_rgba8(
        render_context: &RenderContext,
        rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        origin: Vector2<u32>,
        size: Vector2<u32>,
        label: &str,
    ) -> anyhow::Result<Self> {
        let texture_size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let texture = render_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsage::SAMPLED
                    | wgpu::TextureUsage::COPY_DST
                    | wgpu::TextureUsage::COPY_SRC,
            });

        let stride = 4 * rgba.width();
        let offset = (origin.y * stride + origin.x * 4) as usize;
        render_context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba.as_bytes()[offset..offset + (size.y * stride) as usize],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(stride),
                rows_per_image: NonZeroU32::new(size.y),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("texture_view_{}", label)),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..wgpu::TextureViewDescriptor::default()
        });

        Ok(Self {
            texture,
            sampler: None,
            view,
        })
    }

    pub fn from_bytes(
        render_context: &RenderContext,
        bytes: &[u8],
        label: &str,
    ) -> anyhow::Result<Self> {
        let image = image::load_from_memory(bytes)?;
        let rgba = image.into_rgba8();
        let (width, height) = rgba.dimensions();
        Self::from_rgba8(
            render_context,
            &rgba,
            Vector2::zero(),
            Vector2::new(width, height),
            label,
        )
    }

    pub fn from_bytes_atlas(
        render_context: &RenderContext,
        bytes: &[u8],
        tile_size: Vector2<u32>,
        label: &str,
    ) -> anyhow::Result<Vec<Self>> {
        let image = image::load_from_memory(bytes)?;
        let rgba = image.into_rgba8();

        let (width, height) = rgba.dimensions();
        assert_eq!(width % tile_size.x, 0);
        assert_eq!(height % tile_size.y, 0);

        let mut tiles = Vec::new();
        for y in (0..height).step_by(tile_size.y as usize) {
            for x in (0..width).step_by(tile_size.x as usize) {
                tiles.push(Self::from_rgba8(
                    render_context,
                    &rgba,
                    Vector2::new(x, y),
                    tile_size,
                    &format!("{}({},{})", label, x, y),
                )?);
            }
        }

        Ok(tiles)
    }
}

pub const TEXTURE_COUNT: usize = 44;

pub struct TextureManager {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,

    pub textures: Vec<Texture>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl TextureManager {
    pub fn new(render_context: &RenderContext) -> Self {
        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture_bind_group_layout"),
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

        let sampler = render_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                ..wgpu::SamplerDescriptor::default()
            });

        Self {
            bind_group_layout,
            sampler,

            textures: Vec::new(),
            bind_group: None,
        }
    }

    pub fn load_all(&mut self, render_context: &RenderContext) -> anyhow::Result<()> {
        let tile_size = Vector2::new(16, 16);

        self.load(render_context, "assets/block/cobblestone.png")?; // 0
        self.load(render_context, "assets/block/dirt.png")?; // 1
        self.load(render_context, "assets/block/stone.png")?; // 2
        self.load(render_context, "assets/grass_block_top_plains.png")?; // 3
        self.load(render_context, "assets/grass_block_side_plains.png")?; // 4
        self.load(render_context, "assets/block/bedrock.png")?; // 5
        self.load(render_context, "assets/block/sand.png")?; // 6
        self.load(render_context, "assets/block/gravel.png")?; // 7
        self.load_atlas(render_context, "assets/block/water_still.png", tile_size)?; // 8 - 39
        self.load(render_context, "assets/block/oak_log.png")?; // 40
        self.load(render_context, "assets/block/oak_log_top.png")?; // 41
        self.load(render_context, "assets/block/oak_planks.png")?; // 42
        self.load(render_context, "assets/block/oak_leaves.png")?; // 43
        assert_eq!(TEXTURE_COUNT, self.textures.len());

        let texture_array = render_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: TEXTURE_COUNT as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            });

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("texture copy encoder"),
                });

        for (i, texture) in self.textures.iter().enumerate() {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                },
                wgpu::ImageCopyTexture {
                    texture: &texture_array,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 1,
                },
            )
        }

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        let view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            array_layer_count: NonZeroU32::new(TEXTURE_COUNT as u32),
            ..wgpu::TextureViewDescriptor::default()
        });

        self.bind_group = Some(render_context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some(&("Block texture bind group")),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            },
        ));

        Ok(())
    }

    pub fn load(&mut self, render_context: &RenderContext, path: &str) -> anyhow::Result<usize> {
        let bytes = std::fs::read(path).context(format!("Failed to load {}", path))?;
        let texture = Texture::from_bytes(render_context, &bytes, path)
            .context(format!("Failed to decode {}", path))?;

        let id = self.textures.len();
        self.textures.push(texture);

        println!("loaded {} to {}", path, id);
        Ok(id)
    }

    pub fn load_atlas(
        &mut self,
        render_context: &RenderContext,
        path: &str,
        tile_size: Vector2<u32>,
    ) -> anyhow::Result<Range<usize>> {
        let bytes = std::fs::read(path).context(format!("Failed to load {}", path))?;
        let mut textures = Texture::from_bytes_atlas(render_context, &bytes, tile_size, path)
            .context(format!("Failed to decode {}", path))?;

        let start = self.textures.len();
        self.textures.append(&mut textures);
        let end = self.textures.len();

        println!("loaded atlas {} to {}..{}", path, start, end);
        Ok(start..end)
    }
}
