use raylib::prelude::*;
use std::f32::consts::PI;

#[derive(Copy, Clone, Debug, Default)]
pub struct Fragment {
    pub world_position: Vector3,
    pub color: Vector3, // base color (0..1)
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Uniforms {
    pub time: f32,
    pub pattern: u32, // 0..N patterns selector
    // Static-Shaders compatible fields
    pub mode: u32,              // 0..4 planet modes (Rocky, Gas, Alien, Lava, Ice)
    pub light_dir: Vector3,     // world-space light dir
    pub camera_pos: Vector3,    // world-space camera position
    pub ring_inner: f32,        // ring inner radius (world units)
    pub ring_outer: f32,        // ring outer radius (world units)
    pub ring_enabled: bool,     // enable ring masking/shading
    pub ring_center: Vector3,   // center for ring mask (world units)
    pub performance_mode: bool, // if true, simplify shaders for FPS
}

fn clamp01(x: f32) -> f32 {
    x.max(0.0).min(1.0)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Vector3 {
    let h = h - h.floor();
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - (i as f32);
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i.rem_euclid(6) {
        0 => Vector3::new(v, t, p),
        1 => Vector3::new(q, v, p),
        2 => Vector3::new(p, v, t),
        3 => Vector3::new(p, q, v),
        4 => Vector3::new(t, p, v),
        _ => Vector3::new(v, p, q),
    }
}

fn pattern_rainbow_rings(pos: Vector3, time: f32, base: Vector3) -> Vector3 {
    let angle = pos.x.atan2(pos.z) + time * 0.6;
    let radius = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let stripes = (radius * 12.0 + time * 2.0).sin().abs();
    let hue = ((angle / (2.0 * PI)) % 1.0 + 1.0) % 1.0;
    let rgb = hsv_to_rgb(hue, 1.0, 1.0);
    base * (1.0 - 0.7 * stripes) + rgb * (0.7 * stripes)
}

fn pattern_checker3d(pos: Vector3, time: f32, _base: Vector3) -> Vector3 {
    let scale = 4.0;
    let xi = (pos.x * scale + time * 0.2).floor() as i32;
    let yi = (pos.y * scale).floor() as i32;
    let zi = (pos.z * scale).floor() as i32;
    let parity = ((xi & 1) ^ (yi & 1) ^ (zi & 1)) as f32;
    // two-tone but tint it with a hue over time
    let hue = ((time * 0.1) % 1.0 + 1.0) % 1.0;
    let tint = hsv_to_rgb(hue, 0.8, 1.0);
    let a = Vector3::new(0.15, 0.15, 0.15);
    let b = Vector3::new(0.95, 0.95, 0.95);
    let base = if parity > 0.0 { a } else { b };
    base * 0.6 + tint * 0.4
}

fn pattern_grid_lines(pos: Vector3, time: f32, base: Vector3) -> Vector3 {
    let scale = 6.0;
    let fx = (pos.x * scale + time * 0.5).fract().abs();
    let fy = (pos.y * scale).fract().abs();
    let fz = (pos.z * scale).fract().abs();
    let line = (fx.min(1.0 - fx).min(fy.min(1.0 - fy)).min(fz.min(1.0 - fz)) < 0.04) as i32 as f32;
    let hue = ((pos.y * 0.2 + time * 0.2) % 1.0 + 1.0) % 1.0;
    let line_color = hsv_to_rgb(hue, 1.0, 1.0);
    base * (1.0 - line) + line_color * line
}

fn pattern_stripes(pos: Vector3, time: f32, base: Vector3) -> Vector3 {
    let v = ((pos.x * 8.0 + pos.y * 3.0 + time * 2.0).sin().abs()).powf(0.8);
    let hue = ((pos.z * 0.3 + time * 0.3) % 1.0 + 1.0) % 1.0;
    let color = hsv_to_rgb(hue, 0.9, v);
    base * 0.3 + color * 0.7
}

fn pattern_plasma(pos: Vector3, time: f32, base: Vector3) -> Vector3 {
    let s = (pos.x * 3.0 + time).sin()
        + (pos.y * 3.0 - time * 1.2).sin()
        + (pos.z * 3.0 + time * 0.6).sin();
    let v = 0.5 + 0.5 * (s * 0.5).sin();
    let hue = ((s * 0.1 + time * 0.1) % 1.0 + 1.0) % 1.0;
    let color = hsv_to_rgb(hue, 0.9, v);
    base * 0.2 + color * 0.8
}

// Red gradient pattern: emphasize red channel with slight time wave
fn pattern_red_gradient(pos: Vector3, time: f32, _base: Vector3) -> Vector3 {
    let d = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let v = (1.0 / (1.0 + d)).max(0.0).min(1.0); // more red towards center
    let pulse = (time * 2.0 + d * 2.0).sin() * 0.1 + 0.9; // subtle pulse
    Vector3::new((v * pulse).max(0.0).min(1.0), 0.1 * v, 0.1 * v)
}

// returns RGB (0..1)
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let base = fragment.color;
    let time = uniforms.time;
    // Static-Shaders dispatch: pattern==200 uses mode-based planets (0..4)
    if uniforms.pattern == 200 {
        return shade_fragment_static(fragment, uniforms);
    }
    if uniforms.pattern == 999 {
        return vec3_clamp01(fragment.color);
    }
    // Extended patterns: >=100 use world-space planet/star shaders
    if uniforms.pattern >= 100 {
        return match uniforms.pattern {
            100 => shade_star_world(pos, time),
            101 => shade_planet_rocky(pos, time),
            102 => shade_planet_gas(pos, time),
            103 => shade_planet_scifi(pos, time),
            _ => shade_planet_rocky(pos, time),
        };
    }

    let col = match uniforms.pattern % 6 {
        0 => pattern_rainbow_rings(pos, time, base),
        1 => pattern_checker3d(pos, time, base),
        2 => pattern_grid_lines(pos, time, base),
        3 => pattern_stripes(pos, time, base),
        4 => pattern_plasma(pos, time, base),
        _ => pattern_red_gradient(pos, time, base),
    };

    Vector3::new(clamp01(col.x), clamp01(col.y), clamp01(col.z))
}

// ---- World-space noise and planet/star shaders (simplified) ----

fn hash12(x: f32, y: f32) -> f32 {
    let mut p3x = (x * 0.1031).fract();
    let mut p3y = (y * 0.1031).fract();
    let mut p3z = (x * 0.1031).fract();
    let d = p3x * (p3y + 33.33) + p3y * (p3z + 33.33) + p3z * (p3x + 33.33);
    p3x = (p3x + d).fract();
    p3y = (p3y + d).fract();
    p3z = (p3z + d).fract();
    ((p3x + p3y) * p3z).fract()
}
fn value_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let a = hash12(ix, iy);
    let b = hash12(ix + 1.0, iy);
    let c = hash12(ix, iy + 1.0);
    let d = hash12(ix + 1.0, iy + 1.0);
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let m1 = a + (b - a) * ux;
    let m2 = c + (d - c) * ux;
    m1 + (m2 - m1) * uy
}
fn fbm(mut x: f32, mut y: f32, oct: i32) -> f32 {
    let mut amp = 0.5;
    let mut f = 0.0;
    for _ in 0..oct {
        f += amp * value_noise(x, y);
        x *= 2.02;
        y *= 2.02;
        amp *= 0.5;
    }
    f
}
fn cellular(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let mut dmin = 1e9f32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let fx = (xi + dx) as f32;
            let fy = (yi + dy) as f32;
            let hx = hash12(fx, fy);
            let hy = hash12(fx + 17.0, fy + 31.0);
            let px = fx + hx;
            let py = fy + hy;
            let ddx = x - px;
            let ddy = y - py;
            let d = (ddx * ddx + ddy * ddy).sqrt();
            if d < dmin {
                dmin = d;
            }
        }
    }
    1.0 - (dmin / 1.414).clamp(0.0, 1.0)
}
fn v3_normalize(v: Vector3) -> Vector3 {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt().max(1e-6);
    Vector3::new(v.x / len, v.y / len, v.z / len)
}
fn kelvin_to_rgb(temp_k: f32) -> Vector3 {
    let t = (temp_k / 6500.0).clamp(0.2, 2.0);
    let c1 = Vector3::new(0.9, 0.2, 0.0);
    let c2 = Vector3::new(1.0, 0.5, 0.0);
    let c3 = Vector3::new(1.0, 0.9, 0.2);
    let c4 = Vector3::new(0.95, 0.98, 1.0);
    if t < 0.6 {
        c1.lerp(c2, (t - 0.2) / 0.4)
    } else if t < 1.0 {
        c2.lerp(c3, (t - 0.6) / 0.4)
    } else {
        c3.lerp(c4, (t - 1.0) / 1.0)
    }
}

// --------- Static-Shaders compatible helpers ---------
fn vec3_clamp01(v: Vector3) -> Vector3 {
    Vector3::new(clamp01(v.x), clamp01(v.y), clamp01(v.z))
}
fn lon_lat_from_pos(p: Vector3) -> (f32, f32, Vector3) {
    let n = v3_normalize(p);
    (n.z.atan2(n.x), n.y.asin(), n)
}
fn lighting(n: Vector3, view: Vector3, light_dir: Vector3) -> (f32, f32) {
    let l = v3_normalize(light_dir);
    let lam = (n.x * l.x + n.y * l.y + n.z * l.z).max(0.0);
    let rim = (1.0 - (n.x * view.x + n.y * view.y + n.z * view.z).max(0.0)).powf(2.0);
    (lam, rim)
}

// Ring mask relative to world origin (approx). For planet-centered rings,
// pass world positions offset by planet center when calling triangle raster.
fn ring_mask(fragment: &Fragment, u: &Uniforms) -> bool {
    if !u.ring_enabled {
        return true;
    }
    let dx = fragment.world_position.x - u.ring_center.x;
    let dz = fragment.world_position.z - u.ring_center.z;
    let r = (dx * dx + dz * dz).sqrt();
    (r >= u.ring_inner) && (r <= u.ring_outer)
}

// Mode-based dispatcher (0..4)
fn shade_fragment_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.ring_enabled && !ring_mask(fragment, u) {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    match u.mode {
        0 => rocky_planet_static(fragment, u),
        1 => gas_giant_static(fragment, u),
        2 => scifi_planet_static(fragment, u),
        3 => lava_planet_static(fragment, u),
        4 => ice_planet_static(fragment, u),
        _ => rocky_planet_static(fragment, u),
    }
}

fn rocky_planet_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.performance_mode {
        return Vector3::new(0.16, 0.45, 0.12);
    }
    let tint = fragment.color;
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_position);
    let view = v3_normalize(Vector3::new(
        u.camera_pos.x - fragment.world_position.x,
        u.camera_pos.y - fragment.world_position.y,
        u.camera_pos.z - fragment.world_position.z,
    ));
    let (lam, rim) = lighting(n, view, u.light_dir);
    let cont = fbm(lon * 2.5, lat * 2.5, 5);
    let land_mask = ((cont - 0.50) * 20.0).tanh() * 0.5 + 0.5;
    let equator = 1.0 - ((lat.abs() - 0.2) / 0.4).clamp(0.0, 1.0);
    let ice = ((lat.abs() - 0.85) / 0.10).clamp(0.0, 1.0);
    let rough = fbm(lon * 8.0 + 0.3, lat * 8.0 + 0.1 * u.time, 5);
    let strata = (lat * 40.0 + lon * 20.0).sin() * 0.5 + 0.5;
    let mineral = fbm(lon * 16.0 + 0.5 * u.time, lat * 16.0, 3);
    let ocean = Vector3::new(0.03, 0.40, 0.38);
    let landc = Vector3::new(0.16, 0.45, 0.12);
    let desert = Vector3::new(0.60, 0.55, 0.25);
    let icec = Vector3::new(0.85, 0.95, 1.00);
    let strata_col = Vector3::new(0.35, 0.28, 0.18).lerp(Vector3::new(0.55, 0.45, 0.30), strata);
    let mineral_col = Vector3::new(0.7, 0.6, 0.8).lerp(Vector3::new(0.4, 0.3, 0.5), mineral);
    let base_land = landc * (1.0 - 0.7 * equator) + desert * (0.7 * equator);
    let mut base = ocean * (1.0 - land_mask) + base_land * land_mask;
    base = base * (1.0 - ice) + icec * ice;
    base = base.lerp(strata_col, 0.15 * strata);
    base = base.lerp(mineral_col, 0.10 * mineral);
    base *= 0.9 + 0.2 * rough;
    base = base.lerp(tint, 0.35);
    let pulse = (u.time * 0.6).sin() * 0.5 + 0.5;
    let emission = 0.10 * (strata * mineral * land_mask) + 0.08 * pulse * ice;
    let glow_color = Vector3::new(0.55, 0.25, 0.65);
    let col = base * (0.25 + 0.75 * lam)
        + glow_color * emission
        + Vector3::new(1.0, 1.0, 1.0) * (0.10 * rim);
    vec3_clamp01(col)
}

fn gas_giant_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.performance_mode {
        return Vector3::new(0.20, 0.75, 0.55);
    }
    let tint = fragment.color;
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_position);
    let view = v3_normalize(Vector3::new(
        u.camera_pos.x - fragment.world_position.x,
        u.camera_pos.y - fragment.world_position.y,
        u.camera_pos.z - fragment.world_position.z,
    ));
    let (lam, rim) = lighting(n, view, u.light_dir);
    let warp = fbm(lon * 2.0 + u.time * 0.2, lat * 4.0, 4);
    let band = (lat * 18.0 + 0.15 * warp).sin();
    let k = 0.5 + 0.5 * band;
    let band_a = Vector3::new(0.10, 0.65, 0.55);
    let band_b = Vector3::new(0.70, 0.95, 0.30);
    let mut base = band_a.lerp(band_b, k);
    // eye of storm
    let lat0 = 0.2;
    let lon0 = -0.7 + 0.15 * (0.3 * u.time).sin();
    let d = ((lat - lat0).powf(2.0) + (lon - lon0).powf(2.0)).sqrt();
    let eye = (1.0 - (d / 0.25).clamp(0.0, 1.0)).powf(2.0);
    base = base.lerp(Vector3::new(0.85, 0.55, 0.95), eye);
    base = base.lerp(tint, 0.4);
    if u.ring_enabled {
        // Ring color around ring_center
        let dx = fragment.world_position.x - u.ring_center.x;
        let dz = fragment.world_position.z - u.ring_center.z;
        let r = (dx * dx + dz * dz).sqrt();
        let mask = (r >= u.ring_inner) && (r <= u.ring_outer);
        if !mask {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        let bands = 0.5 + 0.5 * (r * 90.0 + u.time * 0.35).sin();
        let grains = fbm(r * 18.0, 0.0, 4);
        let ang = (dx.atan2(dz) * 8.0 + u.time * 0.5).sin() * 0.5 + 0.5;
        let color_a = Vector3::new(0.15, 0.55, 0.50);
        let color_b = Vector3::new(0.60, 0.90, 0.35);
        let color_c = Vector3::new(0.65, 0.30, 0.85);
        let base_ring = color_a
            .lerp(color_b, bands)
            .lerp(color_c, 0.35 * ang + 0.25 * grains);
        let rimr = (1.0 - ((r - u.ring_inner) / (u.ring_outer - u.ring_inner)).abs()).powf(2.0);
        let pulse = (u.time * 0.8).sin() * 0.5 + 0.5;
        let light = 0.6 + 0.25 * rimr + 0.15 * pulse;
        return vec3_clamp01(base_ring * light + Vector3::new(0.18, 0.18, 0.18) * rimr);
    }
    let pulse = (u.time * 0.5).sin() * 0.5 + 0.5;
    let col = base * (0.30 + 0.70 * lam) + Vector3::new(0.4, 0.15, 0.55) * (0.10 * rim * pulse);
    vec3_clamp01(col)
}

fn scifi_planet_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.performance_mode {
        return Vector3::new(0.45, 0.10, 0.55);
    }
    let tint = fragment.color;
    let (lon, _lat, n) = lon_lat_from_pos(fragment.world_position);
    let view = v3_normalize(Vector3::new(
        u.camera_pos.x - fragment.world_position.x,
        u.camera_pos.y - fragment.world_position.y,
        u.camera_pos.z - fragment.world_position.z,
    ));
    let (lam, rim) = lighting(n, view, u.light_dir);
    let f = fbm(lon * 7.0 + 0.2 * u.time, 0.0, 5);
    let cracks = ((0.62 - f) * 50.0).clamp(0.0, 1.0);
    let core = Vector3::new(0.18, 0.08, 0.22);
    let lava = Vector3::new(1.0, 0.5, 0.2);
    let mut base = core.lerp(lava, cracks * 0.8);
    base = base.lerp(tint, 0.55);
    let pulse_fast = (u.time * 1.5).sin() * 0.5 + 0.5;
    let emission = (cracks * (0.4 + 0.4 * (1.0 - lam)) + rim * 0.20 + 0.25 * pulse_fast) * cracks;
    let emissive_color = Vector3::new(0.35, 0.9, 0.45).lerp(
        Vector3::new(0.70, 0.25, 0.95),
        (u.time * 0.4).sin() * 0.5 + 0.5,
    );
    let col = base * (0.20 + 0.80 * lam) + emissive_color * emission;
    vec3_clamp01(col)
}

fn lava_planet_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.performance_mode {
        return Vector3::new(0.85, 0.35, 0.10);
    }
    let tint = fragment.color;
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_position);
    let view = v3_normalize(Vector3::new(
        u.camera_pos.x - fragment.world_position.x,
        u.camera_pos.y - fragment.world_position.y,
        u.camera_pos.z - fragment.world_position.z,
    ));
    let (lam, rim) = lighting(n, view, u.light_dir);
    let flow = fbm(lon * 5.5 + 0.25 * u.time, lat * 5.5 - 0.15 * u.time, 5);
    let crust_noise = fbm(lon * 12.0, lat * 12.0 + 0.2 * u.time, 4);
    let cracks = ((flow - 0.45) * 25.0).clamp(0.0, 1.0);
    let crust = Vector3::new(0.05, 0.03, 0.02);
    let lava_a = Vector3::new(0.90, 0.30, 0.05);
    let lava_b = Vector3::new(1.0, 0.85, 0.25);
    let lava_col = lava_a.lerp(lava_b, (u.time * 0.7).sin() * 0.5 + 0.5);
    let mut base = crust.lerp(lava_col, cracks);
    base = base.lerp(tint, 0.3);
    let pulse = (u.time * 1.2).sin() * 0.5 + 0.5;
    let emission = (cracks * (0.6 + 0.4 * pulse) + rim * 0.15) * (0.7 + 0.3 * crust_noise);
    let col = base * (0.28 + 0.72 * lam) + lava_col * emission;
    vec3_clamp01(col)
}

fn ice_planet_static(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    if u.performance_mode {
        return Vector3::new(0.50, 0.80, 0.95);
    }
    let tint = fragment.color;
    let (lon, lat, n) = lon_lat_from_pos(fragment.world_position);
    let view = v3_normalize(Vector3::new(
        u.camera_pos.x - fragment.world_position.x,
        u.camera_pos.y - fragment.world_position.y,
        u.camera_pos.z - fragment.world_position.z,
    ));
    let (lam, rim) = lighting(n, view, u.light_dir);
    let base_noise = fbm(lon * 3.5, lat * 3.5, 5);
    let crack_noise = fbm(lon * 14.0 + 0.1 * u.time, lat * 14.0, 4);
    let crystal = fbm(lon * 28.0 - 0.2 * u.time, lat * 28.0 + 0.15 * u.time, 3);
    let cracks = ((crack_noise - 0.55) * 30.0).clamp(0.0, 1.0);
    let snow = (base_noise * 0.6 + crystal * 0.4).clamp(0.0, 1.0);
    let ice_c = Vector3::new(0.50, 0.80, 0.95);
    let deep_c = Vector3::new(0.08, 0.25, 0.40);
    let crack_c = Vector3::new(0.85, 0.95, 1.0);
    let mut base = deep_c.lerp(ice_c, snow).lerp(crack_c, cracks * 0.7);
    base = base.lerp(tint, 0.5);
    let aurora = (lat * 8.0 + lon * 2.0 + u.time * 0.4).sin() * 0.5 + 0.5;
    let aurora_col = Vector3::new(0.15, 0.85, 0.60).lerp(Vector3::new(0.55, 0.25, 0.95), crystal);
    let emission = (aurora * 0.25 + rim * 0.18) * (0.6 + 0.4 * crystal);
    let col = base * (0.30 + 0.70 * lam) + aurora_col * emission;
    vec3_clamp01(col)
}

fn shade_star_world(world: Vector3, time: f32) -> Vector3 {
    let p = v3_normalize(world);
    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let s = 2.5;
    let t = time * 0.6;
    let base = fbm(lon * 3.0 * s + 0.30 * t, lat * 3.0 * s - 0.25 * t, 5);
    let ridge =
        1.0 - (2.0 * value_noise(lon * 2.0 * s - 0.2 * t, lat * 2.0 * s + 0.17 * t) - 1.0).abs();
    let cell = cellular(lon * 4.0 * s + 0.1 * t, lat * 4.0 * s - 0.12 * t);
    let pulse = (t * 1.2).sin() * 0.5 + 0.5;
    let intensity = (0.55 * base + 0.35 * ridge + 0.25 * cell) * (0.90 + 0.40 * pulse);
    let base_col = kelvin_to_rgb(6200.0);
    let hot = Vector3::new(1.0, 0.98, 0.95);
    let mut col = base_col.lerp(hot, (intensity * 1.2).clamp(0.0, 1.0));
    let spikes = (ridge * 1.6 + cell * 1.2) * 0.55;
    let rim = (1.0 - p.y.abs()).powf(3.0);
    let corona = rim * (0.9 + 0.8 * pulse) + 0.30 * spikes;
    col += Vector3::new(1.0, 0.8, 0.3) * corona;
    Vector3::new(clamp01(col.x), clamp01(col.y), clamp01(col.z))
}

fn shade_planet_rocky(world: Vector3, time: f32) -> Vector3 {
    let p = v3_normalize(world);
    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let f = fbm(lon * 5.0 + 0.1 * time, lat * 5.0 - 0.07 * time, 5);
    let ice = (lat.abs() * 1.8 - 0.6).clamp(0.0, 1.0);
    let land = Vector3::new(0.2, 0.5, 0.25);
    let desert = Vector3::new(0.7, 0.65, 0.4);
    let icec = Vector3::new(0.8, 0.9, 1.0);
    let base = land
        .lerp(desert, (f * 1.2 - 0.2).clamp(0.0, 1.0))
        .lerp(icec, ice * 0.9);
    base
}
fn shade_planet_gas(world: Vector3, time: f32) -> Vector3 {
    let p = v3_normalize(world);
    let r = p.y; // bands in latitude
    let bands = (r * 20.0 + time * 0.6).sin().abs();
    let grains = fbm(r * 30.0, 0.0, 3);
    let a = Vector3::new(0.1, 0.8, 0.7);
    let b = Vector3::new(0.6, 1.0, 0.3);
    let c = Vector3::new(0.7, 0.5, 0.9);
    let base = a.lerp(b, 0.6 * bands + 0.3 * grains).lerp(c, 0.2 * bands);
    base
}
fn shade_planet_scifi(world: Vector3, time: f32) -> Vector3 {
    let p = v3_normalize(world);
    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let f = fbm(lon * 7.0 + 0.2 * time, lat * 7.0, 5);
    let cracks = ((0.62 - f) * 50.0).clamp(0.0, 1.0);
    let core = Vector3::new(0.18, 0.08, 0.22);
    let lava = Vector3::new(1.0, 0.5, 0.2);
    core.lerp(lava, cracks * 0.8)
}
