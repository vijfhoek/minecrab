use crate::vertex::Vertex;

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    // Left
    Vertex { position: [-1.0, -1.0, -1.0], texture_coordinates: [1.0, 1.0], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-1.0, -1.0,  1.0], texture_coordinates: [0.0, 1.0], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-1.0,  1.0,  1.0], texture_coordinates: [0.0, 0.0], normal: [-1.0,  0.0,  0.0] },
    Vertex { position: [-1.0,  1.0, -1.0], texture_coordinates: [1.0, 0.0], normal: [-1.0,  0.0,  0.0] },

    // Right
    Vertex { position: [ 1.0, -1.0, -1.0], texture_coordinates: [0.0, 1.0], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], texture_coordinates: [1.0, 1.0], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], texture_coordinates: [1.0, 0.0], normal: [ 1.0,  0.0,  0.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], texture_coordinates: [0.0, 0.0], normal: [ 1.0,  0.0,  0.0] },

    // Back
    Vertex { position: [-1.0, -1.0, -1.0], texture_coordinates: [1.0, 1.0], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], texture_coordinates: [1.0, 0.0], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], texture_coordinates: [0.0, 0.0], normal: [ 0.0,  0.0, -1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], texture_coordinates: [0.0, 1.0], normal: [ 0.0,  0.0, -1.0] },

    // Front
    Vertex { position: [-1.0, -1.0,  1.0], texture_coordinates: [0.0, 1.0], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], texture_coordinates: [0.0, 0.0], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], texture_coordinates: [1.0, 0.0], normal: [ 0.0,  0.0,  1.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], texture_coordinates: [1.0, 1.0], normal: [ 0.0,  0.0,  1.0] },

    // Bottom
    Vertex { position: [-1.0, -1.0, -1.0], texture_coordinates: [1.0, 0.0], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [-1.0, -1.0,  1.0], texture_coordinates: [1.0, 1.0], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], texture_coordinates: [0.0, 1.0], normal: [ 0.0, -1.0,  0.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], texture_coordinates: [0.0, 0.0], normal: [ 0.0, -1.0,  0.0] },

    // Top
    Vertex { position: [ -1.0, 1.0, -1.0], texture_coordinates: [0.0, 0.0], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [ -1.0, 1.0,  1.0], texture_coordinates: [0.0, 1.0], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [  1.0, 1.0,  1.0], texture_coordinates: [1.0, 1.0], normal: [ 0.0,  1.0,  0.0] },
    Vertex { position: [  1.0, 1.0, -1.0], texture_coordinates: [1.0, 0.0], normal: [ 0.0,  1.0,  0.0] },
];

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
