use glam::Vec3;

#[derive(Copy, Clone, Default)]
pub struct Fragment {
    pub world_pos: Vec3,
}

#[derive(Copy, Clone, Default)]
pub struct StarUniforms {
    pub time: f32,
    pub camera_pos: Vec3,
    pub temp_kelvin: f32,
    pub disp_amp: f32,
    pub disp_freq: f32,
    pub speed: f32,
}

#[inline]
fn clamp01(x: f32) -> f32 { x.max(0.0).min(1.0) }
#[inline]
fn vec3_clamp01(v: Vec3) -> Vec3 { Vec3::new(clamp01(v.x), clamp01(v.y), clamp01(v.z)) }

fn hash12(x: f32, y: f32) -> f32 {
    let mut p3x = (x * 0.1031).fract();
    let mut p3y = (y * 0.1031).fract();
    let mut p3z = (x * 0.1031).fract();
    let d = p3x * (p3y + 33.33) + p3y * (p3z + 33.33) + p3z * (p3x + 33.33);
    p3x = (p3x + d).fract(); p3y = (p3y + d).fract(); p3z = (p3z + d).fract();
    ((p3x + p3y) * p3z).fract()
}

fn noise(x: f32, y: f32) -> f32 {
    let ix = x.floor(); let iy = y.floor();
    let fx = x - ix;    let fy = y - iy;
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
    for _ in 0..oct { f += amp * noise(x, y); x *= 2.02; y *= 2.02; amp *= 0.5; }
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
            let d = (ddx*ddx + ddy*ddy).sqrt();
            if d < dmin { dmin = d; }
        }
    }
    1.0 - (dmin / 1.414).clamp(0.0,1.0)
}

fn kelvin_to_rgb(temp_k: f32) -> Vec3 {
    let t = (temp_k / 6500.0).clamp(0.2, 2.0);
    let c1 = Vec3::new(0.9, 0.2, 0.0);
    let c2 = Vec3::new(1.0, 0.5, 0.0);
    let c3 = Vec3::new(1.0, 0.9, 0.2);
    let c4 = Vec3::new(0.95, 0.98, 1.0);
    if t < 0.6 { c1.lerp(c2, (t-0.2)/0.4) }
    else if t < 1.0 { c2.lerp(c3, (t-0.6)/0.4) }
    else { c3.lerp(c4, (t-1.0)/1.0) }
}

pub fn star_color(fragment: Fragment, u: StarUniforms) -> Vec3 {
    let p = fragment.world_pos.normalize();
    let view = (u.camera_pos - fragment.world_pos).normalize();
    let rim = (1.0 - p.dot(view).abs().clamp(0.0,1.0)).powf(3.0);

    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let s = u.disp_freq.max(0.5);

    let t = u.time * u.speed;
    let base = fbm(lon * 3.0 * s + 0.30*t, lat * 3.0 * s - 0.25*t, 5);
    let ridge = 1.0 - (2.0*noise(lon*2.0*s - 0.2*t, lat*2.0*s + 0.17*t) - 1.0).abs();
    let cell = cellular(lon * 4.0 * s + 0.1*t, lat * 4.0 * s - 0.12*t);
    let pulse = (t * 1.2).sin() * 0.5 + 0.5;

    let intensity = (0.5*base + 0.3*ridge + 0.2*cell) * (0.85 + 0.30*pulse);
    let base_col = kelvin_to_rgb(u.temp_kelvin);
    let hot = Vec3::new(1.0, 0.98, 0.95);
    let mut col = base_col.lerp(hot, (intensity*1.2).clamp(0.0,1.0));

    let spikes = (ridge * 1.5 + cell * 1.1) * 0.5;
    let corona = rim * (0.8 + 0.6*pulse) + 0.25*spikes;
    col += Vec3::new(1.0, 0.8, 0.3) * corona;

    vec3_clamp01(col)
}

pub fn star_displacement(normal_ws: Vec3, u: StarUniforms) -> f32 {
    let p = normal_ws;
    let lon = p.z.atan2(p.x);
    let lat = p.y.asin();
    let s = u.disp_freq.max(0.5);
    let t = u.time * u.speed;
    let base = fbm(lon*3.0*s + 0.25*t, lat*3.0*s - 0.22*t, 5);
    let ridge = 1.0 - (2.0*noise(lon*2.0*s - 0.18*t, lat*2.0*s + 0.14*t) - 1.0).abs();
    let cell = cellular(lon*4.0*s + 0.1*t, lat*4.0*s - 0.1*t);
    (0.55*base + 0.30*ridge + 0.25*cell) * u.disp_amp
}
