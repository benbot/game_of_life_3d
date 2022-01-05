extern crate nalgebra as na;
use na::Matrix4;

#[derive(Debug)]
pub struct Camera {
    pub pos: na::Point<f32, 3>,
    pub target: na::Point<f32, 3>,
    pub buffer: wgpu::Buffer,
    pub rot_x: f32,
    pub rot_y: f32,
}

impl Camera {
    fn _layout(_state: &crate::RenderState) -> wgpu::VertexBufferLayout {
        todo!();
    }

    pub fn get_transform(&self, state: &crate::RenderState) -> Matrix4<f32> {
        let size = state.window.inner_size();

        let normal = (self.target - self.pos).normalize();

        let cam =
            Matrix4::new_perspective(size.width as f32 / size.height as f32, 45.0, 0.1, 100.0);

        let x_rot_mat = na::UnitQuaternion::from_axis_angle(
            &na::UnitVector3::new_normalize(na::Vector3::x()),
            self.rot_x * std::f32::consts::FRAC_2_PI,
        );
        let y_rot_mat = na::UnitQuaternion::from_axis_angle(
            &na::UnitVector3::new_normalize(na::Vector3::y()),
            self.rot_y * std::f32::consts::FRAC_2_PI,
        );

        let rot_quat = x_rot_mat * y_rot_mat;

        let look = Matrix4::look_at_rh(&self.pos, &normal.into(), &na::Vector3::y());

        cam * look
    }
}
