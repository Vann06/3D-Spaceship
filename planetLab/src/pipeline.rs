use glam::{Mat4, Vec2, Vec3, Vec4};
use crate::{mesh::Mesh, framebuffer::Framebuffer, triangle::{draw_triangle, Vtx2D}};
use crate::shader::ShaderUniforms;

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

pub fn draw_mesh(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    model: Mat4,
    cam: &Camera,
    u: ShaderUniforms
){
    let mvp = cam.proj * cam.view * model;

    let w = fb.w as f32; let h = fb.h as f32;
    let sx = w * 0.5; let sy = h * 0.5;

    for &[i0,i1,i2] in &mesh.indices {
        let p0 = mesh.positions[i0 as usize];
        let p1 = mesh.positions[i1 as usize];
        let p2 = mesh.positions[i2 as usize];

        let wp0 = model.transform_point3(p0);
        let wp1 = model.transform_point3(p1);
        let wp2 = model.transform_point3(p2);

        let cp0 = mvp * Vec4::new(p0.x, p0.y, p0.z, 1.0);
        let cp1 = mvp * Vec4::new(p1.x, p1.y, p1.z, 1.0);
        let cp2 = mvp * Vec4::new(p2.x, p2.y, p2.z, 1.0);

        if cp0.w.abs() < 1e-6 || cp1.w.abs() < 1e-6 || cp2.w.abs() < 1e-6 { continue; }

        let ndc0 = cp0.truncate() / cp0.w;
        let ndc1 = cp1.truncate() / cp1.w;
        let ndc2 = cp2.truncate() / cp2.w;

        // backface culling
        let a = ndc1 - ndc0;
        let b = ndc2 - ndc0;
        let cross = a.x*b.y - a.y*b.x;
        if cross >= 0.0 { continue; }

        let s0 = Vec2::new(ndc0.x * sx + sx, -ndc0.y * sy + sy);
        let s1 = Vec2::new(ndc1.x * sx + sx, -ndc1.y * sy + sy);
        let s2 = Vec2::new(ndc2.x * sx + sx, -ndc2.y * sy + sy);

        let v0 = Vtx2D { p:s0, z: ndc0.z, world: wp0 };
        let v1 = Vtx2D { p:s1, z: ndc1.z, world: wp1 };
        let v2 = Vtx2D { p:s2, z: ndc2.z, world: wp2 };

        draw_triangle(fb, v0, v1, v2, u);
    }
}
