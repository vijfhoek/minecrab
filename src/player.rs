use std::time::Duration;

use cgmath::{InnerSpace, Point3, Rad, Vector3};

use crate::{aabb::Aabb, render_context::RenderContext, utils, view::View, world::World};

pub struct Player {
    pub sprinting: bool,
    pub grounded: bool,
    pub creative: bool,

    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_speed: f32,

    pub view: View,
}

impl Player {
    pub fn new(render_context: &RenderContext) -> Self {
        let view = View::new(render_context);

        Self {
            sprinting: false,
            grounded: false,
            creative: false,

            forward_pressed: false,
            backward_pressed: false,
            left_pressed: false,
            right_pressed: false,
            up_speed: 0.0,

            view,
        }
    }

    /// Update the camera based on mouse dx and dy.
    pub fn update_camera(&mut self, dx: f64, dy: f64) {
        let camera = &mut self.view.camera;
        camera.yaw += Rad(dx as f32 * 0.003);
        camera.pitch -= Rad(dy as f32 * 0.003);

        if camera.pitch < Rad::from(cgmath::Deg(-80.0)) {
            camera.pitch = Rad::from(cgmath::Deg(-80.0));
        } else if camera.pitch > Rad::from(cgmath::Deg(89.9)) {
            camera.pitch = Rad::from(cgmath::Deg(89.9));
        }
    }

    /// Updates the player's position by their velocity, checks for and
    /// resolves any subsequent collisions, and then adds the jumping speed to
    /// the velocity.
    pub fn update_position(&mut self, dt: Duration, world: &World) {
        let (yaw_sin, yaw_cos) = self.view.camera.yaw.0.sin_cos();

        let speed = 10.0 * (self.sprinting as i32 * 2 + 1) as f32 * dt.as_secs_f32();

        let forward_speed = self.forward_pressed as i32 - self.backward_pressed as i32;
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin) * forward_speed as f32;

        let right_speed = self.right_pressed as i32 - self.left_pressed as i32;
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos) * right_speed as f32;

        let mut velocity = forward + right;
        if velocity.magnitude2() > 1.0 {
            velocity = velocity.normalize();
        }
        velocity *= speed;
        velocity.y = self.up_speed * 10.0 * dt.as_secs_f32();

        let mut new_position = self.view.camera.position;

        // y component (jumping)
        new_position.y += velocity.y;
        if let Some(aabb) = self.check_collision(new_position, world) {
            if self.up_speed < 0.0 {
                new_position.y = aabb.min.y.ceil() + 1.62;
                new_position.y = utils::f32_successor(new_position.y);
            } else if self.up_speed > 0.0 {
                new_position.y = aabb.max.y.floor() - 0.18;
                new_position.y = utils::f32_predecessor(new_position.y);
            }

            self.up_speed = 0.0;
            self.grounded = true;
        } else {
            self.grounded = false;
        }

        // x component
        new_position.x += velocity.x;
        if let Some(aabb) = self.check_collision(new_position, world) {
            if velocity.x < 0.0 {
                new_position.x = aabb.min.x.ceil() + 0.3;
                new_position.x = utils::f32_successor(new_position.x);
            } else if velocity.x > 0.0 {
                new_position.x = aabb.max.x.floor() - 0.3;
                new_position.x = utils::f32_predecessor(new_position.x);
            }
        }

        // z component
        new_position.z += velocity.z;
        if let Some(aabb) = self.check_collision(new_position, world) {
            if velocity.z < 0.0 {
                new_position.z = aabb.min.z.ceil() + 0.3;
                new_position.z = utils::f32_successor(new_position.z);
            } else if velocity.z > 0.0 {
                new_position.z = aabb.max.z.floor() - 0.3;
                new_position.z = utils::f32_predecessor(new_position.z);
            }
        }

        self.view.camera.position = new_position;

        if !self.creative {
            self.up_speed -= 1.6 * dt.as_secs_f32();
            self.up_speed *= 0.98_f32.powf(dt.as_secs_f32() / 20.0);
        }
    }

    fn check_collision(&self, position: Point3<f32>, world: &World) -> Option<Aabb> {
        let aabb = Aabb {
            min: position + Vector3::new(-0.3, -1.62, -0.3),
            max: position + Vector3::new(0.3, 0.18, 0.3),
        };

        for corner in &aabb.get_corners() {
            let block = world.get_block(corner.map(|x| x.floor() as isize));
            if block.is_some() {
                return Some(aabb);
            }
        }

        None
    }
}
