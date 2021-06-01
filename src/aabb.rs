use cgmath::Vector3;
use std::ops::{Div, Sub};

pub struct Aabb<T> {
    pub min: Vector3<T>,
    pub max: Vector3<T>,
}

impl<T: Ord + Copy + Sub<Output = T> + Div<Output = T>> Aabb<T> {
    pub fn intersects_ray(
        &self,
        ray_origin: Vector3<T>,
        ray_direction: Vector3<T>,
    ) -> Option<(T, T)> {
        let mut t_min = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (self.max.x - ray_origin.x) / ray_direction.x;
        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut ty_max = (self.max.y - ray_origin.y) / ray_direction.y;
        if ty_min > ty_max {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        if t_min > ty_max || ty_min > t_max {
            return None;
        }

        t_min = T::max(t_min, ty_min);
        t_max = T::min(t_min, ty_max);

        let mut tz_min = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut tz_max = (self.max.z - ray_origin.z) / ray_direction.z;
        if tz_min > tz_max {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        if t_min > tz_max || tz_min > t_max {
            return None;
        }

        t_min = T::max(t_min, tz_min);
        t_max = T::max(t_max, tz_max);

        Some((t_min, t_max))
    }
}
