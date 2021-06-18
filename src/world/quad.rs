use cgmath::{Point3, Vector3, Vector4, Zero};

use crate::{
    geometry::Geometry,
    vertex::BlockVertex,
    world::{block::BlockType, face_flags::*},
};

#[derive(Debug)]
pub struct Quad {
    pub position: Point3<isize>,
    pub dx: isize,
    pub dz: isize,

    pub highlighted_normal: Vector3<i32>,
    pub visible_faces: FaceFlags,
    pub block_type: Option<BlockType>,
}

impl Quad {
    pub fn new(position: Point3<isize>, dx: isize, dz: isize) -> Self {
        Quad {
            position,
            dx,
            dz,

            /// The normal of the face that was highlighted.
            ///
            /// Set to Vector3::zero if no faces are highlighted.
            highlighted_normal: Vector3::zero(),

            /// Bitmap of the visible faces.
            visible_faces: FACE_ALL,

            /// The `BlockType` of the blocks the quad describes.
            ///
            /// Used for determining which texture to map to it. When `None`, texture index 0 will be used.
            block_type: None,
        }
    }

    /// Converts the quad to `Geometry` (i.e. a list of vertices and indices) to be rendered.
    ///
    /// # Arguments
    ///
    /// * `start_index` - Which geometry index to start at.
    #[allow(clippy::many_single_char_names)]
    #[rustfmt::skip]
    pub fn to_geometry(
        &self,
        start_index: u16,
    ) -> Geometry<BlockVertex, u16> {
        let dx = self.dx as f32;
        let dz = self.dz as f32;
        let dy = 1.0;

        let x = self.position.x as f32;
        let y = self.position.y as f32;
        let z = self.position.z as f32;

        let (t, color) =  match self.block_type {
            Some(block_type) => (block_type.texture_indices(), block_type.color()),
            None => ((0, 0, 0, 0, 0, 0), Vector4::new(1.0, 1.0, 1.0, 1.0)),
        };
        let color = color.into();

        let mut current_index = start_index;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if self.visible_faces & FACE_LEFT == FACE_LEFT {
            let normal = Vector3::new(-1,  0,  0);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x, y,      z     ], texture_coordinates: [dz,  1.0], texture_id: t.0 as i32, normal, highlighted, color },
                BlockVertex { position: [x, y,      z + dz], texture_coordinates: [0.0, 1.0], texture_id: t.0 as i32, normal, highlighted, color },
                BlockVertex { position: [x, y + dy, z + dz], texture_coordinates: [0.0, 0.0], texture_id: t.0 as i32, normal, highlighted, color },
                BlockVertex { position: [x, y + dy, z     ], texture_coordinates: [dz,  0.0], texture_id: t.0 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_RIGHT == FACE_RIGHT {
            let normal = Vector3::new(1, 0, 0);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x + dx, y,      z     ], texture_coordinates: [0.0, 1.0], texture_id: t.1 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y,      z + dz], texture_coordinates: [dz,  1.0], texture_id: t.1 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dz,  0.0], texture_id: t.1 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.1 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_BACK == FACE_BACK {
            let normal = Vector3::new(0, 0, -1);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x,      y,      z], texture_coordinates: [dx,  1.0], texture_id: t.2 as i32, normal, highlighted, color },
                BlockVertex { position: [x,      y + dy, z], texture_coordinates: [dx,  0.0], texture_id: t.2 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z], texture_coordinates: [0.0, 0.0], texture_id: t.2 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y,      z], texture_coordinates: [0.0, 1.0], texture_id: t.2 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                2 + current_index, current_index, 1 + current_index,
                3 + current_index, current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_FRONT == FACE_FRONT {
            let normal = Vector3::new(0, 0, 1);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x,      y,      z + dz], texture_coordinates: [0.0, 1.0], texture_id: t.3 as i32, normal, highlighted, color },
                BlockVertex { position: [x,      y + dy, z + dz], texture_coordinates: [0.0, 0.0], texture_id: t.3 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dx,  0.0], texture_id: t.3 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y,      z + dz], texture_coordinates: [dx,  1.0], texture_id: t.3 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                1 + current_index, current_index, 2 + current_index,
                2 + current_index, current_index, 3 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_BOTTOM == FACE_BOTTOM {
            let normal = Vector3::new(0, -1, 0);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x,      y, z     ], texture_coordinates: [dx,  0.0], texture_id: t.4 as i32, normal, highlighted, color },
                BlockVertex { position: [x,      y, z + dz], texture_coordinates: [dx,  dz ], texture_id: t.4 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y, z + dz], texture_coordinates: [0.0, dz ], texture_id: t.4 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.4 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                current_index, 2 + current_index, 1 + current_index,
                current_index, 3 + current_index, 2 + current_index,
            ]);
            current_index += 4;
        }

        if self.visible_faces & FACE_TOP == FACE_TOP {
            let normal = Vector3::new(0, 1, 0);
            let highlighted = (self.highlighted_normal == normal) as i32;
            let normal = normal.cast().unwrap().into();
            vertices.extend(&[
                BlockVertex { position: [x,      y + dy, z     ], texture_coordinates: [0.0, 0.0], texture_id: t.5 as i32, normal, highlighted, color },
                BlockVertex { position: [x,      y + dy, z + dz], texture_coordinates: [0.0, dz ], texture_id: t.5 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z + dz], texture_coordinates: [dx,  dz ], texture_id: t.5 as i32, normal, highlighted, color },
                BlockVertex { position: [x + dx, y + dy, z     ], texture_coordinates: [dx,  0.0], texture_id: t.5 as i32, normal, highlighted, color },
            ]);
            indices.extend(&[
                current_index, 1 + current_index, 2 + current_index,
                current_index, 2 + current_index, 3 + current_index,
            ]);
        }

        Geometry::new(vertices, indices)
    }
}
