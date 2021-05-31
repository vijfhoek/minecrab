use cgmath::Vector3;

use crate::{quad::Quad, vertex::Vertex};

#[allow(clippy::many_single_char_names)]
pub fn vertices(
    quad: &Quad,
    y: i32,
    z_height: f32,
    offset: Vector3<i32>,
    texture_indices: (usize, usize, usize, usize, usize, usize),
) -> [Vertex; 24] {
    let w = quad.w as f32;
    let h = quad.h as f32;
    let zh = z_height;

    let x = (quad.x + offset.x) as f32;
    let y = (y + offset.y) as f32;
    let z = (quad.y + offset.z) as f32;

    let t = texture_indices;

    #[rustfmt::skip]
    let vertices = [
        // Left
        Vertex { position: [x,     y,      z      ], texture_coordinates: [h,   1.0, t.0 as f32], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y,      z + h  ], texture_coordinates: [0.0, 1.0, t.0 as f32], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y + zh, z + h  ], texture_coordinates: [0.0, 0.0, t.0 as f32], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y + zh, z      ], texture_coordinates: [h,   0.0, t.0 as f32], normal: [-1.0, 0.0, 0.0] },

        // Right
        Vertex { position: [x + w, y,      z      ], texture_coordinates: [0.0, 1.0, t.1 as f32], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y,      z + h  ], texture_coordinates: [h,   1.0, t.1 as f32], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y + zh, z + h  ], texture_coordinates: [h,   0.0, t.1 as f32], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y + zh, z      ], texture_coordinates: [0.0, 0.0, t.1 as f32], normal: [1.0, 0.0, 0.0] },

        // Back
        Vertex { position: [x,     y,      z      ], texture_coordinates: [w,   1.0, t.2 as f32], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x,     y + zh, z      ], texture_coordinates: [w,   0.0, t.2 as f32], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x + w, y + zh, z      ], texture_coordinates: [0.0, 0.0, t.2 as f32], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x + w, y,      z      ], texture_coordinates: [0.0, 1.0, t.2 as f32], normal: [0.0, 0.0, -1.0] },

        // Front
        Vertex { position: [x,     y,      z + h  ], texture_coordinates: [0.0, 1.0, t.3 as f32], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x,     y + zh, z + h  ], texture_coordinates: [0.0, 0.0, t.3 as f32], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x + w, y + zh, z + h  ], texture_coordinates: [w,   0.0, t.3 as f32], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x + w, y,      z + h  ], texture_coordinates: [w,   1.0, t.3 as f32], normal: [0.0, 0.0, 1.0] },

        // Bottom
        Vertex { position: [x,     y,      z + 0.0], texture_coordinates: [w,   0.0, t.4 as f32], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x,     y,      z + h  ], texture_coordinates: [w,   h,   t.4 as f32], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x + w, y,      z + h  ], texture_coordinates: [0.0, h,   t.4 as f32], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x + w, y,      z      ], texture_coordinates: [0.0, 0.0, t.4 as f32], normal: [0.0, -1.0, 0.0] },

        // Top
        Vertex { position: [x,     y + zh, z      ], texture_coordinates: [0.0, 0.0, t.5 as f32], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x,     y + zh, z + h  ], texture_coordinates: [0.0, h,   t.5 as f32], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x + w, y + zh, z + h  ], texture_coordinates: [w,   h,   t.5 as f32], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x + w, y + zh, z      ], texture_coordinates: [w,   0.0, t.5 as f32], normal: [0.0, 1.0, 0.0] },
    ];
    vertices
}

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    2, 0, 1,
    3, 0, 2,

    5, 4, 6,
    6, 4, 7,

    10, 8, 9,
    11, 8, 10,

    13, 12, 14,
    14, 12, 15,

    16, 18, 17,
    16, 19, 18,

    20, 21, 22,
    20, 22, 23,
];
