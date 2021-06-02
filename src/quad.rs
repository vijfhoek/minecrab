use cgmath::{Vector3, Zero};

use crate::{
    chunk::{
        BlockType, FaceFlags, FACE_ALL, FACE_BACK, FACE_BOTTOM, FACE_FRONT, FACE_LEFT, FACE_RIGHT,
        FACE_TOP,
    },
    geometry::Geometry,
    vertex::BlockVertex,
};

#[derive(Debug)]
pub struct Quad {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,

    pub highlighted_normal: Vector3<i32>,
    pub visible_faces: FaceFlags,
}

impl Quad {
    pub fn new(x: i32, y: i32, dx: i32, dy: i32) -> Self {
        Quad {
            x,
            y,
            dx,
            dy,

            /// The normal of the face that was highlighted.
            ///
            /// Set to Vector3::zero if no faces are highlighted.
            highlighted_normal: Vector3::zero(),

            /// Bitmap of the visible faces.
            visible_faces: FACE_ALL,
        }
    }

    /// Converts the quad to `Geometry` (i.e. a list of vertices and indices) to be rendered.
    ///
    /// # Arguments
    ///
    /// * `translation` - How much to translate the quad for when rendering.
    /// * `block_type` - The type of the block. Used for determining the texture indices.
    /// * `start_index` - Which geometry index to start at.
    #[allow(clippy::many_single_char_names)]
    #[rustfmt::skip]
    pub fn to_geometry(
        &self,
        translation: Vector3<i32>,
        block_type: BlockType,
        start_index: u16,
    ) -> Geometry<BlockVertex> {
        let dx = self.dx as f32;
        let dz = self.dy as f32;
        let dy = 1.0;

        let x = (self.x + translation.x) as f32;
        let y = translation.y as f32;
        let z = (self.y + translation.z) as f32;

        let t = block_type.texture_indices();

        let mut current_index = start_index;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let highlighted: [f32; 3] = self.highlighted_normal.cast().unwrap().into();

        if self.visible_faces & FACE_LEFT == FACE_LEFT {
            let normal = [-1.0,  0.0,  0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x, y,      z     ], texture_coordinates: [dz,  1.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y,      z + dz], texture_coordinates: [0.0, 1.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y + dy, z + dz], texture_coordinates: [0.0, 0.0], texture_id: t.0 as i32, normal, highlighted },
                BlockVertex { position: [x, y + dy, z     ], texture_coordinates: [dz,  0.0], texture_id: t.0 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_RIGHT == FACE_RIGHT {
            let normal = [1.0, 0.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x + dx, y,      z     ], texture_coordinates: [0.0, 1.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y,      z + dz], texture_coordinates: [dz,  1.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dz,  0.0], texture_id: t.1 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.1 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_BACK == FACE_BACK {
            let normal = [0.0, 0.0, -1.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,      y,      z], texture_coordinates: [dx,  1.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x,      y + dy, z], texture_coordinates: [dx,  0.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z], texture_coordinates: [0.0, 0.0], texture_id: t.2 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y,      z], texture_coordinates: [0.0, 1.0], texture_id: t.2 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_FRONT == FACE_FRONT {
            let normal = [0.0, 0.0, 1.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,      y,      z + dz], texture_coordinates: [0.0, 1.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x,      y + dy, z + dz], texture_coordinates: [0.0, 0.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dx,  0.0], texture_id: t.3 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y,      z + dz], texture_coordinates: [dx,  1.0], texture_id: t.3 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_BOTTOM == FACE_BOTTOM {
            let normal = [0.0, -1.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,      y, z     ], texture_coordinates: [dx,  0.0], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x,      y, z + dz], texture_coordinates: [dx,  dz ], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y, z + dz], texture_coordinates: [0.0, dz ], texture_id: t.4 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.4 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                current_index, 2 + current_index, 1 + current_index,
                current_index, 3 + current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_TOP == FACE_TOP {
            let normal = [0.0, 1.0, 0.0];
            let highlighted = (normal == highlighted) as i32;
            vertices.extend(&[
                BlockVertex { position: [x,      y + dy, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x,      y + dy, z + dz], texture_coordinates: [0.0, dz ], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dx,  dz ], texture_id: t.5 as i32, normal, highlighted },
                BlockVertex { position: [x + dx, y + dy, z     ], texture_coordinates: [dx,  0.0], texture_id: t.5 as i32, normal, highlighted },
            ]);
            indices.extend(&[
                current_index, 1 + current_index, 2 + current_index,
                current_index, 2 + current_index, 3 + current_index,
            ]);
        }

        Geometry::new(vertices, indices)
    }
}
