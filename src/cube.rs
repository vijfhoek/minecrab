use cgmath::Vector3;

use crate::{quad::Quad, vertex::Vertex};

const S: f32 = 512.0 / 4096.0;

pub fn vertices(
    quad: &Quad,
    y: i32,
    offset: Vector3<i32>,
    left: (i32, i32),
    right: (i32, i32),
    back: (i32, i32),
    front: (i32, i32),
    bottom: (i32, i32),
    top: (i32, i32),
) -> [Vertex; 24] {
    let w = quad.w as f32;
    let h = quad.h as f32;

    let x = (quad.x + offset.x) as f32;
    let y = (y + offset.y) as f32;
    let z = (quad.y + offset.z) as f32;

    #[rustfmt::skip]
    let vertices = [
        // Left
        Vertex { position: [x,     y,       z      ], texture_coordinates: [left.0 as f32 * S + S, left.1 as f32 * S + S], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y,       z + h  ], texture_coordinates: [left.0 as f32 * S,     left.1 as f32 * S + S], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y + 1.0, z + h  ], texture_coordinates: [left.0 as f32 * S,     left.1 as f32 * S    ], normal: [-1.0, 0.0, 0.0] },
        Vertex { position: [x,     y + 1.0, z      ], texture_coordinates: [left.0 as f32 * S + S, left.1 as f32 * S    ], normal: [-1.0, 0.0, 0.0] },

        // Right
        Vertex { position: [x + w, y,       z      ], texture_coordinates: [right.0 as f32 * S,     right.1 as f32 * S + S], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y,       z + h  ], texture_coordinates: [right.0 as f32 * S + S, right.1 as f32 * S + S], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y + 1.0, z + h  ], texture_coordinates: [right.0 as f32 * S + S, right.1 as f32 * S    ], normal: [1.0, 0.0, 0.0] },
        Vertex { position: [x + w, y + 1.0, z      ], texture_coordinates: [right.0 as f32 * S,     right.1 as f32 * S    ], normal: [1.0, 0.0, 0.0] },

        // Back
        Vertex { position: [x,     y,       z      ], texture_coordinates: [back.0 as f32 * S + S, back.1 as f32 * S + S], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x,     y + 1.0, z      ], texture_coordinates: [back.0 as f32 * S + S, back.1 as f32 * S    ], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x + w, y + 1.0, z      ], texture_coordinates: [back.0 as f32 * S,     back.1 as f32 * S    ], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [x + w, y,       z      ], texture_coordinates: [back.0 as f32 * S,     back.1 as f32 * S + S], normal: [0.0, 0.0, -1.0] },

        // Front
        Vertex { position: [x,     y,       z + h], texture_coordinates: [front.0 as f32 * S,     front.1 as f32 * S + S], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x,     y + 1.0, z + h], texture_coordinates: [front.0 as f32 * S,     front.1 as f32 * S    ], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x + w, y + 1.0, z + h], texture_coordinates: [front.0 as f32 * S + S, front.1 as f32 * S    ], normal: [0.0, 0.0, 1.0] },
        Vertex { position: [x + w, y,       z + h], texture_coordinates: [front.0 as f32 * S + S, front.1 as f32 * S + S], normal: [0.0, 0.0, 1.0] },

        // Bottom
        Vertex { position: [x,     y, z + 0.0], texture_coordinates: [bottom.0 as f32 * S + S, bottom.1 as f32 * S    ], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x,     y, z + h  ], texture_coordinates: [bottom.0 as f32 * S + S, bottom.1 as f32 * S + S], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x + w, y, z + h  ], texture_coordinates: [bottom.0 as f32 * S,     bottom.1 as f32 * S + S], normal: [0.0, -1.0, 0.0] },
        Vertex { position: [x + w, y, z      ], texture_coordinates: [bottom.0 as f32 * S,     bottom.1 as f32 * S    ], normal: [0.0, -1.0, 0.0] },

        // Top
        Vertex { position: [x,     y + 1.0, z    ], texture_coordinates: [top.0 as f32 * S,     top.1 as f32 * S    ], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x,     y + 1.0, z + h], texture_coordinates: [top.0 as f32 * S,     top.1 as f32 * S + S], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x + w, y + 1.0, z + h], texture_coordinates: [top.0 as f32 * S + S, top.1 as f32 * S + S], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [x + w, y + 1.0, z    ], texture_coordinates: [top.0 as f32 * S + S, top.1 as f32 * S    ], normal: [0.0, 1.0, 0.0] },
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
