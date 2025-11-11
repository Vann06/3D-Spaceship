use glam::{Mat4, Vec3};

pub struct Camera {
    pub view: Mat4,
    pub proj: Mat4,
}

impl Camera {
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let view = Mat4::look_at_rh(eye, center, up);
        let proj = Mat4::perspective_rh_gl(60f32.to_radians(), 1.0, 0.1, 100.0);
        Self { view, proj }
    }
}

// draw_mesh removed in star-only build; StarLab uses a custom path.
