// triangle.rs

use crate::framebuffer::Framebuffer;
use crate::shader::{fragment_shader, Fragment, Uniforms};
use raylib::prelude::*;

fn edge_function(a: Vector2, b: Vector2, c: Vector2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

// World-aware triangle fill: uses screen-space for raster, but interpolates a separate world position for shading
pub fn triangle_filled_world(
    framebuffer: &mut Framebuffer,
    s1: Vector3,
    s2: Vector3,
    s3: Vector3, // screen-space vertices (x,y in pixels, z depth)
    w1: Vector3,
    w2: Vector3,
    w3: Vector3, // world-space vertices (used for shading)
    base_color: Vector3,
    uniforms: &Uniforms,
) {
    let p0 = Vector2::new(s1.x, s1.y);
    let p1 = Vector2::new(s2.x, s2.y);
    let p2 = Vector2::new(s3.x, s3.y);

    let signed_area = edge_function(p0, p1, p2);
    if signed_area >= 0.0 {
        return;
    }
    let area = signed_area;
    if area.abs() < 1e-5 {
        return;
    }

    let min_x = p0.x.min(p1.x).min(p2.x).floor().max(0.0) as i32;
    let max_x =
        p0.x.max(p1.x)
            .max(p2.x)
            .ceil()
            .min(framebuffer.width as f32 - 1.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor().max(0.0) as i32;
    let max_y =
        p0.y.max(p1.y)
            .max(p2.y)
            .ceil()
            .min(framebuffer.height as f32 - 1.0) as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
            let e0 = edge_function(p1, p2, p);
            let e1 = edge_function(p2, p0, p);
            let e2 = edge_function(p0, p1, p);
            if e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0 {
                let w0n = e0 / area;
                let w1n = e1 / area;
                let w2n = e2 / area;
                let depth = s1.z * w0n + s2.z * w1n + s3.z * w2n;
                if framebuffer.test_and_set_depth(x as u32, y as u32, depth) {
                    let world = Vector3::new(
                        w1.x * w0n + w2.x * w1n + w3.x * w2n,
                        w1.y * w0n + w2.y * w1n + w3.y * w2n,
                        w1.z * w0n + w2.z * w1n + w3.z * w2n,
                    );
                    let frag = Fragment {
                        world_position: world,
                        color: base_color,
                    };
                    let rgb = fragment_shader(&frag, uniforms);
                    let r = rgb.x.clamp(0.0, 1.0);
                    let g = rgb.y.clamp(0.0, 1.0);
                    let b = rgb.z.clamp(0.0, 1.0);
                    let color =
                        Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255);
                    framebuffer.set_pixel_color(x as u32, y as u32, color);
                }
            }
        }
    }
}

// No-depth version to force visibility (e.g., HUD / ship overlay)
pub fn triangle_filled_world_no_depth(
    framebuffer: &mut Framebuffer,
    s1: Vector3,
    s2: Vector3,
    s3: Vector3,
    w1: Vector3,
    w2: Vector3,
    w3: Vector3,
    base_color: Vector3,
    uniforms: &Uniforms,
) {
    let p0 = Vector2::new(s1.x, s1.y);
    let p1 = Vector2::new(s2.x, s2.y);
    let p2 = Vector2::new(s3.x, s3.y);
    let signed_area = edge_function(p0, p1, p2);
    if signed_area >= 0.0 {
        return;
    }
    let area = signed_area;
    if area.abs() < 1e-5 {
        return;
    }
    let min_x = p0.x.min(p1.x).min(p2.x).floor().max(0.0) as i32;
    let max_x =
        p0.x.max(p1.x)
            .max(p2.x)
            .ceil()
            .min(framebuffer.width as f32 - 1.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor().max(0.0) as i32;
    let max_y =
        p0.y.max(p1.y)
            .max(p2.y)
            .ceil()
            .min(framebuffer.height as f32 - 1.0) as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
            let e0 = edge_function(p1, p2, p);
            let e1 = edge_function(p2, p0, p);
            let e2 = edge_function(p0, p1, p);
            if e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0 {
                let w0n = e0 / area;
                let w1n = e1 / area;
                let w2n = e2 / area;
                let world = Vector3::new(
                    w1.x * w0n + w2.x * w1n + w3.x * w2n,
                    w1.y * w0n + w2.y * w1n + w3.y * w2n,
                    w1.z * w0n + w2.z * w1n + w3.z * w2n,
                );
                let frag = Fragment {
                    world_position: world,
                    color: base_color,
                };
                let rgb = fragment_shader(&frag, uniforms);
                let r = rgb.x.clamp(0.0, 1.0);
                let g = rgb.y.clamp(0.0, 1.0);
                let b = rgb.z.clamp(0.0, 1.0);
                let color =
                    Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255);
                framebuffer.set_pixel_color(x as u32, y as u32, color);
            }
        }
    }
}
