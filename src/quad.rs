#[derive(Debug)]
pub struct Quad {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Quad {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Quad { x, y, w, h }
    }
}
