use std::convert::TryInto;

use wgpu::{BufferUsage, RenderPass};

use crate::{
    geometry::Geometry,
    geometry_buffers::GeometryBuffers,
    hud::{UI_SCALE_X, UI_SCALE_Y},
    render_context::RenderContext,
    vertex::HudVertex,
    world::block::BlockType,
};

pub struct HotbarHud {
    pub blocks: [Option<BlockType>; 9],
    pub last_blocks: [Option<BlockType>; 9],

    pub geometry_buffers: GeometryBuffers<u16>,
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
            None,
            None,
            None,
        ];

        Self {
            blocks: hotbar_blocks,
            last_blocks: [None; 9],

            geometry_buffers: GeometryBuffers::from_geometry(
                render_context,
                &Geometry::<HudVertex, _>::default(),
                BufferUsage::empty(),
            ),
        }
    }

    pub fn update(&mut self, render_context: &RenderContext) {
        if self.blocks != self.last_blocks {
            self.geometry_buffers = GeometryBuffers::from_geometry(
                render_context,
                &self.block_vertices(),
                wgpu::BufferUsage::empty(),
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

        let mut index = 0;
        for cursor_index in 0..9 {
            if let Some(block) = self.blocks[cursor_index as usize] {
                let x = (-92 + 20 * cursor_index as i32) as f32;
                let texture_index = block.texture_indices().2.try_into().unwrap();

                #[rustfmt::skip]
                vertices.extend(&[
                    HudVertex { position: [UI_SCALE_X * (x +  5.0), -1.0 + UI_SCALE_Y * 18.0], texture_coordinates: [0.0,  0.0], texture_index },
                    HudVertex { position: [UI_SCALE_X * (x + 19.0), -1.0 + UI_SCALE_Y * 18.0], texture_coordinates: [1.0,  0.0], texture_index },
                    HudVertex { position: [UI_SCALE_X * (x + 19.0), -1.0 + UI_SCALE_Y *  4.0], texture_coordinates: [1.0,  1.0], texture_index },
                    HudVertex { position: [UI_SCALE_X * (x +  5.0), -1.0 + UI_SCALE_Y *  4.0], texture_coordinates: [0.0,  1.0], texture_index },
                ]);

                #[rustfmt::skip]
                indices.extend(&[
                    index, 2 + index, 1 + index,
                    index, 3 + index, 2 + index,
                ]);
                index += 4;
            }
        }

        Geometry::new(vertices, indices)
    }
}
