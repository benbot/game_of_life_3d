use glam::{Mat4, Vec3};

#[derive(Debug)]
pub struct Camera {
    pub pos: glam::Vec3,
    pub target: glam::Vec3,
    pub rot_x: f32,
    pub rot_y: f32,
    pub buffer: wgpu::Buffer,
}

impl Camera {
    fn layout(_state: &crate::RenderState) -> wgpu::VertexBufferLayout {
        todo!();
    }

    pub fn get_transform(&self, state: &crate::RenderState) -> Mat4 {
        let size = state.window.inner_size();
        let look = Mat4::look_at_rh(self.pos, self.target, glam::Vec3::Y);
        let cam = Mat4::perspective_rh(45.0, size.width as f32 / size.height as f32, 0.1, 100.0);
        let rotation_x = glam::Quat::from_axis_angle(glam::Vec3::Y, self.rot_y);
        let rotation_y = glam::Quat::from_axis_angle(glam::Vec3::X, self.rot_x);

        let transform = Mat4::from_rotation_translation(rotation_x * rotation_y, Vec3::ZERO);

        cam * look * transform
    }
}
