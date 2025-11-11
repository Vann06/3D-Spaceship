use glam::{Vec2, Vec3};
// This module kept only for potential reuse; planet shade removed.

#[derive(Copy, Clone)]
pub struct Vtx2D {
    pub p: Vec2,       // posición en pantalla (pixeles)
    pub z: f32,        // profundidad
    pub world: Vec3,   // posición en mundo
}

// edge and to_argb8 remain for optional reuse; suppress dead_code warnings in star-only build
#[allow(dead_code)]
#[inline] fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 { (c.x - a.x)*(b.y - a.y) - (c.y - a.y)*(b.x - a.x) }

// draw_triangle removed (star path uses custom implementation in star_main.rs)

#[allow(dead_code)]
fn to_argb8(c: Vec3) -> u32 {
    let r = (c.x.clamp(0.0,1.0)*255.0) as u32;
    let g = (c.y.clamp(0.0,1.0)*255.0) as u32;
    let b = (c.z.clamp(0.0,1.0)*255.0) as u32;
    (0xFF << 24) | (r << 16) | (g << 8) | b
}
