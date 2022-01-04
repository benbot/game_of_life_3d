extern crate nalgebra as na;
use na::{Matrix, Matrix4, Point3, UnitQuaternion, UnitVector3};

// use na::{Matrix4, Vector3};

#[derive(Debug)]
pub struct Camera {
    pub pos: na::Point<f32, 3>,
    pub target: na::Point<f32, 3>,
    pub rot_x: f32,
    pub rot_y: f32,
    pub buffer: wgpu::Buffer,
}

impl Camera {
    fn _layout(_state: &crate::RenderState) -> wgpu::VertexBufferLayout {
        todo!();
    }

    pub fn get_transform(&self, state: &crate::RenderState) -> Matrix4<f32> {
        let size = state.window.inner_size();
        let normal_up: na::Unit<na::Vector3<f32>> = na::Unit::new_normalize(na::Vector3::y());
        let normal_x: na::Unit<na::Vector3<f32>> = na::Unit::new_normalize(na::Vector3::y());
        let look = Matrix4::look_at_rh(&self.pos, &self.target, &normal_up);
        let cam =
            Matrix4::new_perspective(size.width as f32 / size.height as f32, 45.0, 0.1, 100.0);
        let rotation_y = UnitQuaternion::from_axis_angle(&normal_up, self.rot_y);
        let rotation_x = UnitQuaternion::from_axis_angle(&normal_x, self.rot_x);

        let transform = Matrix4::from(rotation_x * rotation_y);

        cam * look * transform
    }
}
