use cgmath::{Vector3, Zero};

use crate::{
    chunk::{
        BlockType, FaceFlags, FACE_BACK, FACE_BOTTOM, FACE_FRONT, FACE_LEFT, FACE_RIGHT, FACE_TOP,
    },
    geometry::Geometry,
    vertex::BlockVertex,
};

#[derive(Debug)]
pub struct Quad {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,

    pub highlighted_normal: Vector3<i32>,
}

impl Quad {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Quad {
            x,
            y,
            w,
            h,
            highlighted_normal: Vector3::zero(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[rustfmt::skip]
    pub fn to_geometry(
        &self,
        y: i32,
        offset: Vector3<i32>,
        block_type: BlockType,
        visible_faces: FaceFlags,
        start_index: u16,
    ) -> Geometry<BlockVertex> {
        let w = self.w as f32;
        let d = self.h as f32;
        let h = 1.0;

        let x = (self.x + offset.x) as f32;
        let y = (y + offset.y) as f32;
        let z = (self.y + offset.z) as f32;

        let t = block_type.texture_indices();

        let mut current_index = start_index;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let highlighted: [f32; 3] = self.highlighted_normal.cast().unwrap().into();

        if visible_faces & FACE_LEFT == FACE_LEFT {
            let normal = [-1.0,  0.0,  0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x, y,     z    ], texture_coordinates: [d,   1.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y,     z + d], texture_coordinates: [0.0, 1.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y + h, z + d], texture_coordinates: [0.0, 0.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y + h, z    ], texture_coordinates: [d,   0.0], texture_id: t.0 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if visible_faces & FACE_RIGHT == FACE_RIGHT {
            let normal = [1.0, 0.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x + w, y,     z    ], texture_coordinates: [0.0, 1.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y,     z + d], texture_coordinates: [d,   1.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z + d], texture_coordinates: [d,   0.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z    ], texture_coordinates: [0.0, 0.0], texture_id: t.1 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if visible_faces & FACE_BACK == FACE_BACK {
            let normal = [0.0, 0.0, -1.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,     y,     z], texture_coordinates: [w,   1.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x,     y + h, z], texture_coordinates: [w,   0.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z], texture_coordinates: [0.0, 0.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y,     z], texture_coordinates: [0.0, 1.0], texture_id: t.2 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if visible_faces & FACE_FRONT == FACE_FRONT {
            let normal = [0.0, 0.0, 1.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,     y,     z + d], texture_coordinates: [0.0, 1.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x,     y + h, z + d], texture_coordinates: [0.0, 0.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z + d], texture_coordinates: [w,   0.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y,     z + d], texture_coordinates: [w,   1.0], texture_id: t.3 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if visible_faces & FACE_BOTTOM == FACE_BOTTOM {
            let normal = [0.0, -1.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,     y, z    ], texture_coordinates: [w,   0.0], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x,     y, z + d], texture_coordinates: [w,   d  ], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y, z + d], texture_coordinates: [0.0, d  ], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y, z    ], texture_coordinates: [0.0, 0.0], texture_id: t.4 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                current_index, 2 + current_index, 1 + current_index,
                current_index, 3 + current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if visible_faces & FACE_TOP == FACE_TOP {
            let normal = [0.0, 1.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,     y + h, z    ], texture_coordinates: [0.0, 0.0], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x,     y + h, z + d], texture_coordinates: [0.0, d  ], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z + d], texture_coordinates: [w,   d  ], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x + w, y + h, z    ], texture_coordinates: [w,   0.0], texture_id: t.5 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                current_index, 1 + current_index, 2 + current_index,
                current_index, 2 + current_index, 3 + current_index,
            ]);
        }

        Geometry::new(vertices, indices)
    }
}
