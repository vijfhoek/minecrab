use cgmath::Point3;

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
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        }
    }
}
