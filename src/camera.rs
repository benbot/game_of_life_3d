extern crate nalgebra as na;
use na::{Isometry3, Matrix4};

#[derive(Debug)]
pub struct Camera {
    pub pos: na::Point<f32, 3>,
    pub target: na::Point<f32, 3>,
    pub buffer: wgpu::Buffer,
    pub rot_x: f32,
    pub rot_y: f32,
}

impl Camera {
    fn _layout(state: &crate::RenderState) -> wgpu::VertexBufferLayout {
        todo!();
    }

    pub fn get_transform(&self, state: &crate::RenderState) -> Matrix4<f32> {
        let size = state.window.inner_size();
        let proj = na::Perspective3::new(size.width as f32 / size.height as f32, 45.0, 0.1, 100.0);

        let rot =
            Matrix4::new_rotation_wrt_point(na::Vector3::y() * (state.time % 360.0), self.target)
                .normalize()
                * 10.0;

        let view = Isometry3::look_at_rh(
            &rot.transform_point(&self.pos),
            &self.target,
            &na::Vector3::y(),
        );

        proj.as_matrix() * view.to_homogeneous()
    }
}
