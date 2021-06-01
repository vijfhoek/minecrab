use cgmath::Vector3;

use crate::{
    chunk::{FaceFlags, FACE_BACK, FACE_BOTTOM, FACE_FRONT, FACE_LEFT, FACE_RIGHT, FACE_TOP},
    quad::Quad,
    vertex::BlockVertex,
};

#[allow(clippy::many_single_char_names)]
#[rustfmt::skip]
pub fn vertices(
    quad: &Quad,
    y: i32,
    z_height: f32,
    offset: Vector3<i32>,
    texture_indices: (usize, usize, usize, usize, usize, usize),
    highlighted: Vector3<i32>,
    visible_faces: FaceFlags,
    start_index: u16,
) -> (Vec<BlockVertex>, Vec<u16>) {
    let w = quad.w as f32;
    let h = quad.h as f32;
    let zh = z_height;

    let x = (quad.x + offset.x) as f32;
    let y = (y + offset.y) as f32;
    let z = (quad.y + offset.z) as f32;

    let t = texture_indices;

    let mut current_index = start_index;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let highlighted: [f32; 3] = (-highlighted).cast().unwrap().into();

    if visible_faces & FACE_LEFT == FACE_LEFT {
        let normal = [-1.0,  0.0,  0.0];
        let highlighted = (normal == highlighted) as i32;
        vertices.extend(&[
            BlockVertex { position: [x, y,      z    ], texture_coordinates: [h,   1.0], texture_id: t.0 as i32, normal, highlighted },
            BlockVertex { position: [x, y,      z + h], texture_coordinates: [0.0, 1.0], texture_id: t.0 as i32, normal, highlighted },
            BlockVertex { position: [x, y + zh, z + h], texture_coordinates: [0.0, 0.0], texture_id: t.0 as i32, normal, highlighted },
            BlockVertex { position: [x, y + zh, z    ], texture_coordinates: [h,   0.0], texture_id: t.0 as i32, normal, highlighted },
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
            BlockVertex { position: [x + w, y,      z    ], texture_coordinates: [0.0, 1.0], texture_id: t.1 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y,      z + h], texture_coordinates: [h,   1.0], texture_id: t.1 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z + h], texture_coordinates: [h,   0.0], texture_id: t.1 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z    ], texture_coordinates: [0.0, 0.0], texture_id: t.1 as i32, normal, highlighted },
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
            BlockVertex { position: [x,     y,      z], texture_coordinates: [w,   1.0], texture_id: t.2 as i32, normal, highlighted },
            BlockVertex { position: [x,     y + zh, z], texture_coordinates: [w,   0.0], texture_id: t.2 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z], texture_coordinates: [0.0, 0.0], texture_id: t.2 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y,      z], texture_coordinates: [0.0, 1.0], texture_id: t.2 as i32, normal, highlighted },
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
            BlockVertex { position: [x,     y,      z + h], texture_coordinates: [0.0, 1.0], texture_id: t.3 as i32, normal, highlighted },
            BlockVertex { position: [x,     y + zh, z + h], texture_coordinates: [0.0, 0.0], texture_id: t.3 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z + h], texture_coordinates: [w,   0.0], texture_id: t.3 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y,      z + h], texture_coordinates: [w,   1.0], texture_id: t.3 as i32, normal, highlighted },
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
            BlockVertex { position: [x,     y, z + h], texture_coordinates: [w,   h  ], texture_id: t.4 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y, z + h], texture_coordinates: [0.0, h  ], texture_id: t.4 as i32, normal, highlighted },
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
            BlockVertex { position: [x,     y + zh, z    ], texture_coordinates: [0.0, 0.0], texture_id: t.5 as i32, normal, highlighted },
            BlockVertex { position: [x,     y + zh, z + h], texture_coordinates: [0.0, h  ], texture_id: t.5 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z + h], texture_coordinates: [w,   h  ], texture_id: t.5 as i32, normal, highlighted },
            BlockVertex { position: [x + w, y + zh, z    ], texture_coordinates: [w,   0.0], texture_id: t.5 as i32, normal, highlighted },
        ]);
        indices.extend(&[
            current_index, 1 + current_index, 2 + current_index,
            current_index, 2 + current_index, 3 + current_index,
        ]);
    }

    (vertices, indices)
}
