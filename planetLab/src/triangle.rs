use glam::{Vec2, Vec3};
use crate::framebuffer::Framebuffer;
use crate::shader::{Fragment, ShaderUniforms, shade_fragment};

#[derive(Copy, Clone)]
pub struct Vtx2D {
    pub p: Vec2,       // posición en pantalla (pixeles)
    pub z: f32,        // profundidad
    pub world: Vec3,   // posición en mundo
}

#[inline] fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (c.x - a.x)*(b.y - a.y) - (c.y - a.y)*(b.x - a.x)
}

pub fn draw_triangle(
    fb: &mut Framebuffer,
    a: Vtx2D, b: Vtx2D, c: Vtx2D,
    u: ShaderUniforms
){
    let minx = (a.p.x.min(b.p.x).min(c.p.x).floor() as i32).clamp(0, fb.w as i32 - 1);
    let maxx = (a.p.x.max(b.p.x).max(c.p.x).ceil()  as i32).clamp(0, fb.w as i32 - 1);
    let miny = (a.p.y.min(b.p.y).min(c.p.y).floor() as i32).clamp(0, fb.h as i32 - 1);
    let maxy = (a.p.y.max(b.p.y).max(c.p.y).ceil()  as i32).clamp(0, fb.h as i32 - 1);

    let area = edge(a.p, b.p, c.p);
    if area.abs() < 1e-6 { return; }

    for y in miny..=maxy {
        for x in minx..=maxx {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(b.p, c.p, p);
            let w1 = edge(c.p, a.p, p);
            let w2 = edge(a.p, b.p, p);
            if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                let w0n = w0 / area; let w1n = w1 / area; let w2n = w2 / area;
                let z = a.z*w0n + b.z*w1n + c.z*w2n;
                let world = a.world*w0n + b.world*w1n + c.world*w2n;

                // fragment shader (dispatch to selected planet shader)
                let col = shade_fragment(Fragment{world_pos:world}, u);
                let argb = to_argb8(col);
                fb.put(x, y, z, argb);
            }
        }
    }
}

fn to_argb8(c: Vec3) -> u32 {
    let r = (c.x.clamp(0.0,1.0)*255.0) as u32;
    let g = (c.y.clamp(0.0,1.0)*255.0) as u32;
    let b = (c.z.clamp(0.0,1.0)*255.0) as u32;
    (0xFF << 24) | (r << 16) | (g << 8) | b
}
