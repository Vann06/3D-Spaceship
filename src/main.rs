mod framebuffer;
mod line;
mod obj_loader;
mod shader;
mod triangle;

use framebuffer::Framebuffer;
use obj_loader::Mesh;
use raylib::prelude::*;
use std::{f32::consts::PI, fs, path::Path};

const SHIP_CLEARANCE: f32 = 0.045;
const STAR_CLEARANCE: f32 = 0.25;
const MOON_CLEARANCE: f32 = 0.035;

#[derive(Clone)]
struct Body {
    name: &'static str,
    radius: f32,
    color: Vector3,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    mode: u32,
    has_ring: bool,
    ring_inner_scale: f32,
    ring_outer_scale: f32,
    ring_tilt_deg: f32,
}

#[derive(Clone)]
struct Moon {
    parent_index: usize,
    radius: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    phase: f32,
    mode: u32,
}

fn normalize_vec(v: Vector3) -> Vector3 {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt().max(1e-6);
    Vector3::new(v.x / len, v.y / len, v.z / len)
}

fn dot(a: Vector3, b: Vector3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn load_snoopy_color<P: AsRef<Path>>(path: P) -> Vector3 {
    if let Ok(text) = fs::read_to_string(path) {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Kd ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() == 4 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        return Vector3::new(
                            r.clamp(0.0, 1.0),
                            g.clamp(0.0, 1.0),
                            b.clamp(0.0, 1.0),
                        );
                    }
                }
            }
        }
    }
    Vector3::new(1.0, 1.0, 1.0)
}

fn generate_uv_sphere(stacks: u32, slices: u32, radius: f32) -> Mesh {
    let mut vertices: Vec<Vector3> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();
    for i in 0..=stacks {
        let v = i as f32 / stacks as f32;
        let phi = v * PI;
        for j in 0..=slices {
            let u = j as f32 / slices as f32;
            let theta = u * 2.0 * PI;
            let x = theta.cos() * phi.sin();
            let y = phi.cos();
            let z = theta.sin() * phi.sin();
            vertices.push(Vector3::new(x * radius, y * radius, z * radius));
        }
    }
    let row = (slices + 1) as usize;
    for i in 0..stacks as usize {
        for j in 0..slices as usize {
            let i0 = i * row + j;
            let i1 = i * row + j + 1;
            let i2 = (i + 1) * row + j;
            let i3 = (i + 1) * row + j + 1;
            faces.push([i0, i2, i1]);
            faces.push([i1, i2, i3]);
        }
    }
    Mesh { vertices, faces }
}

fn rotate_xyz(vertex: Vector3, rotation_deg: Vector3) -> Vector3 {
    let (sx, cx) = (
        rotation_deg.x.to_radians().sin(),
        rotation_deg.x.to_radians().cos(),
    );
    let (sy, cy) = (
        rotation_deg.y.to_radians().sin(),
        rotation_deg.y.to_radians().cos(),
    );
    let (sz, cz) = (
        rotation_deg.z.to_radians().sin(),
        rotation_deg.z.to_radians().cos(),
    );
    let mut v = vertex;
    let ry = v.y * cx - v.z * sx;
    let rz = v.y * sx + v.z * cx;
    v.y = ry;
    v.z = rz;
    let rx = v.x * cy + v.z * sy;
    let rz2 = -v.x * sy + v.z * cy;
    v.x = rx;
    v.z = rz2;
    let rx2 = v.x * cz - v.y * sz;
    let ry2 = v.x * sz + v.y * cz;
    v.x = rx2;
    v.y = ry2;
    v
}

fn world_to_screen(
    world: Vector3,
    screen_center: Vector2,
    cam_pos_world: Vector3,
    pixels_per_unit: f32,
    camera_distance: f32,
    cam_yaw_deg: f32,
    cam_pitch_deg: f32,
) -> Vector3 {
    let rel = world - cam_pos_world;
    let yaw = cam_yaw_deg.to_radians();
    let pitch = cam_pitch_deg.to_radians();
    let (sin_y, cos_y) = yaw.sin_cos();
    let (sin_p, cos_p) = pitch.sin_cos();
    let x_prime = rel.x * cos_y - rel.z * sin_y;
    let z_prime = rel.x * sin_y + rel.z * cos_y;
    let y_prime = rel.y;
    let cam_y = y_prime * cos_p - z_prime * sin_p;
    let cam_z = y_prime * sin_p + z_prime * cos_p;
    let cam_x = x_prime;
    let persp = camera_distance / (camera_distance + cam_z.max(0.01));
    let sx = cam_x * pixels_per_unit * persp + screen_center.x;
    let sy = cam_y * pixels_per_unit * persp + screen_center.y;
    Vector3::new(sx, sy, cam_z)
}

fn project_point(
    world: Vector3,
    bird_eye: bool,
    screen_center: Vector2,
    cam_pos_world: Vector3,
    pixels_per_unit: f32,
    camera_distance: f32,
    cam_yaw: f32,
    cam_pitch: f32,
) -> Vector3 {
    if bird_eye {
        world_to_screen_top(world, screen_center, cam_pos_world, pixels_per_unit, camera_distance)
    } else {
        world_to_screen(
            world,
            screen_center,
            cam_pos_world,
            pixels_per_unit,
            camera_distance,
            cam_yaw,
            cam_pitch,
        )
    }
}

fn push_out_from_sphere(pos: &mut Vector3, vel: &mut Vector3, center: Vector3, radius: f32, clearance: f32) {
    let to_center = *pos - center;
    let dist = (to_center.x * to_center.x + to_center.y * to_center.y + to_center.z * to_center.z).sqrt();
    let min_dist = radius + clearance;
    if dist < min_dist {
        let normal = if dist < 1e-4 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(to_center.x / dist, to_center.y / dist, to_center.z / dist)
        };
        *pos = center + normal * min_dist;
        let vn = dot(*vel, normal);
        if vn < 0.0 {
            *vel -= normal * vn;
        }
    }
}

fn world_to_screen_top(
    world: Vector3,
    screen_center: Vector2,
    cam_pos_world: Vector3,
    pixels_per_unit: f32,
    camera_distance: f32,
) -> Vector3 {
    let wx = world.x - cam_pos_world.x;
    let wz = world.z - cam_pos_world.z;
    let wy = world.y - cam_pos_world.y;
    let persp = 1.0 / (1.0 + (wy / camera_distance));
    let sx = wx * pixels_per_unit * persp + screen_center.x;
    let sy = wz * pixels_per_unit * persp + screen_center.y;
    Vector3::new(sx, sy, wy)
}

fn body_position(body: &Body, time: f32) -> Vector3 {
    let theta = time * body.orbit_speed;
    Vector3::new(
        body.orbit_radius * theta.cos(),
        0.0,
        body.orbit_radius * theta.sin(),
    )
}

fn moon_position(moon: &Moon, parent: &Body, time: f32) -> Vector3 {
    let parent_world = body_position(parent, time);
    parent_world
        + Vector3::new(
            moon.orbit_radius * moon.phase.cos(),
            0.0,
            moon.orbit_radius * moon.phase.sin(),
        )
}

fn look_angles(from: Vector3, to: Vector3) -> (f32, f32) {
    let dir = to - from;
    let horiz = (dir.x * dir.x + dir.z * dir.z).sqrt().max(1e-5);
    let yaw = dir.x.atan2(dir.z).to_degrees();
    let pitch = dir.y.atan2(horiz).to_degrees();
    (yaw, pitch)
}

fn draw_planet_ring(
    fb: &mut Framebuffer,
    uniforms: &mut shader::Uniforms,
    center: Vector3,
    inner_radius: f32,
    outer_radius: f32,
    tilt_deg: f32,
    base_color: Vector3,
    segments: usize,
    bird_eye: bool,
    screen_center: Vector2,
    cam_pos_world: Vector3,
    pixels_per_unit: f32,
    camera_distance: f32,
    cam_yaw: f32,
    cam_pitch: f32,
) {
    if inner_radius <= 0.0 || outer_radius <= inner_radius {
        return;
    }
    let seg_count = segments.max(16);
    let tilt = tilt_deg.to_radians();
    let sin_tilt = tilt.sin();
    let cos_tilt = tilt.cos();

    let mut inner_points: Vec<Vector3> = Vec::with_capacity(seg_count + 1);
    let mut outer_points: Vec<Vector3> = Vec::with_capacity(seg_count + 1);
    for i in 0..=seg_count {
        let angle = (i as f32 / seg_count as f32) * 2.0 * PI;
        let (sin_a, cos_a) = angle.sin_cos();
        let make_point = |radius: f32| -> Vector3 {
            let local_x = cos_a * radius;
            let local_z = sin_a * radius;
            let tilt_y = -local_z * sin_tilt;
            let tilt_z = local_z * cos_tilt;
            Vector3::new(center.x + local_x, center.y + tilt_y, center.z + tilt_z)
        };
        inner_points.push(make_point(inner_radius));
        outer_points.push(make_point(outer_radius));
    }

    let prev_pattern = uniforms.pattern;
    let prev_ring_flag = uniforms.ring_enabled;
    let prev_inner = uniforms.ring_inner;
    let prev_outer = uniforms.ring_outer;
    let prev_center = uniforms.ring_center;
    uniforms.pattern = 300;
    uniforms.ring_enabled = true;
    uniforms.ring_inner = inner_radius;
    uniforms.ring_outer = outer_radius;
    uniforms.ring_center = center;

    for i in 0..seg_count {
        let p_inner0 = inner_points[i];
        let p_inner1 = inner_points[i + 1];
        let p_outer0 = outer_points[i];
        let p_outer1 = outer_points[i + 1];
        let tris = [
            (p_outer0, p_outer1, p_inner0),
            (p_inner0, p_outer1, p_inner1),
        ];
        for (a, b, c) in tris {
            let sa = project_point(
                a,
                bird_eye,
                screen_center,
                cam_pos_world,
                pixels_per_unit,
                camera_distance,
                cam_yaw,
                cam_pitch,
            );
            let sb = project_point(
                b,
                bird_eye,
                screen_center,
                cam_pos_world,
                pixels_per_unit,
                camera_distance,
                cam_yaw,
                cam_pitch,
            );
            let sc = project_point(
                c,
                bird_eye,
                screen_center,
                cam_pos_world,
                pixels_per_unit,
                camera_distance,
                cam_yaw,
                cam_pitch,
            );
            crate::triangle::triangle_filled_world(
                fb, sa, sb, sc, a, b, c, base_color, uniforms,
            );
        }
    }

    uniforms.pattern = prev_pattern;
    uniforms.ring_enabled = prev_ring_flag;
    uniforms.ring_inner = prev_inner;
    uniforms.ring_outer = prev_outer;
    uniforms.ring_center = prev_center;
}

fn main() {
    let width = 1000;
    let height = 800;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Snoopy Solar System — Software Renderer")
        .build();
    rl.set_target_fps(75);
    unsafe {
        raylib::ffi::SetTraceLogLevel(4);
    }

    let mut fb = Framebuffer::new(width as u32, height as u32);
    fb.set_background_color(Color::new(5, 5, 20, 255));

    let mut high_quality = true;
    let mut sphere = generate_uv_sphere(16, 22, 1.0);

    let star = Body {
        name: "Sol",
        radius: 0.75,
        color: Vector3::new(1.0, 0.85, 0.3),
        orbit_radius: 0.0,
        orbit_speed: 0.0,
        rotation_speed: 5.0,
        mode: 0,
        has_ring: false,
        ring_inner_scale: 0.0,
        ring_outer_scale: 0.0,
        ring_tilt_deg: 0.0,
    };
    let planets = vec![
        Body {
            name: "Asteroide Prisma",
            radius: 0.16,
            color: Vector3::new(0.65, 0.55, 0.35),
            orbit_radius: 1.9,
            orbit_speed: 1.25,
            rotation_speed: 32.0,
            mode: 0,
            has_ring: false,
            ring_inner_scale: 0.0,
            ring_outer_scale: 0.0,
            ring_tilt_deg: 0.0,
        },
        Body {
            name: "Selva Luminosa",
            radius: 0.24,
            color: Vector3::new(0.18, 0.85, 0.4),
            orbit_radius: 3.1,
            orbit_speed: 1.02,
            rotation_speed: 24.0,
            mode: 0,
            has_ring: false,
            ring_inner_scale: 0.0,
            ring_outer_scale: 0.0,
            ring_tilt_deg: 0.0,
        },
        Body {
            name: "Glacia Prisma",
            radius: 0.26,
            color: Vector3::new(0.55, 0.9, 1.0),
            orbit_radius: 4.3,
            orbit_speed: 0.84,
            rotation_speed: 26.0,
            mode: 4,
            has_ring: false,
            ring_inner_scale: 0.0,
            ring_outer_scale: 0.0,
            ring_tilt_deg: 0.0,
        },
        Body {
            name: "Cristal Helicoide",
            radius: 0.30,
            color: Vector3::new(0.48, 0.92, 1.0),
            orbit_radius: 5.7,
            orbit_speed: 0.62,
            rotation_speed: 24.0,
            mode: 4,
            has_ring: true,
            ring_inner_scale: 1.55,
            ring_outer_scale: 3.1,
            ring_tilt_deg: 18.0,
        },
        Body {
            name: "Tormenta Ember",
            radius: 0.34,
            color: Vector3::new(1.0, 0.45, 0.15),
            orbit_radius: 6.9,
            orbit_speed: 0.55,
            rotation_speed: 30.0,
            mode: 3,
            has_ring: false,
            ring_inner_scale: 0.0,
            ring_outer_scale: 0.0,
            ring_tilt_deg: 0.0,
        },
        Body {
            name: "Mar de Neón",
            radius: 0.36,
            color: Vector3::new(0.2, 0.78, 1.0),
            orbit_radius: 8.3,
            orbit_speed: 0.48,
            rotation_speed: 18.0,
            mode: 1,
            has_ring: true,
            ring_inner_scale: 1.35,
            ring_outer_scale: 2.8,
            ring_tilt_deg: 11.0,
        },
        Body {
            name: "Nebulosa Violeta",
            radius: 0.33,
            color: Vector3::new(0.92, 0.35, 1.0),
            orbit_radius: 9.6,
            orbit_speed: 0.41,
            rotation_speed: 18.0,
            mode: 2,
            has_ring: true,
            ring_inner_scale: 1.1,
            ring_outer_scale: 2.4,
            ring_tilt_deg: 27.0,
        },
    ];

    let mut pixels_per_unit: f32 = 60.0;
    let mut ship_world_pos = Vector3::new(0.0, 0.25, -7.0);
    let mut ship_velocity = Vector3::new(0.0, 0.0, 0.0);
    let screen_center = Vector2::new((width / 2) as f32, (height / 2) as f32);
    let camera_distance = 3.0;
    let mut bird_eye = false;
    let bird_eye_height = 7.0f32;
    let mut cam_yaw = 0.0f32;
    let mut cam_pitch = -5.0f32;
    let follow_distance = 1.8f32;

    let t0 = std::time::Instant::now();
    let mut warp_flash = 0.0f32;

    let models_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("models");
    let snoopy_obj = models_dir.join("Snoopy.obj");
    let snoopy_mtl = models_dir.join("Snoopy.mtl");
    let snoopy_mesh = Mesh::load_obj(&snoopy_obj).ok();
    let snoopy_scale: f32 = 0.35;
    let _snoopy_material_color = load_snoopy_color(snoopy_mtl);
    let snoopy_color = Vector3::new(1.0, 1.0, 1.0);

    let mut moons: Vec<Moon> = vec![
        Moon {
            parent_index: 1,
            radius: 0.05,
            orbit_radius: 0.38,
            orbit_speed: 1.6,
            phase: 0.0,
            mode: 0,
        },
        Moon {
            parent_index: 3,
            radius: 0.07,
            orbit_radius: 0.45,
            orbit_speed: 1.4,
            phase: 1.2,
            mode: 4,
        },
        Moon {
            parent_index: 4,
            radius: 0.06,
            orbit_radius: 0.52,
            orbit_speed: 1.1,
            phase: 2.4,
            mode: 3,
        },
        Moon {
            parent_index: 5,
            radius: 0.05,
            orbit_radius: 0.6,
            orbit_speed: 1.3,
            phase: 0.7,
            mode: 1,
        },
    ];

    let mut stars: Vec<(i32, i32, u8)> = Vec::new();
    {
        let mut seed = 1337u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 16) as u16
        };
        for _ in 0..800 {
            let x = (rnd() as i32 % width).abs();
            let y = (rnd() as i32 % height).abs();
            let b = (rnd() % 155 + 100) as u8;
            stars.push((x, y, b));
        }
    }

    const PLANET_WARP_KEYS: [KeyboardKey; 7] = [
        KeyboardKey::KEY_TWO,
        KeyboardKey::KEY_THREE,
        KeyboardKey::KEY_FOUR,
        KeyboardKey::KEY_FIVE,
        KeyboardKey::KEY_SIX,
        KeyboardKey::KEY_SEVEN,
        KeyboardKey::KEY_EIGHT,
    ];
    const MOON_WARP_KEYS: [KeyboardKey; 4] = [
        KeyboardKey::KEY_F1,
        KeyboardKey::KEY_F2,
        KeyboardKey::KEY_F3,
        KeyboardKey::KEY_F4,
    ];

    while !rl.window_should_close() {
        let t = t0.elapsed().as_secs_f32();
        let dt = rl.get_frame_time().max(1.0 / 240.0);
        warp_flash = (warp_flash - dt * 1.5).max(0.0);

        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            bird_eye = !bird_eye;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            high_quality = !high_quality;
            sphere = if high_quality {
                generate_uv_sphere(28, 40, 1.0)
            } else {
                generate_uv_sphere(16, 22, 1.0)
            };
        }

        let inspect_mode = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

        for m in &mut moons {
            m.phase = (m.phase + m.orbit_speed * dt) % (2.0 * PI);
        }

        let rot_speed = 65.0 * dt;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            cam_yaw -= rot_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            cam_yaw += rot_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            cam_pitch = (cam_pitch + rot_speed).clamp(-70.0, 70.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            cam_pitch = (cam_pitch - rot_speed).clamp(-70.0, 70.0);
        }

        let yaw_rad = cam_yaw.to_radians();
        let pitch_rad = cam_pitch.to_radians();
        let forward = Vector3::new(
            yaw_rad.sin() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        );
        let right = normalize_vec(Vector3::new(forward.z, 0.0, -forward.x));
        let up = Vector3::new(0.0, 1.0, 0.0);

        let mut input_dir = Vector3::new(0.0, 0.0, 0.0);
        if rl.is_key_down(KeyboardKey::KEY_W) {
            input_dir += forward;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            input_dir -= forward;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            input_dir -= right;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            input_dir += right;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            input_dir += up;
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            input_dir -= up;
        }

        let input_len =
            (input_dir.x * input_dir.x + input_dir.y * input_dir.y + input_dir.z * input_dir.z)
                .sqrt();
        let cruise_speed = if bird_eye { 4.5 } else { 5.5 };
        let desired_velocity = if input_len > 1e-3 {
            let normalized = Vector3::new(
                input_dir.x / input_len,
                input_dir.y / input_len,
                input_dir.z / input_len,
            );
            normalized * cruise_speed
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };
        let accel = (6.0 * dt).min(1.0);
        ship_velocity = ship_velocity + (desired_velocity - ship_velocity) * accel;
        let speed_len = (ship_velocity.x * ship_velocity.x
            + ship_velocity.y * ship_velocity.y
            + ship_velocity.z * ship_velocity.z)
            .sqrt();
        if speed_len > cruise_speed {
            let scale = cruise_speed / speed_len;
            ship_velocity = ship_velocity * scale;
        }
        ship_world_pos += ship_velocity * dt;
        push_out_from_sphere(
            &mut ship_world_pos,
            &mut ship_velocity,
            Vector3::new(0.0, 0.0, 0.0),
            star.radius,
            STAR_CLEARANCE,
        );
        for body in &planets {
            let planet_pos = body_position(body, t);
            push_out_from_sphere(
                &mut ship_world_pos,
                &mut ship_velocity,
                planet_pos,
                body.radius,
                SHIP_CLEARANCE,
            );
        }
        for moon in &moons {
            let parent = &planets[moon.parent_index];
            let moon_pos = moon_position(moon, parent, t);
            push_out_from_sphere(
                &mut ship_world_pos,
                &mut ship_velocity,
                moon_pos,
                moon.radius,
                MOON_CLEARANCE,
            );
        }

        if rl.is_key_down(KeyboardKey::KEY_Z) {
            pixels_per_unit = (pixels_per_unit * 1.02 + 0.3).min(260.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_X) {
            pixels_per_unit = (pixels_per_unit * 0.98).max(14.0);
        }
        if inspect_mode {
            pixels_per_unit = (pixels_per_unit * 1.045 + 0.5).min(520.0);
        }

        let forward_speed = dot(ship_velocity, forward);
        let lateral_speed = dot(ship_velocity, right);
        let ship_pitch_angle = (-forward_speed * 18.0).clamp(-18.0, 15.0);
        let ship_bank_angle = (-lateral_speed * 28.0).clamp(-24.0, 24.0);

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            ship_world_pos = Vector3::new(0.0, star.radius + 0.9, -(star.radius + 2.6));
            ship_velocity = Vector3::new(0.0, 0.0, 0.0);
            let (yaw, pitch) = look_angles(ship_world_pos, Vector3::new(0.0, 0.0, 0.0));
            cam_yaw = yaw;
            cam_pitch = pitch.clamp(-70.0, 70.0);
            rl.set_window_title(&thread, &format!("Snoopy Solar System — {}", star.name));
        }

        for (idx, key) in PLANET_WARP_KEYS.iter().enumerate() {
            if rl.is_key_pressed(*key) {
                if let Some(body) = planets.get(idx) {
                    let target = body_position(body, t);
                    let outward = if target.x.abs() < 1e-3 && target.z.abs() < 1e-3 {
                        Vector3::new(0.0, 0.0, -1.0)
                    } else {
                        normalize_vec(Vector3::new(target.x, 0.0, target.z))
                    };
                    ship_world_pos = target
                        + outward * (body.radius + 1.15)
                        + Vector3::new(0.0, body.radius * 0.22 + 0.25, 0.0);
                    ship_velocity = Vector3::new(0.0, 0.0, 0.0);
                    let (yaw, pitch) = look_angles(ship_world_pos, target);
                    cam_yaw = yaw;
                    cam_pitch = pitch.clamp(-70.0, 70.0);
                    rl.set_window_title(&thread, &format!("Snoopy Solar System — {}", body.name));
                    warp_flash = 1.0;
                }
            }
        }

        for (idx, key) in MOON_WARP_KEYS.iter().enumerate() {
            if rl.is_key_pressed(*key) {
                if let Some(moon) = moons.get(idx) {
                    if let Some(parent) = planets.get(moon.parent_index) {
                        let theta = t * parent.orbit_speed;
                        let parent_world = Vector3::new(
                            parent.orbit_radius * theta.cos(),
                            0.0,
                            parent.orbit_radius * theta.sin(),
                        );
                        let target = parent_world
                            + Vector3::new(
                                moon.orbit_radius * moon.phase.cos(),
                                0.0,
                                moon.orbit_radius * moon.phase.sin(),
                            );
                        let outward = normalize_vec(target - parent_world);
                        ship_world_pos = target
                            + outward * (moon.radius + 0.18)
                            + Vector3::new(0.0, 0.12, 0.0);
                        ship_velocity = Vector3::new(0.0, 0.0, 0.0);
                        let (yaw, pitch) = look_angles(ship_world_pos, target);
                        cam_yaw = yaw;
                        cam_pitch = pitch.clamp(-70.0, 70.0);
                        rl.set_window_title(&thread, &format!("Snoopy Solar System — Luna {}", idx + 1));
                        warp_flash = 1.0;
                    }
                }
            }
        }

        let mut cam_pos_world = if bird_eye {
            Vector3::new(ship_world_pos.x, bird_eye_height, ship_world_pos.z)
        } else {
            let mut trailing = (follow_distance + forward_speed * 0.12).clamp(1.0, 3.5);
            if inspect_mode {
                trailing = trailing.min(0.55);
            }
            let base_lift = 0.4 + ship_pitch_angle.abs() * 0.01;
            let lift = (base_lift + if inspect_mode { -0.15 } else { 0.0 }).max(0.05);
            ship_world_pos - forward * trailing
                + Vector3::new(0.0, lift, 0.0)
                + right * (-lateral_speed * 0.05)
        };
        if inspect_mode && !bird_eye {
            cam_pos_world += forward * 0.35;
        }

        fb.clear();
        for (x, y, b) in &stars {
            fb.set_current_color(Color::new(*b, *b, *b, 255));
            fb.set_pixel(*x as u32, *y as u32);
        }
        fb.set_current_color(Color::WHITE);

        let perf_mode = if inspect_mode { false } else { !high_quality };
        let mut uniforms = shader::Uniforms {
            time: rl.get_time() as f32,
            pattern: 5,
            mode: 0,
            light_dir: Vector3::new(0.6, 0.7, 0.2),
            camera_pos: cam_pos_world,
            ring_inner: 0.0,
            ring_outer: 0.0,
            ring_enabled: false,
            ring_center: Vector3::new(0.0, 0.0, 0.0),
            performance_mode: perf_mode,
        };

        let star_rot = Vector3::new(0.0, t * star.rotation_speed, 0.0);
        uniforms.pattern = 100;
        for f in &sphere.faces {
            let w1 = rotate_xyz(sphere.vertices[f[0]] * star.radius, star_rot);
            let w2 = rotate_xyz(sphere.vertices[f[1]] * star.radius, star_rot);
            let w3 = rotate_xyz(sphere.vertices[f[2]] * star.radius, star_rot);
            let (s1, s2, s3) = if bird_eye {
                (
                    world_to_screen_top(
                        w1,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                    ),
                    world_to_screen_top(
                        w2,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                    ),
                    world_to_screen_top(
                        w3,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                    ),
                )
            } else {
                (
                    world_to_screen(
                        w1,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                        cam_yaw,
                        cam_pitch,
                    ),
                    world_to_screen(
                        w2,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                        cam_yaw,
                        cam_pitch,
                    ),
                    world_to_screen(
                        w3,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                        cam_yaw,
                        cam_pitch,
                    ),
                )
            };
            crate::triangle::triangle_filled_world(
                &mut fb, s1, s2, s3, w1, w2, w3, star.color, &uniforms,
            );
        }

        for p in planets.iter() {
            let theta = t * p.orbit_speed;
            let pos_world = Vector3::new(
                p.orbit_radius * theta.cos(),
                0.0,
                p.orbit_radius * theta.sin(),
            );
            let rot = Vector3::new(0.0, t * p.rotation_speed, 0.0);
            uniforms.pattern = 200;
            uniforms.mode = p.mode;
            uniforms.ring_enabled = false;
            for f in &sphere.faces {
                let w1 = rotate_xyz(sphere.vertices[f[0]] * p.radius, rot) + pos_world;
                let w2 = rotate_xyz(sphere.vertices[f[1]] * p.radius, rot) + pos_world;
                let w3 = rotate_xyz(sphere.vertices[f[2]] * p.radius, rot) + pos_world;
                let (s1, s2, s3) = if bird_eye {
                    (
                        world_to_screen_top(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                    )
                } else {
                    (
                        world_to_screen(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                    )
                };
                crate::triangle::triangle_filled_world(
                    &mut fb, s1, s2, s3, w1, w2, w3, p.color, &uniforms,
                );
            }
            if p.has_ring {
                let ring_inner = p.radius * p.ring_inner_scale;
                let ring_outer = p.radius * p.ring_outer_scale;
                let ring_color = p.color.lerp(Vector3::new(0.9, 0.9, 0.95), 0.35);
                draw_planet_ring(
                    &mut fb,
                    &mut uniforms,
                    pos_world,
                    ring_inner,
                    ring_outer,
                    p.ring_tilt_deg,
                    ring_color,
                    96,
                    bird_eye,
                    screen_center,
                    cam_pos_world,
                    pixels_per_unit,
                    camera_distance,
                    cam_yaw,
                    cam_pitch,
                );
            }
        }

        for m in &mut moons {
            m.phase += m.orbit_speed * dt;
            let parent = &planets[m.parent_index];
            let theta = t * parent.orbit_speed;
            let parent_world = Vector3::new(
                parent.orbit_radius * theta.cos(),
                0.0,
                parent.orbit_radius * theta.sin(),
            );
            let moon_pos = parent_world
                + Vector3::new(
                    m.orbit_radius * m.phase.cos(),
                    0.0,
                    m.orbit_radius * m.phase.sin(),
                );
            uniforms.pattern = 200;
            uniforms.mode = m.mode;
            uniforms.performance_mode = !high_quality;
            for f in &sphere.faces {
                let w1 = rotate_xyz(
                    sphere.vertices[f[0]] * m.radius,
                    Vector3::new(0.0, t * 40.0, 0.0),
                ) + moon_pos;
                let w2 = rotate_xyz(
                    sphere.vertices[f[1]] * m.radius,
                    Vector3::new(0.0, t * 40.0, 0.0),
                ) + moon_pos;
                let w3 = rotate_xyz(
                    sphere.vertices[f[2]] * m.radius,
                    Vector3::new(0.0, t * 40.0, 0.0),
                ) + moon_pos;
                let (s1, s2, s3) = if bird_eye {
                    (
                        world_to_screen_top(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                    )
                } else {
                    (
                        world_to_screen(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                    )
                };
                crate::triangle::triangle_filled_world(
                    &mut fb,
                    s1,
                    s2,
                    s3,
                    w1,
                    w2,
                    w3,
                    Vector3::new(0.8, 0.8, 0.85),
                    &uniforms,
                );
            }
        }

        fb.set_current_color(Color::new(150, 200, 255, 140));
        for p in &planets {
            let mut prev: Option<Vector2> = None;
            let mut first: Option<Vector2> = None;
            for i in 0..=120 {
                let ang = i as f32 / 120.0 * 2.0 * PI;
                let world_pt =
                    Vector3::new(p.orbit_radius * ang.cos(), 0.0, p.orbit_radius * ang.sin());
                let proj = if bird_eye {
                    world_to_screen_top(
                        world_pt,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                    )
                } else {
                    world_to_screen(
                        world_pt,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                        cam_yaw,
                        cam_pitch,
                    )
                };
                let screen_pt = Vector2::new(proj.x, proj.y);
                if let Some(prev_pt) = prev {
                    line::line(&mut fb, prev_pt, screen_pt);
                } else {
                    first = Some(screen_pt);
                }
                prev = Some(screen_pt);
            }
            if let (Some(first_pt), Some(last_pt)) = (first, prev) {
                line::line(&mut fb, last_pt, first_pt);
            }
        }

        fb.set_current_color(Color::new(120, 210, 255, 110));
        for m in &moons {
            let parent = &planets[m.parent_index];
            let theta = t * parent.orbit_speed;
            let parent_world = Vector3::new(
                parent.orbit_radius * theta.cos(),
                0.0,
                parent.orbit_radius * theta.sin(),
            );
            let mut prev: Option<Vector2> = None;
            let mut first: Option<Vector2> = None;
            for i in 0..=60 {
                let ang = i as f32 / 60.0 * 2.0 * PI;
                let world_pt = parent_world
                    + Vector3::new(m.orbit_radius * ang.cos(), 0.0, m.orbit_radius * ang.sin());
                let proj = if bird_eye {
                    world_to_screen_top(
                        world_pt,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                    )
                } else {
                    world_to_screen(
                        world_pt,
                        screen_center,
                        cam_pos_world,
                        pixels_per_unit,
                        camera_distance,
                        cam_yaw,
                        cam_pitch,
                    )
                };
                let screen_pt = Vector2::new(proj.x, proj.y);
                if let Some(prev_pt) = prev {
                    line::line(&mut fb, prev_pt, screen_pt);
                } else {
                    first = Some(screen_pt);
                }
                prev = Some(screen_pt);
            }
            if let (Some(first_pt), Some(last_pt)) = (first, prev) {
                line::line(&mut fb, last_pt, first_pt);
            }
        }
        fb.set_current_color(Color::WHITE);

        if let Some(mesh) = &snoopy_mesh {
            let rot = Vector3::new(180.0 + ship_pitch_angle, cam_yaw, ship_bank_angle);
            for f in &mesh.faces {
                let w1 = rotate_xyz(mesh.vertices[f[0]] * snoopy_scale, rot) + ship_world_pos;
                let w2 = rotate_xyz(mesh.vertices[f[1]] * snoopy_scale, rot) + ship_world_pos;
                let w3 = rotate_xyz(mesh.vertices[f[2]] * snoopy_scale, rot) + ship_world_pos;
                let (s1, s2, s3) = if bird_eye {
                    (
                        world_to_screen_top(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                        world_to_screen_top(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                        ),
                    )
                } else {
                    (
                        world_to_screen(
                            w1,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w2,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                        world_to_screen(
                            w3,
                            screen_center,
                            cam_pos_world,
                            pixels_per_unit,
                            camera_distance,
                            cam_yaw,
                            cam_pitch,
                        ),
                    )
                };
                uniforms.pattern = 999;
                uniforms.mode = 0;
                crate::triangle::triangle_filled_world_no_depth(
                    &mut fb,
                    s1,
                    s2,
                    s3,
                    w1,
                    w2,
                    w3,
                    snoopy_color,
                    &uniforms,
                );
            }
        }

        fb.swap_buffers(&mut rl, &thread);
    }
}
