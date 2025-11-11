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

    let mut window = Window::new("Planet Lab — Rocoso Bio-Lum", w, h, WindowOptions::default())
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
    let mut current_mode: u32 = 0; // 0=rocky,1=gas,2=alien,3=lava,4=ice
    let mut show_rings = true;
    let mut show_moons = true;

    let t0 = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t = t0.elapsed().as_secs_f32();

        fb.clear(0xFF0B0E13); // fondo oscuro

        // handle input (simple key controls)
    let prev_mode = current_mode;
    if window.is_key_down(Key::Key1) { current_mode = 0; }
    if window.is_key_down(Key::Key2) { current_mode = 1; }
    if window.is_key_down(Key::Key3) { current_mode = 2; }
    if window.is_key_down(Key::Key4) { current_mode = 3; }
    if window.is_key_down(Key::Key5) { current_mode = 4; }
    if window.is_key_down(Key::R) { show_rings = !show_rings; }
    if window.is_key_down(Key::L) { show_moons = !show_moons; }
        if window.is_key_down(Key::Left) { camera_angle -= 0.02; }
        if window.is_key_down(Key::Right) { camera_angle += 0.02; }
        if window.is_key_down(Key::Up) { camera_pitch = (camera_pitch + 0.01).clamp(-1.2, 1.2); }
        if window.is_key_down(Key::Down) { camera_pitch = (camera_pitch - 0.01).clamp(-1.2, 1.2); }
        if window.is_key_down(Key::Z) { camera_dist = (camera_dist - 0.1).max(1.0); }
        if window.is_key_down(Key::X) { camera_dist = (camera_dist + 0.1).min(50.0); }

        let planet_names = [
            "Rocoso Bio-Lum",
            "Gaseoso Turq-Lima",
            "Alien Verde-Purp",
            "Lava Incandescente",
            "Hielo Cristal",
        ];
        if prev_mode != current_mode {
            window.set_title(&format!("Planet Lab — {}", planet_names[current_mode as usize]));
        }

        // compute camera orbit around origin (single planet)
        let focus_pos = Vec3::ZERO;
        let ca = camera_angle; let cp = camera_pitch; let cd = camera_dist;
        let eye = focus_pos + Vec3::new(cd * ca.cos() * cp.cos(), cd * cp.sin(), cd * ca.sin() * cp.cos());
        let cam = Camera::look_at(eye, focus_pos, Vec3::Y);

        // single planet transform
        let rot = Mat4::from_rotation_y(t * 0.25);
        let model = Mat4::from_translation(focus_pos) * rot * Mat4::from_scale(Vec3::splat(1.0));
        let su = ShaderUniforms {
            time: t,
            mode: current_mode,
            light_dir: Vec3::new(0.3,0.6,0.7).normalize(),
            camera_pos: eye,
            ring_inner: 1.0,
            ring_outer: 1.7,
            ring_enabled: false,
        };
        draw_mesh(&mut fb, &sphere, model, &cam, su);

        // moon: small sphere orbiting the planet
        if show_moons {
            let orbit_r = 2.0;
            let speed = 1.0;
            let mx = focus_pos.x + orbit_r * (t*speed).cos();
            let mz = focus_pos.z + orbit_r * (t*speed).sin();
            let moon_model = Mat4::from_translation(Vec3::new(mx, 0.0, mz)) * Mat4::from_scale(Vec3::splat(0.25));
            let moon_su = ShaderUniforms { time: t, mode: 99, light_dir: Vec3::new(0.3,0.6,0.7).normalize(), camera_pos: eye, ring_inner:0.0, ring_outer:0.0, ring_enabled:false };
            draw_mesh(&mut fb, &sphere, moon_model, &cam, moon_su);
        }

        // ring only on gas giant
        if show_rings && current_mode == 1 {
            let ring_model1 = Mat4::from_translation(focus_pos) * Mat4::from_scale(Vec3::new(2.0, 0.01, 2.0));
            let ring_su1 = ShaderUniforms { time: t, mode: 1, light_dir: Vec3::new(0.3,0.6,0.7).normalize(), camera_pos: eye, ring_inner: 0.9, ring_outer: 1.6, ring_enabled: true };
            draw_mesh(&mut fb, &sphere, ring_model1, &cam, ring_su1);
        }

        // HUD: simple text top-left
    draw_text(&mut fb, 8, 8, &format!("Planet: {}", planet_names[current_mode as usize]), 0xFFFFFFFF);
        draw_text(&mut fb, 8, 24, &format!("Rings: {}", if show_rings {"ON"} else {"OFF"}), 0xFFB0FFC0);
        draw_text(&mut fb, 8, 40, &format!("Moon: {}", if show_moons {"ON"} else {"OFF"}), 0xFFFFE0B0);

        // blit a ventana
        window
            .update_with_buffer(&fb.color, fb.w, fb.h)
            .expect("update fail");
    }

    Ok(())
}

// ---------------- HUD text renderer (5x7 bitmap font) -----------------
fn draw_text(fb: &mut Framebuffer, x: i32, y: i32, text: &str, color: u32) {
    const FONT: [u8; 96*5] = generate_font(); // 96 ASCII chars (from 32 to 127), width 5
    for (i, ch) in text.chars().enumerate() {
        let c = ch as u32;
        if c < 32 || c > 127 { continue; }
        let idx = (c - 32) * 5;
        let gx = x + (i as i32) * 6;
        for col in 0..5 { 
            let bits = FONT[(idx + col) as usize];
            for row in 0..7 {
                if (bits >> row) & 1 == 1 {
                    let px = gx + col as i32;
                    let py = y + row as i32;
                    if let Some(pi) = fb.idx(px, py) { fb.color[pi] = color; }
                }
            }
        }
    }
}

const fn generate_font() -> [u8;96*5] {
    // Minimal font: each char 5 columns, bits lowest->row. Define only a subset, rest blanks.
    let mut data = [0u8;96*5];
    // Helper macro-like (manual): define A-Z, digits, space, colon
    // Example pattern for 'A': columns binary with rows bits.
    // We'll define a concise set for needed characters: letters, digits, space, dash, colon, P, L, A, N, E, T, R, I, G, S, O, C, B, U, M, D, F, V, H.
    // For brevity some shapes approximate.
    set_char(&mut data, ' ', [0,0,0,0,0]);
    set_char(&mut data, ':', [0,0b0001000,0,0b0001000,0]);
    set_char(&mut data, 'A', [0b0111110,0b0001001,0b0001001,0b0111110,0b0000000]);
    set_char(&mut data, 'B', [0b0111111,0b0100101,0b0100101,0b0011010,0]);
    set_char(&mut data, 'C', [0b0011110,0b0100001,0b0100001,0b0010010,0]);
    set_char(&mut data, 'D', [0b0111111,0b0100001,0b0100001,0b0011110,0]);
    set_char(&mut data, 'E', [0b0111111,0b0100101,0b0100101,0b0100001,0]);
    set_char(&mut data, 'G', [0b0011110,0b0100001,0b0100101,0b0011111,0]);
    set_char(&mut data, 'H', [0b0111111,0b0000100,0b0000100,0b0111111,0]);
    set_char(&mut data, 'I', [0b0100001,0b0111111,0b0100001,0,0]);
    set_char(&mut data, 'L', [0b0111111,0b0000001,0b0000001,0b0000001,0]);
    set_char(&mut data, 'M', [0b0111111,0b0010000,0b0001000,0b0010000,0b0111111]);
    set_char(&mut data, 'N', [0b0111111,0b0001000,0b0000100,0b0111111,0]);
    set_char(&mut data, 'O', [0b0011110,0b0100001,0b0100001,0b0011110,0]);
    set_char(&mut data, 'P', [0b0111111,0b0100100,0b0100100,0b0011000,0]);
    set_char(&mut data, 'R', [0b0111111,0b0101100,0b0100110,0b0011001,0]);
    set_char(&mut data, 'S', [0b0011001,0b0100101,0b0100101,0b0010011,0]);
    set_char(&mut data, 'T', [0b0100000,0b0111111,0b0100000,0,0]);
    set_char(&mut data, 'U', [0b0111110,0b0000001,0b0000001,0b0111110,0]);
    set_char(&mut data, 'V', [0b0111110,0b0000010,0b0000100,0b0111110,0]);
    set_char(&mut data, 'Y', [0b0110000,0b0001000,0b0000111,0b0001000,0]);
    set_char(&mut data, '0', [0b0111110,0b0100001,0b0100001,0b0111110,0]);
    set_char(&mut data, '1', [0b0000000,0b0111111,0b0000000,0,0]);
    set_char(&mut data, '2', [0b0110010,0b0101001,0b0100101,0b0100010,0]);
    set_char(&mut data, '3', [0b0100001,0b0100101,0b0100101,0b0011010,0]);
    set_char(&mut data, '4', [0b0001100,0b0001010,0b0111111,0b0001000,0]);
    set_char(&mut data, '5', [0b0101111,0b0100101,0b0100101,0b0011001,0]);
    set_char(&mut data, '-', [0b0001000,0b0001000,0b0001000,0b0001000,0]);
    data
}

const fn set_char(buf: &mut [u8;96*5], ch: char, cols: [u8;5]) {
    let c = ch as u32;
    if c < 32 || c > 127 { return; }
    let idx = (c - 32) * 5;
    buf[idx as usize] = cols[0];
    buf[idx as usize + 1] = cols[1];
    buf[idx as usize + 2] = cols[2];
    buf[idx as usize + 3] = cols[3];
    buf[idx as usize + 4] = cols[4];
}
