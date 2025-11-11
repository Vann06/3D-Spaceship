mod framebuffer;
mod mesh;
mod obj_loader;
mod pipeline;
mod triangle;
mod shader;

use glam::{Mat4, Vec3};
use minifb::{Window, WindowOptions, Key};
use std::time::Instant;

use framebuffer::Framebuffer;
use obj_loader::load_obj;
use pipeline::Camera;
use shader::{Fragment, star_color, star_displacement, StarUniforms};

fn main() -> anyhow::Result<()> {
    let w = 800usize; let h = 800usize;
    let mut fb = Framebuffer::new(w, h);
    let mut window = Window::new("Star Lab — Procedural Star", w, h, WindowOptions::default())?;

    // load shared sphere
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sphere_path = manifest.join("planetLab").join("models").join("sphere.obj");
    println!("Loading sphere for star from {}", sphere_path.display());
    let sphere = load_obj(sphere_path.to_str().unwrap())?;

    // star parameters (could be made interactive later)
    let mut params = StarUniforms {
        time: 0.0,
        camera_pos: Vec3::ZERO,
        temp_kelvin: 6200.0,
        disp_amp: 0.18,
        disp_freq: 2.5,
        speed: 0.55,
        noise_mode: 3, // default: 1=Perlin,2=Simplex,3=Cellular (value+cell blend)
    };

    let t0 = Instant::now();

    while window.is_open() {
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) { params.noise_mode = 1; }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) { params.noise_mode = 2; }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) { params.noise_mode = 3; }
        // optional quick tweaks
        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::Yes) { params.disp_amp = (params.disp_amp + 0.005).min(0.6); }
        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::Yes) { params.disp_amp = (params.disp_amp - 0.005).max(0.0); }
        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::Yes) { params.disp_freq = (params.disp_freq + 0.02).min(10.0); }
        if window.is_key_pressed(Key::Left, minifb::KeyRepeat::Yes) { params.disp_freq = (params.disp_freq - 0.02).max(0.3); }
        if window.is_key_pressed(Key::W, minifb::KeyRepeat::Yes) { params.temp_kelvin = (params.temp_kelvin + 30.0).min(12000.0); }
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::Yes) { params.temp_kelvin = (params.temp_kelvin - 30.0).max(3000.0); }
        if window.is_key_pressed(Key::A, minifb::KeyRepeat::Yes) { params.speed = (params.speed - 0.01).max(0.05); }
        if window.is_key_pressed(Key::D, minifb::KeyRepeat::Yes) { params.speed = (params.speed + 0.01).min(2.5); }
        let t = t0.elapsed().as_secs_f32();
        params.time = t;

        fb.clear(0xFF000006); // very dark red background

        // simple slow orbital camera
        let angle = t * 0.15;
        let dist = 6.0;
        let eye = Vec3::new(dist * angle.cos(), 2.5, dist * angle.sin());
        params.camera_pos = eye;
        let cam = Camera::look_at(eye, Vec3::ZERO, Vec3::Y);

        // model transform (rotation for variation)
        let model = Mat4::from_rotation_y(t * 0.1);

        // We need custom draw to apply vertex displacement
        draw_mesh_star(&mut fb, &sphere, model, &cam, params);

        window.set_title(&format!("StarLab | Mode:{} (1=Perlin 2=Simplex 3=Cellular) Amp:{:.2} Freq:{:.2} Temp:{:.0}K Speed:{:.2}",
            params.noise_mode, params.disp_amp, params.disp_freq, params.temp_kelvin, params.speed));
        window.update_with_buffer(&fb.color, fb.w, fb.h)?;
    }
    Ok(())
}

// Custom mesh draw applying displacement + star fragment color
fn draw_mesh_star(
    fb: &mut Framebuffer,
    mesh: &mesh::Mesh,
    model: Mat4,
    cam: &Camera,
    su: StarUniforms
){
    use glam::{Vec2, Vec4};
    use triangle::Vtx2D;

    let mvp = cam.proj * cam.view * model;
    let w = fb.w as f32; let h = fb.h as f32;
    let sx = w * 0.5; let sy = h * 0.5;

    for &[i0,i1,i2] in &mesh.indices {
        let p0 = mesh.positions[i0 as usize];
        let p1 = mesh.positions[i1 as usize];
        let p2 = mesh.positions[i2 as usize];

        let n0 = p0.normalize();
        let n1 = p1.normalize();
        let n2 = p2.normalize();

    let d0 = star_displacement(n0, su);
    let d1 = star_displacement(n1, su);
    let d2 = star_displacement(n2, su);

        let p0d = p0 * (1.0 + d0);
        let p1d = p1 * (1.0 + d1);
        let p2d = p2 * (1.0 + d2);

        let wp0 = model.transform_point3(p0d);
        let wp1 = model.transform_point3(p1d);
        let wp2 = model.transform_point3(p2d);

        let cp0 = mvp * Vec4::new(p0d.x, p0d.y, p0d.z, 1.0);
        let cp1 = mvp * Vec4::new(p1d.x, p1d.y, p1d.z, 1.0);
        let cp2 = mvp * Vec4::new(p2d.x, p2d.y, p2d.z, 1.0);
        if cp0.w.abs() < 1e-6 || cp1.w.abs() < 1e-6 || cp2.w.abs() < 1e-6 { continue; }

        let ndc0 = cp0.truncate()/cp0.w;
        let ndc1 = cp1.truncate()/cp1.w;
        let ndc2 = cp2.truncate()/cp2.w;

        // disable backface culling for star (protuberances can flip)
        // (optional) keep faces

        let s0 = Vec2::new(ndc0.x * sx + sx, -ndc0.y * sy + sy);
        let s1 = Vec2::new(ndc1.x * sx + sx, -ndc1.y * sy + sy);
        let s2 = Vec2::new(ndc2.x * sx + sx, -ndc2.y * sy + sy);

        let v0 = Vtx2D { p:s0, z: ndc0.z, world: wp0 };
        let v1 = Vtx2D { p:s1, z: ndc1.z, world: wp1 };
        let v2 = Vtx2D { p:s2, z: ndc2.z, world: wp2 };

        draw_triangle_star(fb, v0, v1, v2, su);
    }
}

fn draw_triangle_star(
    fb: &mut Framebuffer,
    a: triangle::Vtx2D,
    b: triangle::Vtx2D,
    c: triangle::Vtx2D,
    su: StarUniforms
){
    use glam::Vec2;
    let minx = (a.p.x.min(b.p.x).min(c.p.x).floor() as i32).clamp(0, fb.w as i32 - 1);
    let maxx = (a.p.x.max(b.p.x).max(c.p.x).ceil()  as i32).clamp(0, fb.w as i32 - 1);
    let miny = (a.p.y.min(b.p.y).min(c.p.y).floor() as i32).clamp(0, fb.h as i32 - 1);
    let maxy = (a.p.y.max(b.p.y).max(c.p.y).ceil()  as i32).clamp(0, fb.h as i32 - 1);

    // signed area for barycentric
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
                let col = star_color(Fragment{world_pos:world}, su);
                let argb = to_argb8(col);
                fb.put(x, y, z, argb);
            }
        }
    }
}

#[inline] fn edge(a: glam::Vec2, b: glam::Vec2, c: glam::Vec2) -> f32 { (c.x - a.x)*(b.y - a.y) - (c.y - a.y)*(b.x - a.x) }
fn to_argb8(c: Vec3) -> u32 {
    let r = (c.x.clamp(0.0,1.0)*255.0) as u32;
    let g = (c.y.clamp(0.0,1.0)*255.0) as u32;
    let b = (c.z.clamp(0.0,1.0)*255.0) as u32;
    (0xFF << 24) | (r << 16) | (g << 8) | b
}
