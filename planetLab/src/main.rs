mod framebuffer;
mod mesh;
mod obj_loader;
mod pipeline;
mod triangle;
mod shader;

use glam::{Mat4, Vec3};
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

use framebuffer::Framebuffer;
use obj_loader::load_obj;
use pipeline::{Camera, draw_mesh};
use shader::{ShaderUniforms};

fn main() -> anyhow::Result<()> {
    let w = 800usize;
    let h = 800usize;
    let mut fb = Framebuffer::new(w, h);

    let mut window = Window::new("Planet Lab — Rocoso (solo shader)", w, h, WindowOptions::default())
        .expect("No se pudo crear la ventana");

    // carga esfera (ruta robusta usando CARGO_MANIFEST_DIR)
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sphere_path = manifest.join("planetLab").join("models").join("sphere.obj");
    println!("Loading sphere from {}", sphere_path.display());
    let sphere = load_obj(sphere_path.to_str().unwrap())?;

    // camera state (will be updated each frame)
    let mut camera_angle: f32 = 0.0;
    let mut camera_pitch: f32 = 0.15;
    let mut camera_dist: f32 = 8.0;
    let mut focused: usize = 1; // 0=left,1=center,2=right
    let mut show_rings = true;
    let mut show_moons = true;

    let t0 = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t = t0.elapsed().as_secs_f32();

        fb.clear(0xFF0B0E13); // fondo oscuro

        // handle input (simple key controls)
    if window.is_key_down(Key::Key1) { focused = 0; }
    if window.is_key_down(Key::Key2) { focused = 1; }
    if window.is_key_down(Key::Key3) { focused = 2; }
    if window.is_key_down(Key::R) { show_rings = !show_rings; }
    if window.is_key_down(Key::L) { show_moons = !show_moons; }
        if window.is_key_down(Key::Left) { camera_angle -= 0.02; }
        if window.is_key_down(Key::Right) { camera_angle += 0.02; }
        if window.is_key_down(Key::Up) { camera_pitch = (camera_pitch + 0.01).clamp(-1.2, 1.2); }
        if window.is_key_down(Key::Down) { camera_pitch = (camera_pitch - 0.01).clamp(-1.2, 1.2); }
        if window.is_key_down(Key::Z) { camera_dist = (camera_dist - 0.1).max(1.0); }
        if window.is_key_down(Key::X) { camera_dist = (camera_dist + 0.1).min(50.0); }

        // planet positions (three instances using same sphere mesh)
        let positions = [Vec3::new(-3.0, 0.0, 0.0), Vec3::ZERO, Vec3::new(3.0, 0.0, 0.0)];

        // compute camera orbit around focused planet
        let focus_pos = positions[focused];
        let ca = camera_angle; let cp = camera_pitch; let cd = camera_dist;
        let eye = focus_pos + Vec3::new(cd * ca.cos() * cp.cos(), cd * cp.sin(), cd * ca.sin() * cp.cos());
        let cam = Camera::look_at(eye, focus_pos, Vec3::Y);

        // for each planet instance, choose a shader mode and render
        for (i, &pos) in positions.iter().enumerate() {
            let rot = Mat4::from_rotation_y(t * (0.2 + 0.05 * i as f32));
            let model = Mat4::from_translation(pos) * rot * Mat4::from_scale(Vec3::splat(1.0));

            let mode = match i {
                0 => 0u32, // rocky
                1 => 1u32, // gas
                2 => 2u32, // scifi
                _ => 0u32,
            };

            let su = ShaderUniforms {
                time: t,
                mode,
                light_dir: Vec3::new(0.3,0.6,0.7).normalize(),
                camera_pos: eye,
                ring_inner: 1.2,
                ring_outer: 2.2,
                ring_enabled: false,
            };

            draw_mesh(&mut fb, &sphere, model, &cam, su);

            // moon: small sphere orbiting the planet
            if show_moons {
                let orbit_r = 1.8 + 0.3 * i as f32;
                let speed = 0.9 + 0.2 * i as f32;
                let mx = pos.x + orbit_r * (t*speed).cos();
                let mz = pos.z + orbit_r * (t*speed).sin();
                let moon_model = Mat4::from_translation(Vec3::new(mx, 0.0, mz)) * Mat4::from_scale(Vec3::splat(0.25));
                let moon_su = ShaderUniforms { time: t, mode: 99, light_dir: Vec3::new(0.3,0.6,0.7).normalize(), camera_pos: eye, ring_inner:0.0, ring_outer:0.0, ring_enabled:false };
                draw_mesh(&mut fb, &sphere, moon_model, &cam, moon_su);
            }

            // anillos: gas giant (i==1) y sci-fi (i==2)
            if show_rings && i == 1 {
                // solo el gigante gaseoso tiene anillo
                let ring_model1 = Mat4::from_translation(pos) * Mat4::from_scale(Vec3::new(2.0, 0.01, 2.0));
                let ring_su1 = ShaderUniforms { time: t, mode: 1, light_dir: Vec3::new(0.3,0.6,0.7).normalize(), camera_pos: eye, ring_inner: 0.9, ring_outer: 1.6, ring_enabled: true };
                draw_mesh(&mut fb, &sphere, ring_model1, &cam, ring_su1);
            }
        }

        // blit a ventana
        window
            .update_with_buffer(&fb.color, fb.w, fb.h)
            .expect("update fail");
    }

    Ok(())
}
