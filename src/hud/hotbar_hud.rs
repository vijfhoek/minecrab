use cgmath::{ElementWise, Vector4};
use wgpu::{BufferUsages, RenderPass};

use crate::{
    geometry::Geometry,
    geometry_buffers::GeometryBuffers,
    render_context::RenderContext,
    vertex::HudVertex,
    world::block::BlockType,
    hud::{DEFAULT_UI_SCALE_X, DEFAULT_UI_SCALE_Y},
};

pub struct HotbarHud {
    pub blocks: [Option<BlockType>; 9],
    pub last_blocks: [Option<BlockType>; 9],

    pub geometry_buffers: GeometryBuffers<u16>,

    ui_scale_x: f32,
    ui_scale_y: f32,
}

impl HotbarHud {
    pub fn new(render_context: &RenderContext) -> Self {
        let hotbar_blocks = [
            Some(BlockType::Dirt),
            Some(BlockType::Stone),
            Some(BlockType::Sand),
            None,
            Some(BlockType::Grass),
            Some(BlockType::Cobblestone),
            Some(BlockType::OakPlanks),
            Some(BlockType::OakLog),
            Some(BlockType::OakLeaves),
        ];

        Self {
            blocks: hotbar_blocks,
            last_blocks: [None; 9],

            geometry_buffers: GeometryBuffers::from_geometry(
                render_context,
                &Geometry::<HudVertex, _>::default(),
                BufferUsages::empty(),
            ),

            ui_scale_x: DEFAULT_UI_SCALE_X,
            ui_scale_y: DEFAULT_UI_SCALE_Y,
        }
    }

    pub fn update(&mut self, render_context: &RenderContext) {
        if self.blocks != self.last_blocks {
            self.geometry_buffers = GeometryBuffers::from_geometry(
                render_context,
                &self.block_vertices(),
                wgpu::BufferUsages::empty(),
            );
        }
    }

    pub fn render<'a>(
        &'a self,
        render_context: &'a RenderContext,
        render_pass: &mut RenderPass<'a>,
    ) -> usize {
        let texture_manager = render_context.texture_manager.as_ref().unwrap();

        render_pass.set_bind_group(0, texture_manager.bind_group.as_ref().unwrap(), &[]);
        self.geometry_buffers.apply_buffers(render_pass);
        self.geometry_buffers.draw_indexed(render_pass)
    }

    fn block_vertices(&self) -> Geometry<HudVertex, u16> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut index_offset = 0;
        for slot in 0..9 {
            if let Some(block) = self.blocks[slot as usize] {
                let x = (-92 + 20 * slot as i32) as f32;
                let texture_indices = block.texture_indices();
                let color = block.color();

                let color_left = color
                    .mul_element_wise(Vector4::new(0.5, 0.5, 0.5, 1.0))
                    .into();
                let color_front = color
                    .mul_element_wise(Vector4::new(0.15, 0.15, 0.15, 1.0))
                    .into();
                let color_top = color.into();

                vertices.extend([
                    // Left face
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 3.5],
                        texture_coordinates: [1.0, 1.0],
                        texture_index: texture_indices.0 as i32,
                        color: color_left,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 5.0), -1.0 + self.ui_scale_y * 6.5],
                        texture_coordinates: [0.0, 1.0],
                        texture_index: texture_indices.0 as i32,
                        color: color_left,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 5.0), -1.0 + self.ui_scale_y * 15.5],
                        texture_coordinates: [0.0, 0.0],
                        texture_index: texture_indices.0 as i32,
                        color: color_left,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 12.5],
                        texture_coordinates: [1.0, 0.0],
                        texture_index: texture_indices.0 as i32,
                        color: color_left,
                    },
                    // Front face
                    HudVertex {
                        position: [self.ui_scale_x * (x + 19.0), -1.0 + self.ui_scale_y * 15.5],
                        texture_coordinates: [1.0, 0.0],
                        texture_index: texture_indices.3 as i32,
                        color: color_front,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 12.5],
                        texture_coordinates: [0.0, 0.0],
                        texture_index: texture_indices.3 as i32,
                        color: color_front,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 3.5],
                        texture_coordinates: [0.0, 1.0],
                        texture_index: texture_indices.3 as i32,
                        color: color_front,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 19.0), -1.0 + self.ui_scale_y * 6.5],
                        texture_coordinates: [1.0, 1.0],
                        texture_index: texture_indices.3 as i32,
                        color: color_front,
                    },
                    // Top face
                    HudVertex {
                        position: [self.ui_scale_x * (x + 19.0), -1.0 + self.ui_scale_y * 15.5],
                        texture_coordinates: [1.0, 0.0],
                        texture_index: texture_indices.5 as i32,
                        color: color_top,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 18.5],
                        texture_coordinates: [0.0, 0.0],
                        texture_index: texture_indices.5 as i32,
                        color: color_top,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 5.0), -1.0 + self.ui_scale_y * 15.5],
                        texture_coordinates: [0.0, 1.0],
                        texture_index: texture_indices.5 as i32,
                        color: color_top,
                    },
                    HudVertex {
                        position: [self.ui_scale_x * (x + 12.0), -1.0 + self.ui_scale_y * 12.5],
                        texture_coordinates: [1.0, 1.0],
                        texture_index: texture_indices.5 as i32,
                        color: color_top,
                    },
                ]);

                #[rustfmt::skip]
                indices.extend([
                    // Left face
                    2 + index_offset, index_offset, 1 + index_offset,
                    3 + index_offset, index_offset, 2 + index_offset,

                    // Right face
                    6 + index_offset, 4 + index_offset, 5 + index_offset,
                    7 + index_offset, 4 + index_offset, 6 + index_offset,

                    // Top face
                    10 + index_offset, 8 + index_offset, 9 + index_offset,
                    11 + index_offset, 8 + index_offset, 10 + index_offset,
                ]);

                index_offset += 12;
            }
        }

        Geometry::new(vertices, indices)
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.ui_scale_x = scale_x;
        self.ui_scale_y = scale_y;
    }
}
