use crate::vertex::Vertex;

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texture_coordinates: [0.0, 0.0],
    }, // 0
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texture_coordinates: [0.0, 1.0],
    }, // 1
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texture_coordinates: [1.0, 1.0],
    }, // 2
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texture_coordinates: [1.0, 0.0],
    }, // 3
    Vertex {
        position: [0.5, -0.5, -0.5],
        texture_coordinates: [1.0, 1.0],
    }, // 4
    Vertex {
        position: [0.5, -0.5, 0.5],
        texture_coordinates: [1.0, 0.0],
    }, // 5
    Vertex {
        position: [0.5, 0.5, 0.5],
        texture_coordinates: [0.0, 0.0],
    }, // 6
    Vertex {
        position: [0.5, 0.5, -0.5],
        texture_coordinates: [0.0, 1.0],
    }, // 7
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    // left
    0, 1, 2,
    0, 2, 3,

    // bottom
    1, 0, 4,
    1, 4, 5,

    // right
    5, 7, 6,
    5, 4, 7,

    // top
    3, 2, 6,
    3, 6, 7,

    // front
    2, 1, 5,
    2, 5, 6,

    // back
    0, 3, 7,
    0, 7, 4,
];
