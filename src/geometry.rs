use crate::vertex::Vertex;

/// Represents a set of triangles by its vertices and indices.
#[derive(Default)]
pub struct Geometry<V: Vertex, I> {
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
}

impl<T: Vertex, I> Geometry<T, I> {
    pub fn new(vertices: Vec<T>, indices: Vec<I>) -> Self {
        Self { vertices, indices }
    }

    /// Moves all the vertices and indices of `other` into `Self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        self.vertices.append(&mut other.vertices);
        self.indices.append(&mut other.indices);
    }

    /// Returns the number of indices in the vertex.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}
