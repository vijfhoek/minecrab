use cgmath::Point3;

#[derive(Debug)]
pub struct Aabb {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Aabb {
    pub fn intersects(&self, other: &Self) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
            && (self.min.z <= other.max.z && self.max.z >= other.min.z)
    }

    /// Gets the corners of the AABB that should be checked when checking
    /// collision with the world.
    ///
    /// Returns a `Vec` of all `Point3`s that cover the faces of `self` with
    /// no more than 1 unit of distance between them.
    pub fn get_corners(&self) -> Vec<Point3<f32>> {
        let mut corners = Vec::new();

        let mut x = self.min.x;
        while x < self.max.x.ceil() {
            let mut y = self.min.y;
            while y < self.max.y.ceil() {
                let mut z = self.min.z;
                while z < self.max.z.ceil() {
                    corners.push(Point3::new(
                        x.min(self.max.x),
                        y.min(self.max.y),
                        z.min(self.max.z),
                    ));
                    z += 1.0;
                }
                y += 1.0;
            }
            x += 1.0;
        }

        corners
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        }
    }
}
