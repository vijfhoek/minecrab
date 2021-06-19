use std::mem::size_of;

use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector4, Zero};
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsage};

use crate::{
    aabb::Aabb,
    camera::{Camera, Projection, OPENGL_TO_WGPU_MATRIX},
    render_context::RenderContext,
};

pub struct View {
    position_vector: Vector4<f32>,
    projection_matrix: Matrix4<f32>,
    pub frustrum_aabb: Aabb,

    pub camera: Camera,
    pub projection: Projection,

    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl View {
    pub fn to_raw(&self) -> ViewRaw {
        ViewRaw {
            view_position: self.position_vector.into(),
            view_projection: self.projection_matrix.into(),
        }
    }

    pub fn new(render_context: &RenderContext) -> Self {
        let camera = Camera::new(
            (10.0, 140.0, 10.0).into(),
            cgmath::Deg(45.0).into(),
            cgmath::Deg(-20.0).into(),
        );

        let projection = Projection::new(
            render_context.swap_chain_descriptor.width,
            render_context.swap_chain_descriptor.height,
            cgmath::Deg(45.0),
            0.1,
            300.0,
        );

        let buffer = render_context.device.create_buffer(&BufferDescriptor {
            label: Some("view buffer"),
            size: size_of::<ViewRaw>() as u64,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("view_bind_group_layout"),
                });

        let bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("view_bind_group"),
            });

        Self {
            position_vector: Vector4::zero(),
            projection_matrix: Matrix4::identity(),
            frustrum_aabb: Aabb::default(),
            camera,
            projection,

            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_view_projection(&mut self, render_context: &RenderContext) {
        self.position_vector = self.camera.position.to_homogeneous();
        self.projection_matrix =
            self.projection.calculate_matrix() * self.camera.calculate_matrix();
        self.frustrum_aabb = self.frustrum_aabb();

        render_context
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.to_raw()]));
    }

    fn frustrum_aabb(&self) -> Aabb {
        let projection = OPENGL_TO_WGPU_MATRIX.invert().unwrap() * self.projection_matrix;
        let inverse_matrix = projection.invert().unwrap();

        let corners = [
            Vector4::new(-1.0, -1.0, 1.0, 1.0),
            Vector4::new(-1.0, -1.0, -1.0, 1.0),
            Vector4::new(-1.0, 1.0, 1.0, 1.0),
            Vector4::new(-1.0, 1.0, -1.0, 1.0),
            Vector4::new(1.0, -1.0, 1.0, 1.0),
            Vector4::new(1.0, -1.0, -1.0, 1.0),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
            Vector4::new(1.0, 1.0, -1.0, 1.0),
        ];

        let mut min = Vector4::new(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
        let mut max = Vector4::new(0.0, 0.0, 0.0, 1.0);
        for corner in corners {
            let corner = inverse_matrix * corner;
            let corner = corner / corner.w;

            min = min.zip(corner, f32::min);
            max = max.zip(corner, f32::max);
        }

        Aabb {
            min: Point3::from_vec(min.truncate()),
            max: Point3::from_vec(max.truncate()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewRaw {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}
