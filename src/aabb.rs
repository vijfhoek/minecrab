use std::ops::Sub;

use cgmath::{Point3, Vector3};
use itertools::Itertools;

type T = f32;

#[derive(Debug)]
pub struct Aabb {
    pub min: Point3<T>,
    pub max: Point3<T>,
}

impl Aabb {
    pub fn intersects(&self, other: &Self) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
            && (self.min.z <= other.max.z && self.max.z >= other.min.z)
    }

    pub fn get_corners(&self) -> [Point3<T>; 8] {
        [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
        ]
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
